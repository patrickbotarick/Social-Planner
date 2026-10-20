use crate::{
    assisted_files::{digest, ExportIo},
    backup::*,
    editorial_domain::*,
    editorial_repository::EditorialRepository,
    error::AppError,
};
use std::{fs, path::Path};
fn sample(root: &Path) -> (Result<EditorialRepository, AppError>, String) {
    fs::create_dir(root.join("profile")).unwrap();
    let mut r = EditorialRepository::open(&root.join("profile")).unwrap();
    let image = root.join("source.png");
    image::RgbImage::from_pixel(12, 20, image::Rgb([10, 90, 20]))
        .save(&image)
        .unwrap();
    let media = r.storage.stage(&image).unwrap();
    let p = r
        .save(SavePublication {
            id: None,
            expected_version: None,
            title: "Backup QA".into(),
            caption: "Texto 💚\nPreservar".into(),
            format: PublicationFormat::Feed,
            media_ids: vec![media.id],
        })
        .unwrap();
    fs::create_dir(root.join("backups")).unwrap();
    (Ok(r), p.id)
}
fn selection(report: &BackupReport) -> Inspection {
    Inspection {
        path: report.path.clone().into(),
        fingerprint: report.manifest_sha256.clone(),
        id: uuid::Uuid::new_v4().to_string(),
    }
}
fn rename_publication(state: &mut Result<EditorialRepository, AppError>, id: &str) {
    let r = state.as_mut().unwrap();
    let p = r.get(id).unwrap();
    r.save(SavePublication {
        id: Some(p.id),
        expected_version: Some(p.version),
        title: "Depois do backup".into(),
        caption: p.caption,
        format: PublicationFormat::Feed,
        media_ids: p.media.iter().map(|m| m.id.clone()).collect(),
    })
    .unwrap();
}
fn rewrite_manifest(path: &Path, mutate: impl FnOnce(&mut BackupManifest)) {
    let manifest = path.join("manifesto.json");
    let mut m: BackupManifest = serde_json::from_slice(&fs::read(&manifest).unwrap()).unwrap();
    mutate(&mut m);
    fs::write(manifest, serde_json::to_vec_pretty(&m).unwrap()).unwrap();
}
#[test]
fn consistent_snapshot_includes_committed_wal_and_preserves_preferences_media_without_vault() {
    let t = tempfile::tempdir().unwrap();
    let (state, id) = sample(t.path());
    let r = state.as_ref().unwrap();
    r.database
        .connection
        .pragma_update(None, "journal_mode", "WAL")
        .unwrap();
    let second = rusqlite::Connection::open(t.path().join("profile/planner.sqlite3")).unwrap();
    second
        .execute("UPDATE app_settings SET timezone='UTC' WHERE id=1", [])
        .unwrap();
    second
        .execute_batch(
            "BEGIN IMMEDIATE; UPDATE app_settings SET timezone='Europe/Paris' WHERE id=1;",
        )
        .unwrap();
    let b = create(r, &t.path().join("backups")).unwrap();
    let (copy, n) = inspect_database(&Path::new(&b.path).join("planner.sqlite3")).unwrap();
    assert_eq!(n, 1);
    assert_eq!(
        copy.query_row("SELECT timezone FROM app_settings", [], |r| r
            .get::<_, String>(0))
            .unwrap(),
        "UTC"
    );
    assert_eq!(b.media, 1);
    assert_eq!(r.get(&id).unwrap().caption, "Texto 💚\nPreservar");
    assert_eq!(
        b.manifest_sha256,
        digest(&fs::read(Path::new(&b.path).join("manifesto.json")).unwrap())
    );
    assert_eq!(b.manifest.files.len(), 2);
    assert!(!b
        .manifest
        .files
        .iter()
        .any(|f| f.path.contains("credential")));
    second.execute_batch("ROLLBACK").unwrap();
}
#[test]
fn repeat_backups_are_unique_and_old_packages_never_overwritten() {
    let t = tempfile::tempdir().unwrap();
    let (state, _) = sample(t.path());
    let a = create(state.as_ref().unwrap(), &t.path().join("backups")).unwrap();
    let bytes = fs::read(Path::new(&a.path).join("manifesto.json")).unwrap();
    let b = create(state.as_ref().unwrap(), &t.path().join("backups")).unwrap();
    assert_ne!(a.path, b.path);
    assert_eq!(
        fs::read(Path::new(&a.path).join("manifesto.json")).unwrap(),
        bytes
    );
    assert!(create(state.as_ref().unwrap(), &t.path().join("profile/media")).is_err());
}
#[test]
fn restore_replaces_data_keeps_verified_recovery_and_survives_reopen() {
    let t = tempfile::tempdir().unwrap();
    let (mut state, id) = sample(t.path());
    let b = create(state.as_ref().unwrap(), &t.path().join("backups")).unwrap();
    rename_publication(&mut state, &id);
    let s = selection(&b);
    assert!(restore(&mut state, &s, false).is_err());
    let result = restore(&mut state, &s, true).unwrap();
    assert_eq!(state.as_ref().unwrap().get(&id).unwrap().title, "Backup QA");
    let recovery = inspect(Path::new(&result.recovery_path)).unwrap();
    assert_eq!(recovery.publications, 1);
    assert_eq!(
        inspect_database(&Path::new(&recovery.path).join("planner.sqlite3"))
            .unwrap()
            .0
            .query_row("SELECT title FROM publications", [], |r| r
                .get::<_, String>(0))
            .unwrap(),
        "Depois do backup"
    );
    assert!(Path::new(&result.previous_directory).is_dir());
    drop(state);
    let r = EditorialRepository::open(&t.path().join("profile")).unwrap();
    assert_eq!(r.get(&id).unwrap().caption, "Texto 💚\nPreservar");
    assert!(r.assisted_root().unwrap().is_none());
}
#[test]
fn missing_media_and_changed_manifest_are_rejected_before_replacement() {
    let t = tempfile::tempdir().unwrap();
    let (mut state, id) = sample(t.path());
    let b = create(state.as_ref().unwrap(), &t.path().join("backups")).unwrap();
    let s = selection(&b);
    rewrite_manifest(Path::new(&b.path), |m| m.app_version = "changed".into());
    assert!(restore(&mut state, &s, true).is_err());
    assert_eq!(state.as_ref().unwrap().get(&id).unwrap().title, "Backup QA");
    let media = b
        .manifest
        .files
        .iter()
        .find(|f| f.path.starts_with("media/"))
        .unwrap();
    fs::remove_file(Path::new(&b.path).join(&media.path)).unwrap();
    assert!(inspect(Path::new(&b.path)).is_err());
}
#[test]
fn incompatible_manifest_and_path_traversal_are_rejected() {
    let t = tempfile::tempdir().unwrap();
    let (state, _) = sample(t.path());
    let b = create(state.as_ref().unwrap(), &t.path().join("backups")).unwrap();
    rewrite_manifest(Path::new(&b.path), |m| m.schema_version = 999);
    assert!(inspect(Path::new(&b.path)).is_err());
    rewrite_manifest(Path::new(&b.path), |m| {
        m.schema_version = 6;
        m.files[0].path = "../planner.sqlite3".into();
    });
    assert!(inspect(Path::new(&b.path)).is_err());
    assert!(state.as_ref().unwrap().database.status().is_ok());
}
#[test]
fn invalid_database_and_injected_schema_rejected_even_with_updated_hash() {
    let t = tempfile::tempdir().unwrap();
    let (state, _) = sample(t.path());
    let b = create(state.as_ref().unwrap(), &t.path().join("backups")).unwrap();
    let path = Path::new(&b.path);
    {
        let c = rusqlite::Connection::open(path.join("planner.sqlite3")).unwrap();
        c.execute_batch("CREATE TABLE unexpected(secret TEXT);")
            .unwrap();
    }
    let db = fs::read(path.join("planner.sqlite3")).unwrap();
    rewrite_manifest(path, |m| {
        m.files[0].sha256 = digest(&db);
        m.files[0].bytes = db.len() as u64;
    });
    assert!(inspect(path).is_err());
    fs::write(path.join("planner.sqlite3"), b"not sqlite").unwrap();
    assert!(inspect(path).is_err());
}
struct NoSpace;
impl ExportIo for NoSpace {
    fn available(&self, _: &Path) -> Result<u64, AppError> {
        Ok(0)
    }
}
struct FailedWrite;
impl ExportIo for FailedWrite {
    fn available(&self, _: &Path) -> Result<u64, AppError> {
        Ok(u64::MAX)
    }
    fn write_guard(&self) -> Result<(), AppError> {
        Err(io())
    }
}
#[test]
fn disk_full_and_partial_copy_failure_leave_source_and_existing_backups_intact() {
    let t = tempfile::tempdir().unwrap();
    let (state, id) = sample(t.path());
    let r = state.as_ref().unwrap();
    let dest = t.path().join("backups");
    assert_eq!(
        create_with(r, &dest, &NoSpace).err().unwrap().code,
        "BACKUP_SPACE"
    );
    assert!(create_with(r, &dest, &FailedWrite).is_err());
    assert_eq!(fs::read_dir(dest).unwrap().count(), 0);
    assert_eq!(r.get(&id).unwrap().title, "Backup QA");
    assert!(r.database.status().is_ok());
}
#[test]
fn partial_restore_after_first_or_second_rename_rolls_back_and_preserves_all_copies() {
    for fault in [1, 2] {
        let t = tempfile::tempdir().unwrap();
        let (mut state, id) = sample(t.path());
        let b = create(state.as_ref().unwrap(), &t.path().join("backups")).unwrap();
        rename_publication(&mut state, &id);
        assert!(restore_with(&mut state, &selection(&b), true, fault).is_err());
        assert_eq!(
            state.as_ref().unwrap().get(&id).unwrap().title,
            "Depois do backup"
        );
        assert!(inspect(Path::new(&b.path)).is_ok());
        drop(state);
        recover(&t.path().join("profile").canonicalize().unwrap()).unwrap();
        assert!(EditorialRepository::open(&t.path().join("profile")).is_ok());
    }
}
#[test]
fn malformed_or_forged_manifest_and_extra_files_fail_closed() {
    let t = tempfile::tempdir().unwrap();
    let (state, _) = sample(t.path());
    let b = create(state.as_ref().unwrap(), &t.path().join("backups")).unwrap();
    fs::write(
        Path::new(&b.path).join("token.txt"),
        "not a real credential",
    )
    .unwrap();
    assert!(inspect(Path::new(&b.path)).is_err());
    fs::remove_file(Path::new(&b.path).join("token.txt")).unwrap();
    fs::write(Path::new(&b.path).join("manifesto.json"), "{broken").unwrap();
    assert!(inspect(Path::new(&b.path)).is_err());
}
#[test]
fn opening_existing_database_as_update_preserves_data_and_prior_migrations() {
    let t = tempfile::tempdir().unwrap();
    let (state, id) = sample(t.path());
    let before = state.as_ref().unwrap().get(&id).unwrap();
    drop(state);
    let r = EditorialRepository::open(&t.path().join("profile")).unwrap();
    let after = r.get(&id).unwrap();
    assert_eq!(before.id, after.id);
    assert_eq!(before.version, after.version);
    assert_eq!(before.caption, after.caption);
    assert_eq!(before.media[0].checksum, after.media[0].checksum);
    assert_eq!(r.database.status().unwrap().schema_version, 6);
}
#[test]
fn restoring_recovery_point_returns_to_pre_restore_state() {
    let t = tempfile::tempdir().unwrap();
    let (mut state, id) = sample(t.path());
    let b = create(state.as_ref().unwrap(), &t.path().join("backups")).unwrap();
    rename_publication(&mut state, &id);
    let result = restore(&mut state, &selection(&b), true).unwrap();
    let recovery = inspect(Path::new(&result.recovery_path)).unwrap();
    restore(&mut state, &selection(&recovery), true).unwrap();
    assert_eq!(
        state.as_ref().unwrap().get(&id).unwrap().title,
        "Depois do backup"
    );
}
