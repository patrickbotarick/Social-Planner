use crate::{error::AppError, trello_domain::*};
use reqwest::{
    blocking::Client,
    header::{HeaderValue, AUTHORIZATION},
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::HashSet,
    io::{Read, Write},
    sync::atomic::AtomicBool,
    time::Duration,
};

// Sem Debug: credenciais não devem aparecer em diagnósticos.
#[derive(Serialize, Deserialize)]
pub struct Credentials {
    pub key: String,
    pub token: String,
}
impl Credentials {
    pub fn validate(&self) -> Result<(), AppError> {
        if self.key.len() != 32
            || !self.key.bytes().all(|b| b.is_ascii_hexdigit())
            || !(32..=256).contains(&self.token.len())
            || !self
                .token
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        {
            return Err(error("TRELLO_AUTH"));
        }
        Ok(())
    }
    fn header(&self) -> Result<HeaderValue, AppError> {
        self.validate()?;
        let mut h = HeaderValue::from_str(&format!(
            "OAuth oauth_consumer_key=\"{}\", oauth_token=\"{}\"",
            self.key, self.token
        ))
        .map_err(|_| error("TRELLO_AUTH"))?;
        h.set_sensitive(true);
        Ok(h)
    }
}
pub struct HttpResponse {
    pub status: u16,
    pub length: Option<u64>,
    pub retry_after: Option<u64>,
    pub location: Option<String>,
    pub content_type: Option<String>,
    pub body: Box<dyn Read>,
}
pub trait Transport {
    fn get(&mut self, path: &str, query: &[(String, String)]) -> Result<HttpResponse, AppError>;
    fn pause(&self, millis: u64) {
        std::thread::sleep(Duration::from_millis(millis));
    }
}
pub struct OfficialTransport {
    client: Client,
    authorization: HeaderValue,
    timeout: Duration,
}
impl OfficialTransport {
    pub fn new(credentials: &Credentials, timeout: u64) -> Result<Self, AppError> {
        Ok(Self {
            client: Client::builder()
                .https_only(true)
                .redirect(reqwest::redirect::Policy::none())
                .connect_timeout(Duration::from_secs(10))
                .timeout(Duration::from_secs(timeout))
                .user_agent("GMAD-Social-Planner/0.1")
                .build()
                .map_err(|_| error("TRELLO_NETWORK"))?,
            authorization: credentials.header()?,
            timeout: Duration::from_secs(timeout),
        })
    }
}
impl Transport for OfficialTransport {
    fn get(&mut self, path: &str, query: &[(String, String)]) -> Result<HttpResponse, AppError> {
        // path é construído internamente, nunca fornecido pelo IPC ou por attachment.url.
        let mut url =
            url::Url::parse("https://api.trello.com/1/").map_err(|_| error("TRELLO_URL"))?;
        url.set_path(&format!("/1/{path}"));
        self.pause(120); // no máximo ~8 chamadas/s; limite por token é 100/10s.
        let started = std::time::Instant::now();
        let response = self
            .client
            .get(url.clone())
            .query(query)
            .header(AUTHORIZATION, self.authorization.clone())
            .send()
            .map_err(|_| error("TRELLO_NETWORK"))?;
        let response = http_response(response);
        if crate::trello_redirect::is_download_path(path) {
            crate::trello_redirect::follow_download(
                response,
                url,
                started,
                self.timeout,
                crate::trello_redirect::fetch_file,
            )
        } else {
            Ok(response)
        }
    }
}
pub(crate) fn http_response(response: reqwest::blocking::Response) -> HttpResponse {
    HttpResponse {
        status: response.status().as_u16(),
        length: response.content_length(),
        retry_after: response
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse().ok()),
        location: response
            .headers()
            .get("location")
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned),
        content_type: response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned),
        body: Box::new(response),
    }
}
pub fn status_error(status: u16) -> AppError {
    error(match status {
        401 => "TRELLO_AUTH",
        403 | 404 => "TRELLO_ACCESS",
        429 => "TRELLO_RATE",
        300..=399 => "TRELLO_REDIRECT",
        _ => "TRELLO_NETWORK",
    })
}
pub fn request(
    t: &mut dyn Transport,
    path: &str,
    query: &[(String, String)],
    cancel: &AtomicBool,
) -> Result<HttpResponse, AppError> {
    for attempt in 0..3 {
        cancelled(cancel)?;
        let result = t.get(path, query);
        let retry = match &result {
            Ok(r) => r.status == 429 || r.status >= 500,
            Err(e) => e.code == "TRELLO_NETWORK",
        };
        if retry && attempt < 2 {
            let delay = result
                .as_ref()
                .ok()
                .and_then(|r| r.retry_after)
                .unwrap_or(1 << attempt)
                .clamp(1, 10)
                * 1000;
            drop(result);
            for _ in 0..delay / 100 {
                cancelled(cancel)?;
                t.pause(100);
            }
            continue;
        }
        let r = result?;
        return if r.status == 200 {
            Ok(r)
        } else {
            Err(status_error(r.status))
        };
    }
    Err(error("TRELLO_NETWORK"))
}
pub fn copy_bounded(
    reader: &mut dyn Read,
    writer: &mut dyn Write,
    limit: u64,
    expected: Option<u64>,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(u64),
) -> Result<(), AppError> {
    if expected.is_some_and(|n| n > limit) {
        return Err(error("TRELLO_LIMIT"));
    }
    let mut buf = [0u8; 64 * 1024];
    let mut count = 0;
    loop {
        cancelled(cancel)?;
        let n = reader.read(&mut buf).map_err(|_| error("TRELLO_NETWORK"))?;
        if n == 0 {
            break;
        }
        count += n as u64;
        if count > limit {
            return Err(error("TRELLO_LIMIT"));
        }
        writer.write_all(&buf[..n]).map_err(|_| AppError::file())?;
        progress(count);
    }
    cancelled(cancel)?;
    if expected.is_some_and(|n| n != count) {
        return Err(error("TRELLO_NETWORK"));
    }
    Ok(())
}
pub fn json<T: DeserializeOwned>(
    t: &mut dyn Transport,
    path: &str,
    query: &[(String, String)],
    cancel: &AtomicBool,
) -> Result<T, AppError> {
    let mut r = request(t, path, query, cancel)?;
    let mut bytes = Vec::new();
    copy_bounded(
        &mut r.body,
        &mut bytes,
        16 * 1024 * 1024,
        r.length,
        cancel,
        &mut |_| {},
    )?;
    serde_json::from_slice(&bytes).map_err(|_| error("TRELLO_RESPONSE"))
}
#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Account {
    pub full_name: String,
    pub username: String,
}
pub fn account(t: &mut dyn Transport) -> Result<Account, AppError> {
    json(
        t,
        "members/me",
        &[("fields".into(), "fullName,username".into())],
        &AtomicBool::new(false),
    )
}
pub fn read_board(
    t: &mut dyn Transport,
    short: &str,
    cancel: &AtomicBool,
) -> Result<(Board, Vec<TrelloList>, Vec<RemoteCard>), AppError> {
    if short.len() != 8 || !short.bytes().all(|b| b.is_ascii_alphanumeric()) {
        return Err(error("TRELLO_URL"));
    }
    let board: Board = json(
        t,
        &format!("boards/{short}"),
        &[("fields".into(), "name,shortLink".into())],
        cancel,
    )?;
    if !remote_id(&board.id)
        || board.short_link.len() != 8
        || !board.short_link.bytes().all(|b| b.is_ascii_alphanumeric())
    {
        return Err(error("TRELLO_RESPONSE"));
    }
    let lists: Vec<TrelloList> = json(
        t,
        &format!("boards/{}/lists", board.id),
        &[
            ("filter".into(), "all".into()),
            ("fields".into(), "name,closed".into()),
        ],
        cancel,
    )?;
    if lists.iter().any(|l| !remote_id(&l.id)) {
        return Err(error("TRELLO_RESPONSE"));
    }
    let mut cards = Vec::new();
    let mut seen = HashSet::new();
    let mut before: Option<String> = None;
    // Inclui arquivados no snapshot para diferenciar arquivamento de ausência; UI não os seleciona.
    for _ in 0..10 {
        let mut query = vec![
            ("filter".into(), "all".into()),
            ("limit".into(), "1000".into()),
            (
                "fields".into(),
                "name,desc,idList,shortLink,closed,pos,dateLastActivity,due,start,labels".into(),
            ),
            ("attachments".into(), "true".into()),
            (
                "attachment_fields".into(),
                "id,name,url,mimeType,bytes,isUpload,pos".into(),
            ),
        ];
        if let Some(id) = &before {
            query.push(("before".into(), id.clone()));
        }
        let mut page: Vec<RemoteCard> =
            json(t, &format!("boards/{}/cards", board.id), &query, cancel)?;
        let size = page.len();
        if size > 1000 {
            return Err(error("TRELLO_LIMIT"));
        }
        for card in &mut page {
            card.sanitize()?;
            if !seen.insert(card.id.clone()) {
                return Err(error("TRELLO_RESPONSE"));
            }
        }
        // Mongo IDs ordenam por criação; posição do cartão não serve como cursor.
        let cursor = page.iter().map(|c| c.id.clone()).min();
        if let (Some(previous), Some(next)) = (&before, &cursor) {
            if next >= previous {
                return Err(error("TRELLO_RESPONSE"));
            }
        }
        before = cursor;
        cards.extend(page);
        if size < 1000 {
            cards.sort_by(|a, b| {
                a.id_list
                    .cmp(&b.id_list)
                    .then(a.pos.total_cmp(&b.pos))
                    .then(a.id.cmp(&b.id))
            });
            return Ok((board, lists, cards));
        }
    }
    Err(error("TRELLO_LIMIT"))
}
pub fn download(
    t: &mut dyn Transport,
    card: &str,
    a: &Attachment,
    limit: u64,
    out: &mut dyn Write,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(u64),
) -> Result<(), AppError> {
    if !remote_id(card) || !remote_id(&a.id) || !a.is_upload {
        return Err(error("TRELLO_RESPONSE"));
    }
    if a.bytes > limit {
        return Err(error("TRELLO_LIMIT"));
    }
    // Encode o nome como um único segmento; a URL do anexo nunca é destino de rede.
    let mut u = url::Url::parse("https://api.trello.com").map_err(|_| error("TRELLO_URL"))?;
    u.path_segments_mut()
        .map_err(|_| error("TRELLO_URL"))?
        .extend(["cards", card, "attachments", &a.id, "download", &a.name]);
    let mut r = request(t, u.path().trim_start_matches('/'), &[], cancel)?;
    copy_bounded(
        &mut r.body,
        out,
        limit,
        r.length.or((a.bytes > 0).then_some(a.bytes)),
        cancel,
        progress,
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "inspeção remota opt-in; somente metadados, sem download ou escrita operacional"]
    fn inspect_authorized_media_candidates() {
        use crate::trello_vault::{Vault, WindowsVault};
        assert_eq!(std::env::var("GMAD_LIVE_TRELLO_DIAG").as_deref(), Ok("1"));
        let short = std::env::var("GMAD_LIVE_TRELLO_BOARD").expect("explicit board");
        let credentials = WindowsVault
            .load()
            .expect("vault read")
            .expect("configured credential");
        let mut api = OfficialTransport::new(&credentials, 120).expect("transport");
        let (_, lists, cards) =
            read_board(&mut api, &short, &AtomicBool::new(false)).expect("board metadata");
        println!("lists={} cards={}", lists.len(), cards.len());
        for card in cards.iter().filter(|c| !c.closed) {
            let uploads: Vec<_> = card
                .attachments
                .iter()
                .filter(|a| a.is_upload && a.extension().is_some())
                .collect();
            let videos = uploads
                .iter()
                .filter(|a| a.extension() == Some("mp4"))
                .count();
            if videos == 0 && uploads.len() < 2 {
                continue;
            }
            let list = lists
                .iter()
                .find(|l| l.id == card.id_list)
                .map(|l| l.name.as_str());
            println!(
                "{}",
                serde_json::json!({
                    "id": card.id, "title": card.name, "shortLink": card.short_link, "list": list,
                    "attachmentCount": card.attachments.len(), "supportedUploads": uploads.len(), "videos": videos,
                    "attachments": uploads.iter().map(|a| serde_json::json!({"id": a.id, "name": a.name, "mime": a.mime_type, "bytes": a.bytes})).collect::<Vec<_>>()
                })
            );
        }
    }
    #[test]
    #[ignore = "download remoto opt-in; cofre existente, armazenamento temporário, sem alterar banco operacional"]
    fn verify_authorized_attachment_download() {
        use crate::trello_vault::{Vault, WindowsVault};
        assert_eq!(std::env::var("GMAD_LIVE_TRELLO_DIAG").as_deref(), Ok("1"));
        let card_id = std::env::var("GMAD_LIVE_TRELLO_CARD_ID").expect("explicit remote ID");
        let attachment_id =
            std::env::var("GMAD_LIVE_TRELLO_ATTACHMENT_ID").expect("explicit attachment ID");
        assert!(remote_id(&card_id) && remote_id(&attachment_id));
        let credentials = WindowsVault
            .load()
            .expect("vault read")
            .expect("configured credential");
        let mut api = OfficialTransport::new(&credentials, 120).expect("transport");
        let attachment: Attachment = json(
            &mut api,
            &format!("cards/{card_id}/attachments/{attachment_id}"),
            &[],
            &AtomicBool::new(false),
        )
        .expect("attachment metadata");
        assert_eq!(attachment.id, attachment_id);
        let dir = tempfile::tempdir().unwrap();
        let mut store = crate::media_store::MediaStore::open(dir.path()).unwrap();
        let media = crate::trello_service::download_asset(
            &mut api,
            &mut store,
            &card_id,
            &attachment,
            &DownloadLimits::default(),
            &AtomicBool::new(false),
            &mut |_| {},
        )
        .expect("validated download");
        assert_eq!(media.size_bytes, attachment.bytes);
        assert_eq!(
            crate::media_store::checksum(&store.path(&media.storage_path).unwrap()).unwrap(),
            media.checksum
        );
        println!(
            "download_validated bytes={} mime={} dimensions={:?}x{:?} sha256={}",
            media.size_bytes, media.mime_type, media.width, media.height, media.checksum
        );
    }
    #[test]
    fn credential_header_is_sensitive_and_never_a_query_parameter() {
        let c = Credentials {
            key: "a".repeat(32),
            token: "b".repeat(64),
        };
        let header = c.header().unwrap();
        assert!(header.is_sensitive());
        assert_eq!(
            header.to_str().unwrap(),
            format!(
                "OAuth oauth_consumer_key=\"{}\", oauth_token=\"{}\"",
                c.key, c.token
            )
        );
        let client = Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .unwrap();
        let request=client.get("https://api.trello.com/1/cards/aaaaaaaaaaaaaaaaaaaaaaaa/attachments/bbbbbbbbbbbbbbbbbbbbbbbb/download/image.png").header(AUTHORIZATION,header).build().unwrap();
        assert!(request.url().query().is_none());
        assert!(request.headers().get(AUTHORIZATION).unwrap().is_sensitive());
    }
}
