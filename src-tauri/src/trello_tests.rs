use crate::{
    editorial_domain::{PublicationFormat, SavePublication},
    editorial_repository::EditorialRepository,
    media_store::MediaStore,
    trello_api::*,
    trello_domain::*,
    trello_service::{download_asset, import_cards, ImportInput},
};
use serde_json::json;
use std::{
    collections::VecDeque,
    io::Cursor,
    sync::atomic::{AtomicBool, Ordering},
};
const BID: &str = "aaaaaaaaaaaaaaaaaaaaaaaa";
const LID: &str = "bbbbbbbbbbbbbbbbbbbbbbbb";
fn board() -> Board {
    Board {
        id: BID.into(),
        name: "Quadro de teste".into(),
        short_link: "Ab12Cd34".into(),
    }
}
fn card(n: u32) -> RemoteCard {
    serde_json::from_value(json!({"id":format!("{n:024x}"),"name":format!("Campanha {n}"),"desc":"Descrição original 😊\n#GMAD","idList":LID,"shortLink":"Cd12Ab34","closed":false,"pos":n,"dateLastActivity":"2026-10-14T12:00:00Z","due":"2026-11-01T12:00:00Z","start":null,"labels":[],"attachments":[]})).unwrap()
}
fn attachment(n: u32, name: &str) -> Attachment {
    Attachment {
        id: format!("{:024x}", 100000 + n),
        name: name.into(),
        url: "https://api.trello.com/file?token=must-not-persist#secret".into(),
        mime_type: None,
        bytes: 0,
        is_upload: true,
        pos: f64::from(n),
    }
}
fn response(status: u16, data: Vec<u8>) -> HttpResponse {
    HttpResponse {
        status,
        length: Some(data.len() as u64),
        retry_after: None,
        body: Box::new(Cursor::new(data)),
    }
}
struct Mock {
    responses: VecDeque<HttpResponse>,
    calls: Vec<(String, Vec<(String, String)>)>,
}
impl Mock {
    fn new(r: Vec<HttpResponse>) -> Self {
        Self {
            responses: r.into(),
            calls: vec![],
        }
    }
}
impl Transport for Mock {
    fn get(
        &mut self,
        p: &str,
        q: &[(String, String)],
    ) -> Result<HttpResponse, crate::error::AppError> {
        self.calls.push((p.into(), q.to_vec()));
        Ok(self.responses.pop_front().expect("unexpected request"))
    }
    fn pause(&self, _: u64) {}
}
fn png(color: u8) -> Vec<u8> {
    let mut c = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgb8(image::RgbImage::from_pixel(
        12,
        16,
        image::Rgb([color, 10, 20]),
    ))
    .write_to(&mut c, image::ImageFormat::Png)
    .unwrap();
    c.into_inner()
}
fn preview(r: &EditorialRepository, mut cards: Vec<RemoteCard>) -> BoardPreview {
    for c in &mut cards {
        c.sanitize().unwrap();
    }
    r.trello_preview(
        board(),
        vec![TrelloList {
            id: LID.into(),
            name: "Campanhas".into(),
            closed: false,
        }],
        cards,
    )
    .unwrap()
}
fn run(r: &mut EditorialRepository, p: &BoardPreview, t: &mut Mock, cancel: &AtomicBool) -> Job {
    let mut j = r.trello_begin(&p.board, p.cards.len() as u32).unwrap();
    let data = r.storage.directory.parent().unwrap().to_path_buf();
    let selected = p
        .cards
        .iter()
        .map(|c| c.card.id.clone())
        .collect::<Vec<_>>();
    let result = import_cards(
        t,
        ImportInput {
            data: &data,
            preview: p,
            selected: &selected,
            limits: &DownloadLimits::default(),
            cancel,
        },
        &mut j,
        |op| op(r),
        &mut |_| {},
    );
    j.status = match result {
        Ok(()) => {
            if j.attachment_failures > 0 || j.failed_cards > 0 {
                "partial"
            } else {
                "completed"
            }
        }
        Err(e) => {
            if e.code == "TRELLO_CANCELLED" {
                "cancelled"
            } else {
                "failed"
            }
        }
    }
    .into();
    r.trello_job_save(&j).unwrap();
    j
}
fn publication(r: &EditorialRepository, id: &str) -> String {
    r.database
        .connection
        .query_row(
            "SELECT publication_id FROM imported_cards WHERE remote_id=?1",
            [id],
            |r| r.get(0),
        )
        .unwrap()
}
#[test]
fn urls_accept_official_variants_and_extract_short_link() {
    for u in [
        "https://trello.com/b/Ab12Cd34",
        "https://www.trello.com/b/Ab12Cd34/month/?utm=test#x",
        " https://trello.com/b/Ab12Cd34/month ",
    ] {
        assert_eq!(board_short_link(u).unwrap(), "Ab12Cd34");
    }
}
#[test]
fn urls_reject_ssrf_and_confused_authorities() {
    for u in [
        "http://trello.com/b/Ab12Cd34",
        "https://trello.com.evil/b/Ab12Cd34",
        "https://evil@trello.com/b/Ab12Cd34",
        "https://127.0.0.1/b/Ab12Cd34",
        "https://trello.com:444/b/Ab12Cd34",
        "https://trello.com/c/Ab12Cd34",
        "https://trello.com/b/%2fsecret",
        "https://trello.com/b/Ab12Cd34/a/b",
    ] {
        assert_eq!(board_short_link(u).unwrap_err().code, "TRELLO_URL");
    }
}
#[test]
fn auth_and_access_failures_are_redacted() {
    for (s, code) in [
        (401, "TRELLO_AUTH"),
        (403, "TRELLO_ACCESS"),
        (404, "TRELLO_ACCESS"),
    ] {
        let mut t = Mock::new(vec![response(s, b"token=secret upstream body".to_vec())]);
        let e = account(&mut t).err().unwrap();
        assert_eq!(e.code, code);
        assert!(!e.message.contains("secret"));
    }
}
#[test]
fn rate_limit_and_server_failures_have_bounded_retry() {
    let mut t = Mock::new(vec![
        response(429, vec![]),
        response(500, vec![]),
        response(429, vec![]),
    ]);
    assert_eq!(
        request(&mut t, "members/me", &[], &AtomicBool::new(false))
            .err()
            .unwrap()
            .code,
        "TRELLO_RATE"
    );
    assert_eq!(t.calls.len(), 3);
}
#[test]
fn retries_can_recover() {
    let mut t = Mock::new(vec![response(503, vec![]), response(200, b"ok".to_vec())]);
    assert_eq!(
        request(&mut t, "x", &[], &AtomicBool::new(false))
            .unwrap()
            .status,
        200
    );
    assert_eq!(t.calls.len(), 2);
}
#[test]
fn pagination_reads_all_lists_cards_and_dates() {
    let cards = (1001..=2000).rev().map(card).collect::<Vec<_>>();
    let mut t = Mock::new(vec![
        response(200, serde_json::to_vec(&board()).unwrap()),
        response(
            200,
            serde_json::to_vec(&vec![TrelloList {
                id: LID.into(),
                name: "Lista".into(),
                closed: false,
            }])
            .unwrap(),
        ),
        response(200, serde_json::to_vec(&cards).unwrap()),
        response(200, serde_json::to_vec(&vec![card(1000)]).unwrap()),
    ]);
    let (_, lists, cards) = read_board(&mut t, "Ab12Cd34", &AtomicBool::new(false)).unwrap();
    assert_eq!(lists.len(), 1);
    assert_eq!(cards.len(), 1001);
    assert!(t.calls[3]
        .1
        .contains(&("before".into(), format!("{:024x}", 1001))));
    assert_eq!(cards[0].desc, "Descrição original 😊\n#GMAD");
    assert!(cards[0].due.is_some());
}
#[test]
fn duplicate_page_is_rejected_instead_of_claiming_full_snapshot() {
    let cards = (1001..=2000).map(card).collect::<Vec<_>>();
    let page = serde_json::to_vec(&cards).unwrap();
    let mut t = Mock::new(vec![
        response(200, serde_json::to_vec(&board()).unwrap()),
        response(200, b"[]".to_vec()),
        response(200, page.clone()),
        response(200, page),
    ]);
    assert!(read_board(&mut t, "Ab12Cd34", &AtomicBool::new(false)).is_err());
}
#[test]
fn image_download_and_local_import_are_real() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    let mut c = card(1);
    c.attachments.push(attachment(1, "imagem.png"));
    let p = preview(&r, vec![c]);
    assert_eq!(r.counts().unwrap().total, 0);
    let mut t = Mock::new(vec![response(200, png(1))]);
    let j = run(&mut r, &p, &mut t, &AtomicBool::new(false));
    assert_eq!(j.new_publications, 1);
    let post = r.get(&publication(&r, &card(1).id)).unwrap();
    assert_eq!(post.source, "trello");
    assert_eq!(post.editorial_status, "pending_review");
    assert_eq!(post.media[0].mime_type, "image/png");
    assert_eq!(post.format, "feed");
    assert!(r.storage.path(&post.media[0].storage_path).is_ok());
    assert!(t.calls[0]
        .0
        .starts_with("cards/000000000000000000000001/attachments/"));
    assert!(t.calls[0].1.is_empty());
}
#[test]
fn video_import_uses_bounded_stream_and_reels_suggestion() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    let mut c = card(2);
    c.attachments.push(attachment(2, "video.mp4"));
    let p = preview(&r, vec![c]);
    let mut t = Mock::new(vec![response(
        200,
        include_bytes!("../../tests/fixtures/minimal.mp4").to_vec(),
    )]);
    assert_eq!(
        run(&mut r, &p, &mut t, &AtomicBool::new(false)).downloaded_attachments,
        1
    );
    let post = r.get(&publication(&r, &card(2).id)).unwrap();
    assert_eq!(post.format, "reels");
    assert_eq!(post.media[0].mime_type, "video/mp4");
    assert!(post.media[0].duration.is_some());
}
#[test]
fn multiple_images_keep_order_and_sha_dedup() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    let mut c = card(3);
    c.attachments = vec![attachment(3, "first.png"), attachment(4, "second.png")];
    let p = preview(&r, vec![c.clone()]);
    let mut t = Mock::new(vec![response(200, png(3)), response(200, png(4))]);
    run(&mut r, &p, &mut t, &AtomicBool::new(false));
    let post = r.get(&publication(&r, &c.id)).unwrap();
    assert_eq!(post.format, "carousel");
    assert_eq!(post.media[0].filename, "first.png");
    assert_eq!(post.media[1].filename, "second.png");
    let j = run(&mut r, &p, &mut Mock::new(vec![]), &AtomicBool::new(false));
    assert_eq!(j.new_publications, 0);
    assert_eq!(j.downloaded_attachments, 0);
    assert_eq!(r.counts().unwrap().total, 1);
}
#[test]
fn text_only_import_remains_editable_without_schedule() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    let c = card(4);
    let p = preview(&r, vec![c.clone()]);
    run(&mut r, &p, &mut Mock::new(vec![]), &AtomicBool::new(false));
    let id = publication(&r, &c.id);
    let post = r.get(&id).unwrap();
    assert!(post.media.is_empty());
    assert!(r.trello_source(&id).unwrap().unwrap().needs_adjustment);
    let saved = r
        .save(SavePublication {
            id: Some(id),
            expected_version: Some(1),
            title: "Título local".into(),
            caption: "Legenda local".into(),
            format: PublicationFormat::Stories,
            media_ids: vec![],
        })
        .unwrap();
    assert_eq!(saved.version, 2);
    assert_eq!(saved.editorial_status, "pending_review");
}
#[test]
fn external_links_are_references_never_downloaded() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    let mut c = card(5);
    let mut a = attachment(5, "reference.png");
    a.is_upload = false;
    a.url = "https://example.org/campaign?token=secret#signed".into();
    c.attachments.push(a);
    let p = preview(&r, vec![c.clone()]);
    let mut t = Mock::new(vec![]);
    run(&mut r, &p, &mut t, &AtomicBool::new(false));
    let s = r.trello_source(&publication(&r, &c.id)).unwrap().unwrap();
    assert_eq!(s.attachments[0].status, "reference");
    assert_eq!(s.attachments[0].url, "https://example.org/campaign");
    assert!(t.calls.is_empty());
}
#[test]
fn invalid_file_partial_import_and_retry_preserve_success() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    let mut c = card(6);
    c.attachments = vec![attachment(6, "ok.png"), attachment(7, "retry.png")];
    let p = preview(&r, vec![c.clone()]);
    let j = run(
        &mut r,
        &p,
        &mut Mock::new(vec![
            response(200, png(6)),
            response(200, b"invalid".to_vec()),
        ]),
        &AtomicBool::new(false),
    );
    assert_eq!(j.status, "partial");
    assert_eq!(j.attachment_failures, 1);
    assert!(r.trello_history().unwrap()[0].last_synced_at.is_none());
    let id = publication(&r, &c.id);
    let media = r.get(&id).unwrap().media;
    let j = run(
        &mut r,
        &p,
        &mut Mock::new(vec![response(200, png(7))]),
        &AtomicBool::new(false),
    );
    assert_eq!(j.status, "completed");
    assert_eq!(j.new_publications, 0);
    assert_eq!(r.get(&id).unwrap().media[0].id, media[0].id);
    assert_eq!(r.get(&id).unwrap().media.len(), 1);
    assert_eq!(
        r.trello_source(&id)
            .unwrap()
            .unwrap()
            .attachments
            .iter()
            .filter(|a| a.media.is_some())
            .count(),
        2
    );
}
#[test]
fn sync_preserves_title_caption_format_status_and_manual_media() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    let mut c = card(7);
    c.attachments.push(attachment(8, "image.png"));
    let p = preview(&r, vec![c.clone()]);
    run(
        &mut r,
        &p,
        &mut Mock::new(vec![response(200, png(8))]),
        &AtomicBool::new(false),
    );
    let id = publication(&r, &c.id);
    let path = dir.path().join("manual.png");
    std::fs::write(&path, png(9)).unwrap();
    let m = r.storage.stage(&path).unwrap();
    r.save(SavePublication {
        id: Some(id.clone()),
        expected_version: Some(1),
        title: "Meu título".into(),
        caption: "Minha legenda".into(),
        format: PublicationFormat::Stories,
        media_ids: vec![m.id],
    })
    .unwrap();
    r.database
        .connection
        .execute(
            "UPDATE publications SET editorial_status='ignored' WHERE id=?1",
            [&id],
        )
        .unwrap();
    c.name = "Novo título remoto".into();
    c.desc = "Nova descrição".into();
    c.attachments.push(attachment(9, "new.png"));
    let p = preview(&r, vec![c]);
    assert_eq!(p.cards[0].state, "changed");
    let j = run(
        &mut r,
        &p,
        &mut Mock::new(vec![response(200, png(10))]),
        &AtomicBool::new(false),
    );
    assert_eq!(j.changed_sources, 1);
    let post = r.get(&id).unwrap();
    assert_eq!(post.title, "Meu título");
    assert_eq!(post.caption, "Minha legenda");
    assert_eq!(post.format, "stories");
    assert_eq!(post.editorial_status, "ignored");
    assert_eq!(post.version, 2);
    assert_eq!(post.media[0].filename, "manual.png");
    let source = r.trello_source(&id).unwrap().unwrap();
    assert!(source.source_changed);
    assert_eq!(source.original_title, "Novo título remoto");
}
#[test]
fn removed_attachments_and_cards_never_delete_local_media() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    let mut c = card(8);
    c.attachments.push(attachment(10, "image.png"));
    let p = preview(&r, vec![c.clone()]);
    run(
        &mut r,
        &p,
        &mut Mock::new(vec![response(200, png(10))]),
        &AtomicBool::new(false),
    );
    let id = publication(&r, &c.id);
    c.attachments.clear();
    let p = preview(&r, vec![c]);
    run(&mut r, &p, &mut Mock::new(vec![]), &AtomicBool::new(false));
    assert!(r.trello_source(&id).unwrap().unwrap().attachments[0].removed);
    r.trello_finish_snapshot(&board(), &[], true).unwrap();
    assert!(r.trello_source(&id).unwrap().unwrap().source_missing);
    assert_eq!(r.get(&id).unwrap().media.len(), 1);
    assert!(r
        .storage
        .path(&r.get(&id).unwrap().media[0].storage_path)
        .is_ok());
}
#[test]
fn archived_cards_are_marked_unavailable_without_deleting() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    let mut c = card(9);
    let p = preview(&r, vec![c.clone()]);
    run(&mut r, &p, &mut Mock::new(vec![]), &AtomicBool::new(false));
    c.closed = true;
    let p = preview(&r, vec![c.clone()]);
    assert_eq!(p.cards[0].state, "archived");
    r.trello_finish_snapshot(&board(), &p.cards, true).unwrap();
    assert!(
        r.trello_source(&publication(&r, &c.id))
            .unwrap()
            .unwrap()
            .source_missing
    );
    assert_eq!(r.counts().unwrap().total, 1);
}
#[test]
fn interrupted_stream_is_failure_and_cleans_partial() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = MediaStore::open(dir.path()).unwrap();
    let mut t = Mock::new(vec![HttpResponse {
        status: 200,
        length: Some(100),
        retry_after: None,
        body: Box::new(Cursor::new(vec![1, 2, 3])),
    }]);
    assert!(download_asset(
        &mut t,
        &mut s,
        &card(1).id,
        &attachment(1, "x.png"),
        &DownloadLimits::default(),
        &AtomicBool::new(false),
        &mut |_| {}
    )
    .is_err());
    assert_eq!(std::fs::read_dir(&s.staging_directory).unwrap().count(), 0);
}
#[test]
fn cancellation_during_stream_cleans_partial() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = MediaStore::open(dir.path()).unwrap();
    let flag = AtomicBool::new(false);
    let mut t = Mock::new(vec![response(200, vec![0; 128 * 1024])]);
    let e = download_asset(
        &mut t,
        &mut s,
        &card(1).id,
        &attachment(1, "x.png"),
        &DownloadLimits::default(),
        &flag,
        &mut |_| flag.store(true, Ordering::Relaxed),
    )
    .err()
    .unwrap();
    assert_eq!(e.code, "TRELLO_CANCELLED");
    assert_eq!(std::fs::read_dir(&s.staging_directory).unwrap().count(), 0);
}
#[test]
fn cancellation_keeps_completed_records() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    let p = preview(&r, vec![card(1), card(2)]);
    let mut j = r.trello_begin(&board(), 2).unwrap();
    let flag = AtomicBool::new(false);
    let selected = p
        .cards
        .iter()
        .map(|c| c.card.id.clone())
        .collect::<Vec<_>>();
    let result = import_cards(
        &mut Mock::new(vec![]),
        ImportInput {
            data: dir.path(),
            preview: &p,
            selected: &selected,
            limits: &DownloadLimits::default(),
            cancel: &flag,
        },
        &mut j,
        |op| op(&mut r),
        &mut |j| {
            if j.processed_cards == 1 {
                flag.store(true, Ordering::Relaxed)
            }
        },
    );
    assert_eq!(result.unwrap_err().code, "TRELLO_CANCELLED");
    assert_eq!(r.counts().unwrap().total, 1);
    assert_eq!(j.processed_cards, 1);
}
#[test]
fn redirects_and_size_limits_do_not_follow_unknown_hosts() {
    let mut t = Mock::new(vec![response(
        302,
        b"https://unknown.example/secret".to_vec(),
    )]);
    assert_eq!(
        request(&mut t, "x", &[], &AtomicBool::new(false))
            .err()
            .unwrap()
            .code,
        "TRELLO_REDIRECT"
    );
    let mut bytes = vec![];
    assert_eq!(
        copy_bounded(
            &mut Cursor::new(vec![0; 20]),
            &mut bytes,
            10,
            Some(20),
            &AtomicBool::new(false),
            &mut |_| {}
        )
        .unwrap_err()
        .code,
        "TRELLO_LIMIT"
    );
    assert!(bytes.is_empty());
}
#[test]
fn restart_preserves_sqlite_integrity_and_recovers_job() {
    let dir = tempfile::tempdir().unwrap();
    {
        let mut r = EditorialRepository::open(dir.path()).unwrap();
        let p = preview(&r, vec![card(1)]);
        run(&mut r, &p, &mut Mock::new(vec![]), &AtomicBool::new(false));
        r.trello_begin(&board(), 2).unwrap();
    }
    let r = EditorialRepository::open(dir.path()).unwrap();
    r.trello_recover().unwrap();
    assert_eq!(r.counts().unwrap().total, 1);
    assert!(r
        .trello_jobs()
        .unwrap()
        .iter()
        .any(|j| j.status == "interrupted"));
    assert_eq!(r.database.status().unwrap().integrity, "ok");
    assert_eq!(
        r.database
            .connection
            .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| r
                .get::<_, u32>(
                0
            ))
            .unwrap(),
        0
    );
}
#[test]
fn credentials_and_authenticated_urls_never_enter_sqlite() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    let mut c = card(1);
    let mut a = attachment(1, "link");
    a.is_upload = false;
    c.attachments.push(a);
    let p = preview(&r, vec![c]);
    run(&mut r, &p, &mut Mock::new(vec![]), &AtomicBool::new(false));
    drop(r);
    let b = std::fs::read(dir.path().join("planner.sqlite3")).unwrap();
    let text = String::from_utf8_lossy(&b);
    assert!(!text.contains("must-not-persist"));
    assert!(!text.contains("?token="));
    let credentials = Credentials {
        key: "injected\"\r\n".into(),
        token: "secret".into(),
    };
    assert_eq!(credentials.validate().unwrap_err().code, "TRELLO_AUTH");
}
#[test]
fn manual_publications_remain_independent_and_unchanged() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    let path = dir.path().join("manual.png");
    std::fs::write(&path, png(8)).unwrap();
    let m = r.storage.stage(&path).unwrap();
    let post = r
        .save(SavePublication {
            id: None,
            expected_version: None,
            title: "Manual".into(),
            caption: "Minha legenda".into(),
            format: PublicationFormat::Feed,
            media_ids: vec![m.id],
        })
        .unwrap();
    let p = preview(&r, vec![card(1)]);
    run(&mut r, &p, &mut Mock::new(vec![]), &AtomicBool::new(false));
    assert_eq!(r.get(&post.id).unwrap().caption, "Minha legenda");
    assert_eq!(r.get(&post.id).unwrap().source, "manual");
    assert!(r.trello_source(&post.id).unwrap().is_none());
    assert_eq!(r.counts().unwrap().drafts, 1);
    assert_eq!(r.counts().unwrap().pending_review, 1);
}
#[test]
fn invalid_limits_are_rejected() {
    for limits in [
        DownloadLimits {
            image_mib: 65,
            ..Default::default()
        },
        DownloadLimits {
            video_mib: 0,
            ..Default::default()
        },
        DownloadLimits {
            timeout_seconds: 301,
            ..Default::default()
        },
    ] {
        assert!(limits.validate().is_err());
    }
}
#[test]
fn duplicate_rejects_missing_media_without_creating_another_publication() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    let mut c = card(13);
    c.attachments.push(attachment(13, "campanha.png"));
    let p = preview(&r, vec![c.clone()]);
    run(
        &mut r,
        &p,
        &mut Mock::new(vec![response(200, png(13))]),
        &AtomicBool::new(false),
    );
    let id = publication(&r, &c.id);
    let post = r.get(&id).unwrap();
    let path = r.storage.path(&post.media[0].storage_path).unwrap();
    std::fs::remove_file(path).unwrap();
    assert!(r.duplicate(&id).is_err());
    assert_eq!(r.counts().unwrap().total, 1);
    assert_eq!(publication(&r, &c.id), id);
}
#[test]
fn changed_file_before_commit_cannot_create_broken_publication() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    let mut c = card(12);
    let a = attachment(12, "campanha.png");
    c.attachments.push(a.clone());
    c.sanitize().unwrap();
    r.trello_begin(&board(), 1).unwrap();
    let mut storage = MediaStore::open(dir.path()).unwrap();
    let media = download_asset(
        &mut Mock::new(vec![response(200, png(42))]),
        &mut storage,
        &c.id,
        &a,
        &DownloadLimits::default(),
        &AtomicBool::new(false),
        &mut |_| {},
    )
    .unwrap();
    std::fs::write(storage.path(&media.storage_path).unwrap(), b"changed").unwrap();
    let result = crate::trello_repository::AttachmentResult {
        attachment: a,
        media: Some(media),
        status: "available".into(),
        error_message: None,
    };
    assert!(r
        .trello_commit_card(&board(), &c, "Campanhas", &[result])
        .is_err());
    assert_eq!(r.counts().unwrap().total, 0);
    let origins: u32 = r
        .database
        .connection
        .query_row("SELECT count(*) FROM imported_cards", [], |row| row.get(0))
        .unwrap();
    assert_eq!(origins, 0);
}
#[test]
fn metadata_and_deleted_publication_survive_reimport() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    let c = card(11);
    let p = preview(&r, vec![c.clone()]);
    run(&mut r, &p, &mut Mock::new(vec![]), &AtomicBool::new(false));
    let id = publication(&r, &c.id);
    r.delete(&id, 1).unwrap();
    let j = run(&mut r, &p, &mut Mock::new(vec![]), &AtomicBool::new(false));
    assert_eq!(j.new_publications, 0);
    assert_eq!(r.counts().unwrap().total, 0);
    assert_eq!(publication(&r, &c.id), id);
}
#[test]
#[ignore = "prepara somente um perfil explícito em .tmp para inspeção nativa; sem serviço Trello"]
fn prepare_native_qa_fixture() {
    let root = std::path::PathBuf::from(
        std::env::var_os("GMAD_TRELLO_QA_ROOT").expect("explicit QA directory"),
    );
    let workspace = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(".tmp");
    assert!(root.is_absolute() && root.starts_with(&workspace));
    assert!(
        !root.join("planner.sqlite3").exists(),
        "preserve existing QA data"
    );
    std::fs::create_dir_all(&root).unwrap();
    let mut r = EditorialRepository::open(&root).unwrap();
    let mut image = card(1);
    image.name = "QA Trello — Imagem".into();
    image.attachments.push(attachment(1, "campanha.png"));
    let mut video = card(2);
    video.name = "QA Trello — Vídeo".into();
    video.attachments.push(attachment(2, "campanha.mp4"));
    let mut text = card(3);
    text.name = "QA Trello — Texto sem mídia".into();
    let p = preview(&r, vec![image, video, text]);
    let mut t = Mock::new(vec![
        response(200, png(70)),
        response(
            200,
            include_bytes!("../../tests/fixtures/minimal.mp4").to_vec(),
        ),
    ]);
    let j = run(&mut r, &p, &mut t, &AtomicBool::new(false));
    assert_eq!(j.status, "completed");
    assert_eq!(r.counts().unwrap().pending_review, 3);
    assert_eq!(r.database.status().unwrap().schema_version, 3);
}
#[test]
#[ignore = "teste Windows opt-in, usa e remove somente entrada exclusiva de QA"]
fn windows_vault_roundtrip_isolated() {
    let entry = keyring::Entry::new(
        "br.com.gmad.socialplanner.qa",
        &uuid::Uuid::new_v4().to_string(),
    )
    .unwrap();
    entry
        .set_password("isolated-vault-test-not-a-trello-credential")
        .unwrap();
    let value = entry.get_password().unwrap();
    entry.delete_credential().unwrap();
    assert_eq!(value, "isolated-vault-test-not-a-trello-credential");
    assert!(matches!(entry.get_password(), Err(keyring::Error::NoEntry)));
}
