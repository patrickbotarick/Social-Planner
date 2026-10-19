use crate::{
    assisted::*, assisted_files::*, calendar::*, editorial_domain::*,
    editorial_repository::EditorialRepository, review::*,
};
use std::{fs, path::Path};
fn publication(
    r: &mut EditorialRepository,
    root: &Path,
    n: u8,
    format: PublicationFormat,
) -> Publication {
    let mut media = Vec::new();
    let count = if matches!(format, PublicationFormat::Carousel) {
        3
    } else {
        1
    };
    for index in 0..count {
        let path = if matches!(format, PublicationFormat::Reels) {
            Path::new(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/minimal.mp4")
        } else {
            let path = root.join(format!("{n}-{index}.png"));
            image::RgbImage::from_pixel(24, 32, image::Rgb([n, index, 70]))
                .save(&path)
                .unwrap();
            path
        };
        media.push(r.storage.stage(&path).unwrap().id);
    }
    let p = r
        .save(SavePublication {
            id: None,
            expected_version: None,
            title: "CON: campanha / exemplo? *".into(),
            caption: "GMAD 💚\nRio Preto\n#Madeira".into(),
            format,
            media_ids: media,
        })
        .unwrap();
    for action in [ReviewAction::Submit, ReviewAction::Approve] {
        let current = r.get(&p.id).unwrap();
        assert!(
            r.review_decide(ReviewDecision {
                targets: vec![ReviewTarget {
                    id: p.id.clone(),
                    expected_version: current.version
                }],
                action,
                reason: None,
                note: None
            })
            .unwrap()[0]
                .success
        )
    }
    r.get(&p.id).unwrap()
}
fn planned(r: &mut EditorialRepository, p: &Publication, date: &str) -> String {
    r.schedule_save(ScheduleInput {
        id: None,
        expected_version: None,
        publication_id: p.id.clone(),
        publication_version: p.version,
        scheduled_date: date.into(),
        scheduled_time: "10:00".into(),
        timezone: "America/Sao_Paulo".into(),
    })
    .unwrap()
}
fn target(p: &Publication, schedule: &str) -> PreparationTarget {
    PreparationTarget {
        schedule_id: schedule.into(),
        schedule_version: 1,
        publication_version: p.version,
    }
}
fn query() -> AssistedQuery {
    AssistedQuery {
        calendar: CalendarQuery {
            year: 2099,
            month: 11,
            timezone: "America/Sao_Paulo".into(),
            ..Default::default()
        },
        state: None,
        offset: 0,
    }
}
fn root(r: &EditorialRepository, path: &Path) {
    fs::create_dir(path).unwrap();
    r.assisted_choose_root(path).unwrap();
}
fn prepared(r: &mut EditorialRepository, p: &Publication, schedule: &str) -> String {
    let result = r.assisted_prepare(vec![target(p, schedule)]).unwrap();
    assert!(result[0].error.is_none(), "{:?}", result[0].error);
    result[0].preparation_id.clone().unwrap()
}
fn confirm(prep: &str) -> ConfirmationInput {
    ConfirmationInput {
        preparation_id: prep.into(),
        previous_id: None,
        expected_version: None,
        actual_date: "2099-11-03".into(),
        actual_time: "10:00".into(),
        timezone: "America/Sao_Paulo".into(),
        reference: "QA local, sem agendamento real".into(),
        note: "Somente declaração de teste 💚".into(),
        verified_by_operator: true,
    }
}
#[test]
fn exports_utf8_original_bytes_manifest_and_idempotent_selection() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    let p = publication(&mut r, dir.path(), 1, PublicationFormat::Feed);
    let s = planned(&mut r, &p, "2099-11-03");
    root(&r, &dir.path().join("exports"));
    let prep = prepared(&mut r, &p, &s);
    let folder = r.assisted_folder(&prep).unwrap();
    let path = Path::new(&folder);
    assert_eq!(
        fs::read(path.join("legenda.txt")).unwrap(),
        p.caption.as_bytes()
    );
    let m: serde_json::Value =
        serde_json::from_slice(&fs::read(path.join("informacoes.json")).unwrap()).unwrap();
    assert_eq!(m["publicationId"], p.id);
    assert_eq!(m["timezone"], "America/Sao_Paulo");
    assert!(m.get("access_token").is_none());
    assert_eq!(
        file_digest(&path.join("media_01.png")).unwrap(),
        p.media[0].checksum
    );
    assert_eq!(r.assisted_caption(&prep).unwrap(), p.caption);
    assert_eq!(prepared(&mut r, &p, &s), prep);
    assert_eq!(r.assisted_history(0).unwrap().total, 2);
    assert_eq!(r.get(&p.id).unwrap().version, p.version);
    let manifest: Manifest =
        serde_json::from_slice(&fs::read(path.join("informacoes.json")).unwrap()).unwrap();
    let page = r.assisted_list(query()).unwrap();
    let relative = &page.items[0].preparation.as_ref().unwrap().relative_path;
    let export = fs::canonicalize(dir.path().join("exports")).unwrap();
    let collision = write_package(
        &export,
        relative,
        &p,
        &manifest,
        |key| r.storage.path(key),
        &LocalIo,
    )
    .unwrap_err();
    assert_eq!(collision.code, "EXPORT_COLLISION");
    assert_eq!(
        fs::read(path.join("legenda.txt")).unwrap(),
        p.caption.as_bytes()
    );
}
#[test]
fn batch_exports_ordered_carousel_and_video_with_independent_outcomes() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    root(&r, &dir.path().join("exports"));
    let a = publication(&mut r, dir.path(), 2, PublicationFormat::Carousel);
    let b = publication(&mut r, dir.path(), 3, PublicationFormat::Reels);
    let sa = planned(&mut r, &a, "2099-11-03");
    let sb = planned(&mut r, &b, "2099-11-04");
    let results = r
        .assisted_prepare(vec![target(&a, &sa), target(&b, &sb)])
        .unwrap();
    assert!(results.iter().all(|x| x.error.is_none()));
    let folder = r
        .assisted_folder(results[0].preparation_id.as_ref().unwrap())
        .unwrap();
    for (i, m) in a.media.iter().enumerate() {
        assert_eq!(
            file_digest(&Path::new(&folder).join(format!("media_{:02}.png", i + 1))).unwrap(),
            m.checksum
        )
    }
    let folder = r
        .assisted_folder(results[1].preparation_id.as_ref().unwrap())
        .unwrap();
    assert_eq!(
        file_digest(&Path::new(&folder).join("media_01.mp4")).unwrap(),
        b.media[0].checksum
    );
    let mut stale = target(&a, &sa);
    stale.publication_version += 1;
    let results = r.assisted_prepare(vec![stale, target(&b, &sb)]).unwrap();
    assert!(results[0].error.is_some());
    assert!(results[1].error.is_none());
    assert!(r
        .assisted_prepare(vec![target(&a, &sa), target(&a, &sa)])
        .is_err());
}
#[test]
fn invalid_roots_and_traversal_and_collisions_never_overwrite() {
    let dir = tempfile::tempdir().unwrap();
    let r = EditorialRepository::open(dir.path()).unwrap();
    assert!(r.assisted_choose_root(Path::new("relative")).is_err());
    assert!(r.assisted_choose_root(&dir.path().join("missing")).is_err());
    assert!(r.assisted_choose_root(&r.storage.directory).is_err());
    root(&r, &dir.path().join("exports"));
    let canonical = fs::canonicalize(dir.path().join("exports")).unwrap();
    for rel in [
        "../escape",
        "2026-10/../../escape",
        "C:/escape",
        "2026-10/CON:bad",
        "/absolute",
    ] {
        assert!(inside(&canonical, rel).is_err(), "{rel}")
    }
    assert_eq!(slug("CON:<>?./\\"), "post-con");
}
struct NoSpace;
impl ExportIo for NoSpace {
    fn available(&self, _: &Path) -> Result<u64, crate::error::AppError> {
        Ok(0)
    }
}
struct FailedWrite;
impl ExportIo for FailedWrite {
    fn available(&self, _: &Path) -> Result<u64, crate::error::AppError> {
        Ok(u64::MAX)
    }
    fn write_guard(&self) -> Result<(), crate::error::AppError> {
        Err(io_error(std::io::Error::from_raw_os_error(5)))
    }
}
#[test]
fn disk_full_and_failed_write_audit_failures_clean_staging_and_retry() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    root(&r, &dir.path().join("exports"));
    let p = publication(&mut r, dir.path(), 4, PublicationFormat::Feed);
    let s = planned(&mut r, &p, "2099-11-03");
    for io in [&NoSpace as &dyn ExportIo, &FailedWrite as &dyn ExportIo] {
        let result = r.assisted_prepare_with(vec![target(&p, &s)], io).unwrap();
        assert!(result[0].error.is_some());
        assert_eq!(
            r.assisted_list(query()).unwrap().items[0]
                .preparation
                .as_ref()
                .unwrap()
                .status,
            "preparation_failed"
        );
    }
    let month = dir.path().join("exports/2099-11");
    if month.exists() {
        assert_eq!(fs::read_dir(month).unwrap().count(), 0)
    }
    let prep = prepared(&mut r, &p, &s);
    assert!(r.assisted_folder(&prep).is_ok());
    assert!(r
        .assisted_history(0)
        .unwrap()
        .items
        .iter()
        .any(|e| e.event == "package_regenerated"));
}
#[test]
fn missing_or_changed_source_media_and_changed_export_are_detected() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    root(&r, &dir.path().join("exports"));
    let p = publication(&mut r, dir.path(), 5, PublicationFormat::Feed);
    let s = planned(&mut r, &p, "2099-11-03");
    let prep = prepared(&mut r, &p, &s);
    let folder = r.assisted_folder(&prep).unwrap();
    fs::write(Path::new(&folder).join("legenda.txt"), "changed").unwrap();
    assert!(r.assisted_caption(&prep).is_err());
    assert_eq!(r.assisted_list(query()).unwrap().counts.outdated, 1);
    let new = prepared(&mut r, &p, &s);
    assert_ne!(new, prep);
    assert_eq!(
        fs::read_to_string(Path::new(&folder).join("legenda.txt")).unwrap(),
        "changed"
    );
    let source = r.storage.path(&p.media[0].storage_path).unwrap();
    let bytes = fs::read(&source).unwrap();
    fs::write(&source, vec![0; bytes.len()]).unwrap();
    assert_eq!(r.assisted_list(query()).unwrap().counts.outdated, 1);
    assert!(r.assisted_prepare(vec![target(&p, &s)]).unwrap()[0]
        .error
        .is_some());
    fs::remove_file(&source).unwrap();
    assert!(!r.assisted_list(query()).unwrap().items[0].eligible);
}
#[test]
fn reschedule_marks_package_outdated_and_regeneration_keeps_old_files() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    root(&r, &dir.path().join("exports"));
    let p = publication(&mut r, dir.path(), 6, PublicationFormat::Feed);
    let s = planned(&mut r, &p, "2099-11-03");
    let old = prepared(&mut r, &p, &s);
    let folder = r.assisted_folder(&old).unwrap();
    let signature = r.assisted_list(query()).unwrap().items[0]
        .preparation
        .as_ref()
        .unwrap()
        .fingerprint
        .clone();
    r.schedule_save(ScheduleInput {
        id: Some(s.clone()),
        expected_version: Some(1),
        publication_id: p.id.clone(),
        publication_version: p.version,
        scheduled_date: "2099-11-04".into(),
        scheduled_time: "11:30".into(),
        timezone: "America/Sao_Paulo".into(),
    })
    .unwrap();
    assert_eq!(r.assisted_list(query()).unwrap().counts.outdated, 1);
    let mut t = target(&p, &s);
    t.schedule_version = 2;
    let new = r.assisted_prepare(vec![t]).unwrap()[0]
        .preparation_id
        .clone()
        .unwrap();
    assert_ne!(new, old);
    assert!(Path::new(&folder).exists());
    assert_ne!(
        r.assisted_list(query()).unwrap().items[0]
            .preparation
            .as_ref()
            .unwrap()
            .fingerprint,
        signature
    );
}
#[test]
fn manual_confirmation_requires_checkbox_avoids_duplicates_and_preserves_snapshots() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    root(&r, &dir.path().join("exports"));
    let p = publication(&mut r, dir.path(), 7, PublicationFormat::Feed);
    let s = planned(&mut r, &p, "2099-11-03");
    let prep = prepared(&mut r, &p, &s);
    let mut bad = confirm(&prep);
    bad.verified_by_operator = false;
    assert!(r.assisted_confirm(bad).is_err());
    let old = r.assisted_confirm(confirm(&prep)).unwrap();
    assert!(r.assisted_confirm(confirm(&prep)).is_err());
    let mut revised = confirm(&prep);
    revised.previous_id = Some(old.clone());
    revised.expected_version = Some(1);
    revised.actual_time = "11:00".into();
    let new = r.assisted_confirm(revised).unwrap();
    assert_ne!(old, new);
    let page = r.assisted_list(query()).unwrap();
    assert_eq!(page.counts.confirmed, 1);
    assert_eq!(page.counts.divergent, 1);
    assert_eq!(page.items[0].schedule.scheduled_time, "10:00");
    assert_eq!(
        page.items[0].confirmation.as_ref().unwrap().actual_time,
        "11:00"
    );
    assert_eq!(
        r.database
            .connection
            .query_row(
                "SELECT status FROM assisted_confirmations WHERE id=?1",
                [old],
                |r| r.get::<_, String>(0)
            )
            .unwrap(),
        "superseded"
    );
    assert!(r
        .database
        .connection
        .execute("DELETE FROM assisted_events", [])
        .is_err());
    assert!(r
        .database
        .connection
        .execute("UPDATE assisted_events SET event='preparation_failed'", [])
        .is_err());
    assert_eq!(r.get(&p.id).unwrap().caption, p.caption);
}
#[test]
fn explicit_reconciliation_changes_only_calendar_and_requires_new_confirmation() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    root(&r, &dir.path().join("exports"));
    let p = publication(&mut r, dir.path(), 8, PublicationFormat::Feed);
    let s = planned(&mut r, &p, "2099-11-03");
    let prep = prepared(&mut r, &p, &s);
    let mut input = confirm(&prep);
    input.actual_time = "11:00".into();
    let c = r.assisted_confirm(input).unwrap();
    r.assisted_reconcile(ReconcileInput {
        confirmation_id: c,
        expected_version: 1,
        schedule_version: 1,
    })
    .unwrap();
    let page = r.assisted_list(query()).unwrap();
    assert_eq!(page.items[0].schedule.scheduled_time, "11:00");
    assert_eq!(
        page.items[0].confirmation.as_ref().unwrap().status,
        "requires_review"
    );
    assert_eq!(
        page.items[0].preparation.as_ref().unwrap().status,
        "outdated"
    );
    assert_eq!(r.get(&p.id).unwrap().version, p.version);
}
#[test]
fn editorial_edits_and_media_order_changes_invalidate_confirmation_transactionally() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    root(&r, &dir.path().join("exports"));
    let p = publication(&mut r, dir.path(), 9, PublicationFormat::Carousel);
    let s = planned(&mut r, &p, "2099-11-03");
    let prep = prepared(&mut r, &p, &s);
    r.assisted_confirm(confirm(&prep)).unwrap();
    let mut ids = p.media.iter().map(|m| m.id.clone()).collect::<Vec<_>>();
    ids.reverse();
    r.save(SavePublication {
        id: Some(p.id.clone()),
        expected_version: Some(p.version),
        title: p.title.clone(),
        caption: "Legenda alterada 💚".into(),
        format: PublicationFormat::Carousel,
        media_ids: ids,
    })
    .unwrap();
    let page = r.assisted_list(query()).unwrap();
    assert_eq!(page.counts.outdated, 1);
    assert_eq!(page.counts.divergent, 1);
    assert!(!page.items[0].eligible);
    assert_eq!(
        page.items[0].confirmation.as_ref().unwrap().status,
        "requires_review"
    );
    assert_eq!(page.items[0].schedule.status, "invalidated_local");
}
#[test]
fn roots_preparations_confirmation_history_and_integrity_survive_reopen() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    root(&r, &dir.path().join("exports"));
    let p = publication(&mut r, dir.path(), 10, PublicationFormat::Feed);
    let s = planned(&mut r, &p, "2099-11-03");
    let prep = prepared(&mut r, &p, &s);
    r.assisted_confirm(confirm(&prep)).unwrap();
    drop(r);
    let r = EditorialRepository::open(dir.path()).unwrap();
    let page = r.assisted_list(query()).unwrap();
    assert_eq!(page.counts.confirmed, 1);
    assert_eq!(page.counts.prepared, 1);
    assert_eq!(page.counts.pending_transfer, 0);
    assert!(page.export_root.is_some());
    assert_eq!(r.assisted_history(0).unwrap().total, 3);
    assert_eq!(r.database.status().unwrap().schema_version, 6);
    assert_eq!(r.get(&p.id).unwrap().caption, p.caption);
    assert!(!r
        .database
        .connection
        .prepare("PRAGMA foreign_key_check")
        .unwrap()
        .exists([])
        .unwrap());
}

