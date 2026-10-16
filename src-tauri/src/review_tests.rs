use crate::{editorial_domain::*, editorial_repository::EditorialRepository, review::*};
use std::{fs, path::Path};
fn post(r: &mut EditorialRepository, dir: &Path, color: u8) -> Publication {
    let file = dir.join(format!("source-{color}.png"));
    image::RgbImage::from_pixel(20, 30, image::Rgb([color, 100, 50]))
        .save(&file)
        .unwrap();
    let m = r.storage.stage(&file).unwrap();
    r.save(SavePublication {
        id: None,
        expected_version: None,
        title: format!("Campanha ação {color}"),
        caption: "**Legenda 😊**\n#GMAD".into(),
        format: PublicationFormat::Feed,
        media_ids: vec![m.id],
    })
    .unwrap()
}
fn decide(r: &mut EditorialRepository, p: &Publication, action: ReviewAction) -> bool {
    r.review_decide(ReviewDecision {
        targets: vec![ReviewTarget {
            id: p.id.clone(),
            expected_version: p.version,
        }],
        action,
        reason: None,
        note: None,
    })
    .unwrap()[0]
        .success
}
fn pending(r: &mut EditorialRepository, dir: &Path, color: u8) -> Publication {
    let p = post(r, dir, color);
    assert!(decide(r, &p, ReviewAction::Submit));
    r.get(&p.id).unwrap()
}
#[test]
fn decisions_are_versioned_reversible_persistent_and_history_is_immutable() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    let p = pending(&mut r, dir.path(), 1);
    assert!(decide(&mut r, &p, ReviewAction::Approve));
    assert!(!decide(&mut r, &p, ReviewAction::Ignore));
    let p = r.get(&p.id).unwrap();
    assert_eq!(p.editorial_status, "approved");
    assert!(decide(&mut r, &p, ReviewAction::Reopen));
    let p = r.get(&p.id).unwrap();
    let result = r
        .review_decide(ReviewDecision {
            targets: vec![ReviewTarget {
                id: p.id.clone(),
                expected_version: p.version,
            }],
            action: ReviewAction::Ignore,
            reason: Some("product".into()),
            note: Some("Loja não trabalha com o produto".into()),
        })
        .unwrap();
    assert!(result[0].success);
    let id = p.id;
    drop(r);
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    let p = r.get(&id).unwrap();
    assert_eq!(p.editorial_status, "ignored");
    assert_eq!(p.media.len(), 1);
    assert!(r.storage.path(&p.media[0].storage_path).is_ok());
    let history = r
        .review_history(HistoryQuery {
            publication_id: Some(id),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(history.total, 4);
    assert_eq!(history.items[0].reason.as_deref(), Some("product"));
    assert_eq!(
        history.items[0].note.as_deref(),
        Some("Loja não trabalha com o produto")
    );
    assert!(r
        .database
        .connection
        .execute("DELETE FROM editorial_reviews", [])
        .is_err());
    assert!(r
        .database
        .connection
        .execute("UPDATE editorial_reviews SET event='edited'", [])
        .is_err());
    assert!(decide(&mut r, &p, ReviewAction::Reopen));
    assert_eq!(r.counts().unwrap().pending_review, 1);
    assert_eq!(r.counts().unwrap().ignored, 0);
    assert_eq!(r.database.status().unwrap().integrity, "ok");
}
#[test]
fn edits_invalidate_approval_preserve_unicode_media_and_record_event() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    let p = pending(&mut r, dir.path(), 2);
    assert!(decide(&mut r, &p, ReviewAction::Approve));
    let p = r.get(&p.id).unwrap();
    let edited = r
        .save(SavePublication {
            id: Some(p.id.clone()),
            expected_version: Some(p.version),
            title: p.title.clone(),
            caption: "Rio Preto 😊\n#GMAD ç ã".into(),
            format: PublicationFormat::Feed,
            media_ids: p.media.iter().map(|m| m.id.clone()).collect(),
        })
        .unwrap();
    assert_eq!(edited.editorial_status, "pending_review");
    assert_eq!(edited.caption, "Rio Preto 😊\n#GMAD ç ã");
    assert_eq!(edited.media[0].id, p.media[0].id);
    assert!(!decide(&mut r, &p, ReviewAction::Approve));
    let h = r.review_history(HistoryQuery::default()).unwrap();
    assert_eq!(h.items[0].event, "edited");
    assert_eq!(h.items[0].previous_status, "approved");
    assert_eq!(h.items[0].resulting_version, edited.version);
}
#[test]
fn batch_reports_partial_results_and_never_changes_unselected_or_invalid_items() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    let a = pending(&mut r, dir.path(), 3);
    let b = pending(&mut r, dir.path(), 4);
    let c = pending(&mut r, dir.path(), 5);
    fs::remove_file(r.storage.path(&b.media[0].storage_path).unwrap()).unwrap();
    let res = r
        .review_decide(ReviewDecision {
            targets: vec![
                ReviewTarget {
                    id: a.id.clone(),
                    expected_version: a.version,
                },
                ReviewTarget {
                    id: b.id.clone(),
                    expected_version: b.version,
                },
            ],
            action: ReviewAction::Approve,
            reason: None,
            note: None,
        })
        .unwrap();
    assert!(res[0].success);
    assert!(!res[1].success);
    assert_eq!(r.get(&b.id).unwrap().version, b.version);
    assert_eq!(r.get(&c.id).unwrap().version, c.version);
    let p = r.get(&a.id).unwrap();
    assert!(decide(&mut r, &p, ReviewAction::Reopen));
    let res = r
        .review_decide(ReviewDecision {
            targets: vec![
                ReviewTarget {
                    id: a.id.clone(),
                    expected_version: p.version + 1,
                },
                ReviewTarget {
                    id: b.id.clone(),
                    expected_version: b.version,
                },
            ],
            action: ReviewAction::Ignore,
            reason: None,
            note: None,
        })
        .unwrap();
    assert!(res.iter().all(|r| r.success));
    assert_eq!(r.counts().unwrap().ignored, 2);
    assert_eq!(r.get(&c.id).unwrap().editorial_status, "pending_review");
}
#[test]
fn missing_tampered_empty_and_incompatible_media_block_approval_but_mixed_is_warning() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    let p = pending(&mut r, dir.path(), 6);
    let path = r.storage.path(&p.media[0].storage_path).unwrap();
    let mut bytes = fs::read(&path).unwrap();
    bytes[10] ^= 1;
    fs::write(&path, &bytes).unwrap();
    assert!(!decide(&mut r, &p, ReviewAction::Approve));
    let p = pending(&mut r, dir.path(), 7);
    r.database
        .connection
        .execute(
            "UPDATE publications SET format='reels' WHERE id=?1",
            [&p.id],
        )
        .unwrap();
    assert!(!decide(&mut r, &p, ReviewAction::Approve));
    r.database
        .connection
        .execute(
            "DELETE FROM publication_media WHERE publication_id=?1",
            [&p.id],
        )
        .unwrap();
    let item = r.review_item(&p.id, true).unwrap();
    assert!(item.issues.iter().any(|i| i.code == "NO_MEDIA"));
    assert!(!decide(&mut r, &p, ReviewAction::Approve));
    assert!(decide(&mut r, &p, ReviewAction::Ignore));
    let video = dir.path().join("video.mp4");
    fs::write(&video, include_bytes!("../../tests/fixtures/minimal.mp4")).unwrap();
    let v = r.storage.stage(&video).unwrap();
    let image = post(&mut r, dir.path(), 8);
    let mixed = r
        .save(SavePublication {
            id: None,
            expected_version: None,
            title: "Misto".into(),
            caption: "".into(),
            format: PublicationFormat::Carousel,
            media_ids: vec![image.media[0].id.clone(), v.id],
        })
        .unwrap();
    assert!(decide(&mut r, &mixed, ReviewAction::Submit));
    let mixed = r.get(&mixed.id).unwrap();
    let item = r.review_item(&mixed.id, true).unwrap();
    assert!(item
        .issues
        .iter()
        .any(|i| i.code == "MIXED" && i.severity == "warning"));
    assert!(!item.issues.iter().any(|i| i.severity == "error"));
    assert!(decide(&mut r, &mixed, ReviewAction::Approve));
}
#[test]
fn queue_search_filters_pagination_history_and_counts_are_real() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    let a = pending(&mut r, dir.path(), 9);
    let b = pending(&mut r, dir.path(), 10);
    assert!(decide(&mut r, &b, ReviewAction::Ignore));
    let q = ReviewQuery {
        search: "ACAO 9".into(),
        status: Some("pending_review".into()),
        source: Some("manual".into()),
        has_media: Some(true),
        ..Default::default()
    };
    let list = r.review_list(q).unwrap();
    assert_eq!(list.total, 1);
    assert_eq!(list.items[0].publication.id, a.id);
    assert_eq!(list.counts.ignored, 1);
    assert_eq!(
        r.review_list(ReviewQuery {
            has_media: Some(false),
            ..Default::default()
        })
        .unwrap()
        .total,
        0
    );
    assert_eq!(
        r.review_list(ReviewQuery {
            offset: 1,
            ..Default::default()
        })
        .unwrap()
        .items
        .len(),
        1
    );
    let h = r
        .review_history(HistoryQuery {
            search: "ação 10".into(),
            event: Some("ignored".into()),
            ..Default::default()
        })
        .unwrap();
    assert_eq!(h.total, 1);
}
#[test]
fn invalid_requests_and_audit_failure_cannot_mutate_publication() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    let p = pending(&mut r, dir.path(), 11);
    assert!(r
        .review_decide(ReviewDecision {
            targets: vec![],
            action: ReviewAction::Approve,
            reason: None,
            note: None
        })
        .is_err());
    r.database.connection.execute_batch("CREATE TRIGGER fail_review BEFORE INSERT ON editorial_reviews BEGIN SELECT RAISE(ABORT,'test'); END;").unwrap();
    assert!(!decide(&mut r, &p, ReviewAction::Approve));
    assert_eq!(r.get(&p.id).unwrap().version, p.version);
    assert_eq!(r.get(&p.id).unwrap().editorial_status, "pending_review");
}
#[test]
#[ignore = "prepara apenas perfil QA novo em .tmp; nunca usa o banco operacional"]
fn prepare_review_qa_fixture() {
    let path =
        std::path::PathBuf::from(std::env::var_os("GMAD_QA_DATA_DIR").expect("explicit QA path"));
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(".tmp");
    assert!(path.is_absolute() && path.starts_with(root));
    assert!(!path.exists());
    fs::create_dir_all(&path).unwrap();
    let mut r = EditorialRepository::open(&path).unwrap();
    for n in 20..24 {
        pending(&mut r, &path, n);
    }
    let a = post(&mut r, &path, 30);
    let b = post(&mut r, &path, 31);
    let carousel = r
        .save(SavePublication {
            id: None,
            expected_version: None,
            title: "QA — Carrossel".into(),
            caption: "**Carrossel 😊**\n#GMAD".into(),
            format: PublicationFormat::Carousel,
            media_ids: vec![a.media[0].id.clone(), b.media[0].id.clone()],
        })
        .unwrap();
    assert!(decide(&mut r, &carousel, ReviewAction::Submit));
    let video = path.join("fixture.mp4");
    fs::write(&video, include_bytes!("../../tests/fixtures/minimal.mp4")).unwrap();
    let v = r.storage.stage(&video).unwrap();
    let p = r
        .save(SavePublication {
            id: None,
            expected_version: None,
            title: "QA — Vídeo".into(),
            caption: "Vídeo fixture".into(),
            format: PublicationFormat::Reels,
            media_ids: vec![v.id],
        })
        .unwrap();
    assert!(decide(&mut r, &p, ReviewAction::Submit));
    r.database.connection.execute("INSERT INTO publications(id,title,caption,search_text,format,source,editorial_status) VALUES(?1,'QA — Sem mídia','Preservar','qa sem midia','feed','trello','pending_review')",[uuid::Uuid::new_v4().to_string()]).unwrap();
    println!("QA_PROFILE_READY");
}
