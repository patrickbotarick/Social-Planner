use crate::{
    error::AppError,
    trello_api::{http_response, HttpResponse},
    trello_domain::{error, remote_id},
};
use reqwest::blocking::{Client, Request};
use std::{
    net::{IpAddr, SocketAddr, ToSocketAddrs},
    sync::mpsc,
    time::{Duration, Instant},
};
use url::Url;

const FILE_HOST: &str = "files.trello.com";
const MAX_REDIRECTS: usize = 3;

pub fn is_download_path(path: &str) -> bool {
    let parts: Vec<_> = path.split('/').collect();
    parts.len() == 6
        && parts[0] == "cards"
        && remote_id(parts[1])
        && parts[2] == "attachments"
        && remote_id(parts[3])
        && parts[4] == "download"
        && !parts[5].is_empty()
}

// URLs assinadas só vivem em memória. Nunca incluir o destino em erros/logs.
fn destination(current: &Url, location: &str, followed: usize) -> Result<Url, AppError> {
    if followed >= MAX_REDIRECTS {
        return Err(error("TRELLO_REDIRECT"));
    }
    let next = current
        .join(location)
        .map_err(|_| error("TRELLO_REDIRECT"))?;
    if next.scheme() != "https"
        || next.host_str() != Some(FILE_HOST)
        || next.port().is_some()
        || !next.username().is_empty()
        || next.password().is_some()
        || next.fragment().is_some()
        || next.query_pairs().any(|(key, _)| {
            matches!(
                key.to_ascii_lowercase().as_str(),
                "token"
                    | "key"
                    | "api_key"
                    | "apikey"
                    | "access_token"
                    | "oauth_token"
                    | "oauth_consumer_key"
                    | "authorization"
            )
        })
    {
        return Err(error("TRELLO_REDIRECT"));
    }
    Ok(next)
}

pub fn follow_download(
    mut response: HttpResponse,
    mut current: Url,
    started: Instant,
    timeout: Duration,
    mut fetch: impl FnMut(&Url, Duration) -> Result<HttpResponse, AppError>,
) -> Result<HttpResponse, AppError> {
    let mut followed = 0;
    while matches!(response.status, 301 | 302 | 303 | 307 | 308) {
        let next = destination(
            &current,
            response
                .location
                .as_deref()
                .ok_or_else(|| error("TRELLO_REDIRECT"))?,
            followed,
        )?;
        let remaining = timeout
            .checked_sub(started.elapsed())
            .filter(|d| !d.is_zero())
            .ok_or_else(|| error("TRELLO_NETWORK"))?;
        drop(response);
        response = fetch(&next, remaining)?;
        current = next;
        followed += 1;
    }
    if response.status == 200 {
        if let Some(value) = &response.content_type {
            let mime = value
                .split(';')
                .next()
                .unwrap_or("")
                .trim()
                .to_ascii_lowercase();
            if !matches!(
                mime.as_str(),
                "image/jpeg"
                    | "image/png"
                    | "image/webp"
                    | "video/mp4"
                    | "application/octet-stream"
                    | "binary/octet-stream"
            ) {
                return Err(AppError::media());
            }
        }
    }
    // Ausência de Content-Type não prova validade: MediaStore sempre inspeciona os bytes.
    Ok(response)
}

fn public_address(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            let [a, b, c, _] = ip.octets();
            !(matches!(a, 0 | 10 | 127)
                || a >= 224
                || a == 100 && (64..=127).contains(&b)
                || a == 169 && b == 254
                || a == 172 && (16..=31).contains(&b)
                || a == 192 && (b == 168 || b == 0 && matches!(c, 0 | 2) || b == 88 && c == 99)
                || a == 198 && (matches!(b, 18 | 19) || b == 51 && c == 100)
                || a == 203 && b == 0 && c == 113)
        }
        IpAddr::V6(ip) => {
            let s = ip.segments();
            s[0] & 0xe000 == 0x2000
                && !(s[0] == 0x2001 && (s[1] < 0x200 || s[1] == 0xdb8) || s[0] == 0x2002)
        }
    }
}