#[test]
fn new_plan_after_editorial_edit_revises_old_confirmation_without_active_duplicate() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    root(&r, &dir.path().join("exports"));
    let p = publication(&mut r, dir.path(), 11, PublicationFormat::Feed);
    let s = planned(&mut r, &p, "2099-11-03");
    let prep = prepared(&mut r, &p, &s);
    let old = r.assisted_confirm(confirm(&prep)).unwrap();
    let changed = r
        .save(SavePublication {
            id: Some(p.id.clone()),
            expected_version: Some(p.version),
            title: p.title,
            caption: "Nova legenda aprovada 💚".into(),
            format: PublicationFormat::Feed,
            media_ids: p.media.iter().map(|m| m.id.clone()).collect(),
        })
        .unwrap();
    assert!(
        r.review_decide(ReviewDecision {
            targets: vec![ReviewTarget {
                id: p.id.clone(),
                expected_version: changed.version
            }],
            action: ReviewAction::Approve,
            reason: None,
            note: None
        })
        .unwrap()[0]
            .success
    );
    let current = r.get(&p.id).unwrap();
    let new_plan = planned(&mut r, &current, "2099-11-03");
    let new_prep = prepared(&mut r, &current, &new_plan);
    assert!(r.assisted_confirm(confirm(&new_prep)).is_err());
    let page = r.assisted_list(query()).unwrap();
    let active = page
        .items
        .iter()
        .find(|i| i.schedule.id == new_plan)
        .unwrap();
    let previous = active.confirmation.as_ref().unwrap();
    assert_eq!(previous.id, old);
    assert_eq!(previous.status, "requires_review");
    let mut revised = confirm(&new_prep);
    revised.previous_id = Some(old);
    revised.expected_version = Some(previous.version);
    r.assisted_confirm(revised).unwrap();
    let count:i64=r.database.connection.query_row("SELECT count(*) FROM assisted_confirmations WHERE publication_id=?1 AND status!='superseded'",[&p.id],|r|r.get(0)).unwrap();
    assert_eq!(count, 1);
    assert_eq!(r.assisted_list(query()).unwrap().counts.divergent, 0);
}

