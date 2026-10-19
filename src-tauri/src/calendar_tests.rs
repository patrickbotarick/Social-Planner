use crate::{
    calendar::*, editorial_domain::*, editorial_repository::EditorialRepository, review::*,
};
use std::path::Path;
fn approved(r: &mut EditorialRepository, root: &Path, n: u8) -> Publication {
    let path = root.join(format!("{n}.png"));
    image::RgbImage::from_pixel(24, 32, image::Rgb([n, 90, 30]))
        .save(&path)
        .unwrap();
    let m = r.storage.stage(&path).unwrap();
    let p = r
        .save(SavePublication {
            id: None,
            expected_version: None,
            title: format!("QA Calendário {n}"),
            caption: "Rio Preto 😊\n#GMAD".into(),
            format: PublicationFormat::Feed,
            media_ids: vec![m.id],
        })
        .unwrap();
    for action in [ReviewAction::Submit, ReviewAction::Approve] {
        let p = r.get(&p.id).unwrap();
        assert!(
            r.review_decide(ReviewDecision {
                targets: vec![ReviewTarget {
                    id: p.id,
                    expected_version: p.version
                }],
                action,
                reason: None,
                note: None
            })
            .unwrap()[0]
                .success
        );
    }
    r.get(&p.id).unwrap()
}
fn input(p: &Publication, date: &str, time: &str) -> ScheduleInput {
    ScheduleInput {
        id: None,
        expected_version: None,
        publication_id: p.id.clone(),
        publication_version: p.version,
        scheduled_date: date.into(),
        scheduled_time: time.into(),
        timezone: "America/Sao_Paulo".into(),
    }
}
fn query() -> CalendarQuery {
    CalendarQuery {
        year: 2099,
        month: 11,
        timezone: "America/Sao_Paulo".into(),
        ..Default::default()
    }
}
fn available() -> AvailableQuery {
    AvailableQuery {
        search: String::new(),
        format: None,
        source: None,
        board_id: None,
        media_type: None,
        offset: 0,
    }
}
#[test]
fn timezone_validates_wall_clock_dst_and_midnight_utc() {
    assert_eq!(
        local_instant("2099-11-03", "23:30", "America/Sao_Paulo")
            .unwrap()
            .to_rfc3339(),
        "2099-11-04T02:30:00+00:00"
    );
    for (d, t, z) in [
        ("2026-02-30", "10:00", "UTC"),
        ("2026-01-01", "24:00", "UTC"),
        ("2026-01-01", "9:00", "UTC"),
        ("2026-01-01", "10:00", "Invalid/Zone"),
        ("2018-11-04", "00:30", "America/Sao_Paulo"),
        ("2026-11-01", "01:30", "America/New_York"),
    ] {
        assert!(local_instant(d, t, z).is_err(), "{d} {t} {z}");
    }
    assert_eq!(
        local_instant("2099-11-03", "09:15", "Asia/Kolkata")
            .unwrap()
            .to_rfc3339(),
        "2099-11-03T03:45:00+00:00"
    );
}
#[test]
fn plans_moves_cancels_replans_without_copying_media_or_changing_approval() {
    let root = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(root.path()).unwrap();
    let p = approved(&mut r, root.path(), 1);
    let id = r.schedule_save(input(&p, "2099-11-03", "10:00")).unwrap();
    assert!(r.calendar_available(available()).unwrap().items.is_empty());
    assert!(r.schedule_save(input(&p, "2099-11-04", "10:00")).is_err());
    let mut edit = input(&p, "2099-11-04", "11:15");
    edit.id = Some(id.clone());
    edit.expected_version = Some(1);
    r.schedule_save(edit).unwrap();
    let page = r.calendar_list(query()).unwrap();
    assert_eq!(page.total, 1);
    assert_eq!(page.items[0].display_date, "2099-11-04");
    assert_eq!(page.items[0].display_time, "11:15");
    assert_eq!(page.items[0].version, 2);
    assert!(r.schedule_cancel(&id, 1).is_err());
    r.schedule_cancel(&id, 2).unwrap();
    assert_eq!(r.get(&p.id).unwrap().editorial_status, "approved");
    assert_eq!(r.calendar_available(available()).unwrap().total, 1);
    assert_eq!(r.get(&p.id).unwrap().media[0].checksum, p.media[0].checksum);
    r.schedule_save(input(&p, "2099-11-05", "12:00")).unwrap();
    assert_eq!(
        r.database
            .connection
            .query_row("SELECT count(*) FROM media_assets", [], |r| r
                .get::<_, u32>(0))
            .unwrap(),
        1
    );
    let h = r.schedule_history(0).unwrap();
    assert_eq!(h.total, 4);
    assert_eq!(h.items[0].event, "created_local");
    assert_eq!(h.items[1].event, "cancelled_local");
    assert_eq!(h.items[2].previous_time.as_deref(), Some("10:00"));
    assert!(r
        .database
        .connection
        .execute("DELETE FROM schedules", [])
        .is_err());
    assert!(r
        .database
        .connection
        .execute("UPDATE schedule_events SET event='created_local'", [])
        .is_err());
}
#[test]
fn rejects_pending_ignored_deleted_missing_media_past_and_stale_versions() {
    let root = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(root.path()).unwrap();
    let p = approved(&mut r, root.path(), 2);
    assert!(r.schedule_save(input(&p, "2000-01-01", "10:00")).is_err());
    let mut stale = input(&p, "2099-11-03", "10:00");
    stale.publication_version -= 1;
    assert!(r.schedule_save(stale).is_err());
    std::fs::remove_file(r.storage.path(&p.media[0].storage_path).unwrap()).unwrap();
    assert!(r.schedule_save(input(&p, "2099-11-03", "10:00")).is_err());
    let p = approved(&mut r, root.path(), 3);
    r.database
        .connection
        .execute(
            "UPDATE publications SET editorial_status='pending_review' WHERE id=?1",
            [&p.id],
        )
        .unwrap();
    assert!(r.schedule_save(input(&p, "2099-11-03", "10:00")).is_err());
    r.database
        .connection
        .execute(
            "UPDATE publications SET editorial_status='ignored' WHERE id=?1",
            [&p.id],
        )
        .unwrap();
    assert!(r.schedule_save(input(&p, "2099-11-03", "10:00")).is_err());
    assert_eq!(r.calendar_available(available()).unwrap().total, 1); // aprovação com arquivo ausente é visível, mas não pode ser planejada
    r.delete(&p.id, p.version).unwrap();
    assert!(r.schedule_save(input(&p, "2099-11-03", "10:00")).is_err());
    assert_eq!(r.schedule_history(0).unwrap().total, 0);
}
#[test]
fn content_edits_review_reopening_and_logical_deletion_invalidate_atomically() {
    let root = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(root.path()).unwrap();
    for n in 4..7 {
        let p = approved(&mut r, root.path(), n);
        r.schedule_save(input(&p, "2099-11-03", "10:00")).unwrap();
        if n == 4 {
            r.save(SavePublication {
                id: Some(p.id.clone()),
                expected_version: Some(p.version),
                title: p.title.clone(),
                caption: "Editada".into(),
                format: PublicationFormat::Feed,
                media_ids: p.media.iter().map(|m| m.id.clone()).collect(),
            })
            .unwrap();
        } else if n == 5 {
            r.review_decide(ReviewDecision {
                targets: vec![ReviewTarget {
                    id: p.id.clone(),
                    expected_version: p.version,
                }],
                action: ReviewAction::Reopen,
                reason: None,
                note: None,
            })
            .unwrap();
        } else {
            r.delete(&p.id, p.version).unwrap();
        }
        let row = r
            .database
            .connection
            .query_row(
                "SELECT status,version FROM schedules WHERE publication_id=?1",
                [&p.id],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, u32>(1)?)),
            )
            .unwrap();
        assert_eq!(row, ("invalidated_local".into(), 2));
    }
    assert_eq!(r.calendar_list(query()).unwrap().total, 0);
    assert_eq!(r.calendar_list(query()).unwrap().items.len(), 3);
    assert_eq!(r.schedule_history(0).unwrap().total, 6);
    // Falha ao auditar a invalidação reverte também a edição editorial.
    let p = approved(&mut r, root.path(), 7);
    r.schedule_save(input(&p, "2099-11-03", "10:00")).unwrap();
    r.database.connection.execute_batch("CREATE TRIGGER reject_audit BEFORE INSERT ON schedule_events BEGIN SELECT RAISE(ABORT,'QA'); END;").unwrap();
    assert!(r.delete(&p.id, p.version).is_err());
    assert_eq!(r.get(&p.id).unwrap().version, p.version);
}
#[test]
fn conflicts_cross_midnight_filters_and_preferences_are_backend_queries() {
    let root = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(root.path()).unwrap();
    for (n, d, t) in [
        (8, "2099-11-03", "23:50"),
        (9, "2099-11-03", "23:50"),
        (10, "2099-11-04", "00:10"),
        (11, "2099-11-07", "12:00"),
    ] {
        let p = approved(&mut r, root.path(), n);
        r.schedule_save(input(&p, d, t)).unwrap();
    }
    let page = r.calendar_list(query()).unwrap();
    assert_eq!(page.total, 4);
    assert_eq!(page.empty_days, 27);
    assert_eq!(page.conflicts, 3);
    assert_eq!(page.items[0].conflict, Some("exact"));
    assert_eq!(page.items[2].conflict, Some("nearby"));
    assert_eq!(page.formats["feed"], 4);
    let mut q = query();
    q.search = "Calendario 10".into();
    q.source = Some("manual".into());
    assert_eq!(r.calendar_list(q).unwrap().total, 1);
    r.schedule_settings_save(SettingsInput {
        proximity_minutes: 0,
        time_step_minutes: 15,
        expected_version: 1,
    })
    .unwrap();
    assert_eq!(r.calendar_list(query()).unwrap().conflicts, 2);
    assert!(r
        .schedule_settings_save(SettingsInput {
            proximity_minutes: 0,
            time_step_minutes: 15,
            expected_version: 1
        })
        .is_err());
    assert!(r
        .schedule_settings_save(SettingsInput {
            proximity_minutes: 181,
            time_step_minutes: 13,
            expected_version: 2
        })
        .is_err());
}
#[test]
fn restart_preserves_schedule_rows_history_and_foreign_keys() {
    let root = tempfile::tempdir().unwrap();
    let mut r = EditorialRepository::open(root.path()).unwrap();
    let p = approved(&mut r, root.path(), 12);
    r.schedule_save(input(&p, "2099-11-03", "10:00")).unwrap();
    drop(r);
    let r = EditorialRepository::open(root.path()).unwrap();
    assert_eq!(r.calendar_list(query()).unwrap().total, 1);
    assert_eq!(r.schedule_history(0).unwrap().total, 1);
    assert_eq!(r.database.status().unwrap().integrity, "ok");
    assert_eq!(r.database.status().unwrap().schema_version, 5);
    assert!(!r
        .database
        .connection
        .prepare("PRAGMA foreign_key_check")
        .unwrap()
        .exists([])
        .unwrap());
    assert_eq!(
        r.planning_dashboard("America/Sao_Paulo")
            .unwrap()
            .upcoming
            .len(),
        1
    );
}