fn public_addresses(addresses: Vec<SocketAddr>) -> Result<Vec<SocketAddr>, AppError> {
    if addresses.is_empty()
        || addresses
            .iter()
            .any(|a| a.port() != 443 || !public_address(a.ip()))
    {
        return Err(error("TRELLO_REDIRECT"));
    }
    Ok(addresses)
}

fn file_client(addresses: &[SocketAddr], timeout: Duration) -> Result<Client, AppError> {
    Client::builder()
        .https_only(true)
        .redirect(reqwest::redirect::Policy::none())
        .no_proxy()
        .resolve_to_addrs(FILE_HOST, addresses)
        .connect_timeout(timeout.min(Duration::from_secs(10)))
        .timeout(timeout)
        .build()
        .map_err(|_| error("TRELLO_NETWORK"))
}

fn file_request(client: &Client, url: &Url) -> Result<Request, AppError> {
    // Cliente novo: sem Authorization, cookies, Referer ou headers herdados da API.
    client
        .get(url.clone())
        .build()
        .map_err(|_| error("TRELLO_NETWORK"))
}

pub fn fetch_file(url: &Url, timeout: Duration) -> Result<HttpResponse, AppError> {
    destination(url, url.as_str(), 0)?;
    let started = Instant::now();
    let (send, recv) = mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let resolved = (FILE_HOST, 443)
            .to_socket_addrs()
            .map(|a| a.collect::<Vec<_>>());
        let _ = send.send(resolved);
    });
    let addresses = recv
        .recv_timeout(timeout.min(Duration::from_secs(10)))
        .map_err(|_| error("TRELLO_NETWORK"))?
        .map_err(|_| error("TRELLO_NETWORK"))?;
    let addresses = public_addresses(addresses)?;
    let remaining = timeout
        .checked_sub(started.elapsed())
        .filter(|d| !d.is_zero())
        .ok_or_else(|| error("TRELLO_NETWORK"))?;
    // DNS validado e fixado nesta conexão; sem nova resolução entre verificação e uso.
    let client = file_client(&addresses, remaining)?;
    let response = client
        .execute(file_request(&client, url)?)
        .map_err(|_| error("TRELLO_NETWORK"))?;
    Ok(http_response(response))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    fn response(status: u16, location: Option<&str>, mime: Option<&str>) -> HttpResponse {
        HttpResponse {
            status,
            location: location.map(str::to_owned),
            content_type: mime.map(str::to_owned),
            length: Some(0),
            retry_after: None,
            body: Box::new(Cursor::new(Vec::<u8>::new())),
        }
    }
    fn original() -> Url {
        Url::parse("https://api.trello.com/1/cards/aaaaaaaaaaaaaaaaaaaaaaaa/attachments/bbbbbbbbbbbbbbbbbbbbbbbb/download/image.png").unwrap()
    }
    #[test]
    fn legitimate_302_download_reaches_file_host() {
        let mut calls = 0;
        let result = follow_download(
            response(
                302,
                Some("https://files.trello.com/file.png?signature=opaque"),
                Some("text/plain"),
            ),
            original(),
            Instant::now(),
            Duration::from_secs(30),
            |url, remaining| {
                calls += 1;
                assert_eq!(url.host_str(), Some(FILE_HOST));
                assert!(remaining <= Duration::from_secs(30));
                Ok(response(200, None, Some("image/png")))
            },
        )
        .unwrap();
        assert_eq!(result.status, 200);
        assert_eq!(calls, 1);
    }
    #[test]
    fn every_redirect_must_be_allowed() {
        for target in [
            "https://evil.test/x",
            "https://files.trello.com.evil.test/x",
            "https://sub.files.trello.com/x",
            "http://files.trello.com/x",
            "https://localhost/x",
            "https://127.0.0.1/x",
            "https://10.0.0.1/x",
            "https://192.168.1.2/x",
            "https://[::1]/x",
            "https://[fc00::1]/x",
            "file:///C:/file",
            "https://user:pass@files.trello.com/x",
            "https://files.trello.com:444/x",
            "https://files.trello.com/x?oauth_token=secret",
            "https://api.trello.com/x",
        ] {
            let mut calls = 0;
            let result = follow_download(
                response(302, Some(target), None),
                original(),
                Instant::now(),
                Duration::from_secs(30),
                |_, _| {
                    calls += 1;
                    Ok(response(200, None, None))
                },
            );
            assert!(result.is_err());
            assert_eq!(calls, 0);
            let result = follow_download(
                response(302, Some("https://files.trello.com/start"), None),
                original(),
                Instant::now(),
                Duration::from_secs(30),
                |_, _| Ok(response(302, Some(target), None)),
            );
            assert!(result.is_err());
        }
    }
    #[test]
    fn redirect_chain_is_bounded_and_missing_location_rejected() {
        let mut calls = 0;
        assert!(follow_download(
            response(302, Some("https://files.trello.com/loop"), None),
            original(),
            Instant::now(),
            Duration::from_secs(30),
            |_, _| {
                calls += 1;
                Ok(response(302, Some("/loop"), None))
            }
        )
        .is_err());
        assert_eq!(calls, MAX_REDIRECTS);
        assert!(follow_download(
            response(302, None, None),
            original(),
            Instant::now(),
            Duration::from_secs(30),
            |_, _| panic!("must not request")
        )
        .is_err());
    }
    #[test]
    fn credentials_cookies_and_referer_are_not_forwarded() {
        let addresses = public_addresses(vec!["1.1.1.1:443".parse().unwrap()]).unwrap();
        let client = file_client(&addresses, Duration::from_secs(2)).unwrap();
        let request = file_request(
            &client,
            &Url::parse("https://files.trello.com/file?signature=opaque").unwrap(),
        )
        .unwrap();
        for header in ["authorization", "proxy-authorization", "cookie", "referer"] {
            assert!(!request.headers().contains_key(header));
        }
    }
    #[test]
    fn dns_cannot_resolve_to_local_private_or_reserved_addresses() {
        for ip in [
            "0.0.0.0",
            "10.0.0.1",
            "100.64.1.2",
            "127.0.0.1",
            "169.254.169.254",
            "172.16.1.1",
            "192.168.0.1",
            "198.18.1.1",
            "192.0.2.1",
            "224.0.0.1",
            "255.255.255.255",
            "::1",
            "::",
            "fc00::1",
            "fe80::1",
            "::ffff:127.0.0.1",
            "2001:db8::1",
            "2002:7f00:1::",
        ] {
            assert!(public_addresses(vec![SocketAddr::new(ip.parse().unwrap(), 443)]).is_err());
        }
        assert!(public_addresses(vec![]).is_err());
        assert!(public_addresses(vec![
            "1.1.1.1:443".parse().unwrap(),
            "127.0.0.1:443".parse().unwrap()
        ])
        .is_err());
        assert!(public_addresses(vec![
            "1.1.1.1:443".parse().unwrap(),
            "2600:9000::1:443"
                .parse::<IpAddr>()
                .map(|a| SocketAddr::new(a, 443))
                .unwrap()
        ])
        .is_ok());
    }
    #[test]
    fn html_response_and_expired_redirect_deadline_are_rejected() {
        assert!(follow_download(
            response(200, None, Some("text/html; charset=utf-8")),
            original(),
            Instant::now(),
            Duration::from_secs(30),
            |_, _| panic!("no fetch")
        )
        .is_err());
        assert!(follow_download(
            response(302, Some("https://files.trello.com/file"), None),
            original(),
            Instant::now() - Duration::from_secs(5),
            Duration::from_secs(1),
            |_, _| panic!("no fetch")
        )
        .is_err());
    }
    #[test]
    fn only_typed_attachment_paths_can_follow_redirects() {
        assert!(is_download_path(
            original().path().strip_prefix("/1/").unwrap()
        ));
        for p in [
            "boards/aaaaaaaaaaaaaaaaaaaaaaaa",
            "cards/x/attachments/y/download/file",
            "cards/aaaaaaaaaaaaaaaaaaaaaaaa/attachments/bbbbbbbbbbbbbbbbbbbbbbbb/download/../x",
            "members/me",
        ] {
            assert!(!is_download_path(p));
        }
    }
}