#[test]
fn interrupted_preparation_recovers_only_recorded_staging_and_preserves_other_files() {
    let dir = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(dir.path()).unwrap();
    root(&r, &dir.path().join("exports"));
    let p = publication(&mut r, dir.path(), 12, PublicationFormat::Feed);
    let s = planned(&mut r, &p, "2099-11-03");
    let prep = uuid::Uuid::new_v4().to_string();
    let root_id: String = r
        .database
        .connection
        .query_row("SELECT root_id FROM assisted_settings", [], |r| r.get(0))
        .unwrap();
    r.database.connection.execute("INSERT INTO preparations(id,publication_id,schedule_id,publication_version,schedule_version,fingerprint,status,root_id,relative_path) VALUES(?1,?2,?3,?4,1,?5,'preparing',?6,'2099-11/package')",rusqlite::params![prep,p.id,s,p.version,"a".repeat(64),root_id]).unwrap();
    let parent = dir.path().join("exports/2099-11");
    fs::create_dir(&parent).unwrap();
    let stage = parent.join(format!(".tmp-{prep}"));
    fs::create_dir(&stage).unwrap();
    fs::write(stage.join("media_01.png"), "partial").unwrap();
    fs::write(parent.join("preserve.txt"), "keep").unwrap();
    drop(r);
    let r = EditorialRepository::open(dir.path()).unwrap();
    assert!(!stage.exists());
    assert_eq!(
        fs::read_to_string(parent.join("preserve.txt")).unwrap(),
        "keep"
    );
    let page = r.assisted_list(query()).unwrap();
    assert_eq!(
        page.items[0]
            .preparation
            .as_ref()
            .unwrap()
            .error_code
            .as_deref(),
        Some("EXPORT_INTERRUPTED")
    );
    assert_eq!(page.counts.failed, 1);
}
