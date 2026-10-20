//! Pacotes locais verificados. Nenhum acesso ao cofre ou à rede.
use crate::{
    assisted_files::{digest, linked, validate_root, ExportIo, LocalIo},
    database::{migrate, MIGRATIONS},
    editorial_repository::EditorialRepository,
    error::AppError,
};
use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions as FileOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use uuid::Uuid;
const MAX_FILE: u64 = 2 * 1024 * 1024 * 1024;
const RESERVE: u64 = 16 * 1024 * 1024;
pub(crate) fn invalid() -> AppError {
    AppError {
        code: "BACKUP_INVALID",
        message: "Pacote inválido, incompleto ou incompatível. Nenhum dado atual foi substituído.",
    }
}
pub(crate) fn io() -> AppError {
    AppError{code:"BACKUP_IO",message:"Não foi possível concluir. Confira pasta, permissões e espaço. Preserve o ponto de recuperação e tente novamente."}
}
pub(crate) fn busy() -> AppError {
    AppError {
        code: "BACKUP_BUSY",
        message:
            "Aguarde a operação terminar e feche outras instâncias deste perfil antes de continuar.",
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackupFile {
    pub path: String,
    pub bytes: u64,
    pub sha256: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct BackupManifest {
    pub kind: String,
    pub format: u32,
    pub id: String,
    pub app_version: String,
    pub schema_version: i64,
    pub created_at: String,
    pub files: Vec<BackupFile>,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupReport {
    pub path: String,
    pub manifest: BackupManifest,
    pub manifest_sha256: String,
    pub publications: u64,
    pub media: u64,
}
#[derive(Clone)]
pub struct Inspection {
    pub path: PathBuf,
    pub fingerprint: String,
    pub id: String,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RestoreReport {
    pub recovery_path: String,
    pub previous_directory: String,
    pub restart_required: bool,
}
fn sync_json<T: Serialize>(path: &Path, value: &T) -> Result<(), AppError> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|_| invalid())?;
    let mut f = FileOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(io_detail)?;
    f.write_all(&bytes).map_err(io_detail)?;
    f.sync_all().map_err(io_detail)
}
fn hash_file(path: &Path) -> Result<BackupFile, AppError> {
    if linked(path) {
        return Err(invalid());
    }
    let mut f = File::open(path).map_err(|_| invalid())?;
    let meta = f.metadata().map_err(|_| invalid())?;
    if !meta.is_file() || meta.len() > MAX_FILE {
        return Err(invalid());
    }
    use sha2::{Digest, Sha256};
    let mut hash = Sha256::new();
    let mut buf = [0; 65536];
    let mut total = 0;
    loop {
        let n = f.read(&mut buf).map_err(|_| io())?;
        if n == 0 {
            break;
        }
        total += n as u64;
        if total > MAX_FILE {
            return Err(invalid());
        }
        hash.update(&buf[..n]);
    }
    Ok(BackupFile {
        path: String::new(),
        bytes: total,
        sha256: format!("{:x}", hash.finalize()),
    })
}
fn media_name(name: &str) -> bool {
    let Some((hash, ext)) = name.split_once('.') else {
        return false;
    };
    hash.len() == 64
        && hash
            .bytes()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        && matches!(ext, "jpg" | "png" | "webp" | "mp4")
}
fn safe_relative(path: &str) -> bool {
    path == "planner.sqlite3" || path.strip_prefix("media/").is_some_and(media_name)
}
fn regular_dir(path: &Path) -> Result<PathBuf, AppError> {
    let root = validate_root(path).map_err(|_| invalid())?;
    if linked(path) {
        return Err(invalid());
    }
    Ok(root)
}
fn free_space(root: &Path, bytes: u64, io: &dyn ExportIo) -> Result<(), AppError> {
    if io.available(root)? < bytes.saturating_add(RESERVE) {
        return Err(AppError{code:"BACKUP_SPACE",message:"Espaço insuficiente. Libere espaço ou escolha outro destino; os dados atuais foram preservados."});
    }
    Ok(())
}
fn copy_verified(
    from: &Path,
    to: &Path,
    expected: &BackupFile,
    io: &dyn ExportIo,
) -> Result<(), AppError> {
    if linked(from) || linked(to) {
        return Err(invalid());
    }
    io.write_guard()?;
    let mut input = File::open(from).map_err(io_detail)?;
    let mut output = FileOptions::new()
        .write(true)
        .create_new(true)
        .open(to)
        .map_err(io_detail)?;
    let mut buffer = [0; 65536];
    let mut total = 0;
    loop {
        let n = input.read(&mut buffer).map_err(io_detail)?;
        if n == 0 {
            break;
        }
        total += n as u64;
        if total > expected.bytes {
            return Err(invalid());
        }
        output.write_all(&buffer[..n]).map_err(io_detail)?;
    }
    output.sync_all().map_err(io_detail)?;
    let actual = hash_file(to)?;
    if total != expected.bytes || actual.sha256 != expected.sha256 {
        return Err(invalid());
    }
    Ok(())
}
fn io_error() -> AppError {
    io()
}
fn io_detail(error: std::io::Error) -> AppError {
    if matches!(error.raw_os_error(), Some(28 | 39 | 112)) {
        AppError {
            code: "BACKUP_SPACE",
            message:
                "Espaço insuficiente para concluir o backup. Os dados atuais foram preservados.",
        }
    } else {
        io()
    }
}
type SchemaRow = (String, String, String, Option<String>);
fn schema_rows(c: &Connection) -> Result<Vec<SchemaRow>, AppError> {
    Ok(
        c.prepare("SELECT type,name,tbl_name,sql FROM sqlite_master ORDER BY type,name")?
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))?
            .collect::<Result<_, _>>()?,
    )
}
pub(crate) fn inspect_database(path: &Path) -> Result<(Connection, u64), AppError> {
    let mut header = [0u8; 20];
    File::open(path)
        .map_err(|_| invalid())?
        .read_exact(&mut header)
        .map_err(|_| invalid())?;
    if &header[..16] != b"SQLite format 3\0" || header[18] != 1 || header[19] != 1 {
        return Err(invalid());
    }
    let c = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|_| invalid())?;
    c.pragma_update(None, "trusted_schema", "OFF")
        .map_err(|_| invalid())?;
    let mut model = Connection::open_in_memory()?;
    migrate(&mut model, MIGRATIONS)?;
    if schema_rows(&c).map_err(|_| invalid())? != schema_rows(&model)? {
        return Err(invalid());
    }
    let rows = c
        .prepare("SELECT version,name,source_sql FROM schema_migrations ORDER BY version")?
        .query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;
    if rows.len() != MIGRATIONS.len()
        || rows
            .iter()
            .zip(MIGRATIONS)
            .any(|((v, n, s), (ev, en, es))| v != ev || n != en || s != es)
    {
        return Err(invalid());
    }
    let integrity: String = c
        .query_row("PRAGMA integrity_check", [], |r| r.get(0))
        .map_err(|_| invalid())?;
    if integrity != "ok" || c.prepare("PRAGMA foreign_key_check")?.exists([])? {
        return Err(invalid());
    }
    let publications = c.query_row(
        "SELECT count(*) FROM publications WHERE deleted_at IS NULL",
        [],
        |r| r.get::<_, u32>(0),
    )?;
    Ok((c, publications.into()))
}
pub fn inspect(root: &Path) -> Result<BackupReport, AppError> {
    let root = regular_dir(root)?;
    let manifest_path = root.join("manifesto.json");
    if linked(&manifest_path)
        || fs::metadata(&manifest_path).map_err(|_| invalid())?.len() > 16 * 1024 * 1024
    {
        return Err(invalid());
    }
    let bytes = fs::read(&manifest_path).map_err(|_| invalid())?;
    let manifest: BackupManifest = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
    if manifest.kind != "gmad-social-planner-backup"
        || manifest.format != 1
        || manifest.schema_version != MIGRATIONS.len() as i64
        || Uuid::parse_str(&manifest.id).is_err()
        || manifest.files.is_empty()
        || manifest.files.len() > 100001
        || chrono::DateTime::parse_from_rfc3339(&manifest.created_at).is_err()
    {
        return Err(invalid());
    }
    if linked(&root.join("media")) {
        return Err(invalid());
    }
    let mut expected = BTreeSet::from(["manifesto.json".to_string(), "media".to_string()]);
    let mut media_expected = BTreeSet::new();
    let mut total = 0u64;
    for f in &manifest.files {
        if !safe_relative(&f.path)
            || !expected.insert(f.path.clone())
            || f.bytes > MAX_FILE
            || f.sha256.len() != 64
        {
            return Err(invalid());
        }
        total = total.checked_add(f.bytes).ok_or_else(invalid)?;
        if total > 64 * 1024 * 1024 * 1024 {
            return Err(invalid());
        }
        if let Some(n) = f.path.strip_prefix("media/") {
            if f.sha256 != n[..64] {
                return Err(invalid());
            }
            media_expected.insert(n.to_string());
        }
        let actual = hash_file(&root.join(&f.path))?;
        if actual.bytes != f.bytes || actual.sha256 != f.sha256 {
            return Err(invalid());
        }
    }
    if !expected.contains("planner.sqlite3") || linked(&root.join("media")) {
        return Err(invalid());
    }
    let names = fs::read_dir(&root)
        .map_err(|_| invalid())?
        .map(|e| e.map(|e| e.file_name().to_string_lossy().into_owned()))
        .collect::<Result<BTreeSet<_>, _>>()
        .map_err(|_| invalid())?;
    if names
        != BTreeSet::from([
            "manifesto.json".into(),
            "planner.sqlite3".into(),
            "media".into(),
        ])
    {
        return Err(invalid());
    }
    let media_names = fs::read_dir(root.join("media"))
        .map_err(|_| invalid())?
        .map(|e| e.map(|e| e.file_name().to_string_lossy().into_owned()))
        .collect::<Result<BTreeSet<_>, _>>()
        .map_err(|_| invalid())?;
    if media_names != media_expected {
        return Err(invalid());
    }
    let (c, publications) = inspect_database(&root.join("planner.sqlite3"))?;
    for row in c
        .prepare("SELECT storage_path,checksum,size_bytes FROM media_assets")?
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, u32>(2)?,
            ))
        })?
    {
        let (name, hash, size) = row?;
        if !media_name(&name)
            || !manifest.files.iter().any(|f| {
                f.path == format!("media/{name}") && f.sha256 == hash && f.bytes == u64::from(size)
            })
        {
            return Err(invalid());
        }
    }
    for id in c
        .prepare("SELECT id FROM media_assets")?
        .query_map([], |r| r.get::<_, String>(0))?
    {
        let asset = crate::editorial_repository::asset(&c, &id?)?;
        crate::media_store::verify_asset(&root.join("media").join(&asset.storage_path), &asset)
            .map_err(|_| invalid())?;
    }
    drop(c);
    Ok(BackupReport {
        path: root.to_string_lossy().into_owned(),
        manifest_sha256: digest(&bytes),
        media: media_expected.len() as u64,
        publications,
        manifest,
    })
}
pub fn create(r: &EditorialRepository, destination: &Path) -> Result<BackupReport, AppError> {
    create_with(r, destination, &LocalIo)
}
pub(crate) fn create_with(
    r: &EditorialRepository,
    destination: &Path,
    io: &dyn ExportIo,
) -> Result<BackupReport, AppError> {
    let dest = regular_dir(destination)?;
    let data = regular_dir(r.storage.directory.parent().ok_or_else(invalid)?)?;

    if dest.starts_with(&data) || data.starts_with(&dest) {
        return Err(invalid());
    }
    let mut files = Vec::new();
    let mut size = fs::metadata(data.join("planner.sqlite3"))
        .map_err(io_detail)?
        .len()
        .saturating_mul(2);
    for entry in fs::read_dir(&r.storage.directory).map_err(io_detail)? {
        let entry = entry.map_err(io_detail)?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if !media_name(&name) {
            return Err(invalid());
        }
        let mut f = hash_file(&entry.path())?;
        if f.sha256 != name[..64] {
            return Err(invalid());
        }
        f.path = format!("media/{name}");
        size = size.saturating_add(f.bytes);
        files.push(f);
    }
    files.sort_by(|a, b| a.path.cmp(&b.path));

    free_space(&dest, size, io)?;

    let id = Uuid::new_v4().to_string();
    // O manifesto completo finaliza o pacote numa pasta de nome estável.
    let stage = dest.join(format!(
        "GMAD-Backup-{}-{id}",
        chrono::Utc::now().format("%Y%m%d-%H%M%S")
    ));
    fs::create_dir(&stage).map_err(io_detail)?;

    let result = (|| {
        fs::create_dir(stage.join("media")).map_err(io_detail)?;
        io.write_guard()?;
        let database = stage.join("planner.sqlite3");

        {
            let mut output = Connection::open(&database)?;
            let backup = rusqlite::backup::Backup::new(&r.database.connection, &mut output)?;
            let started = std::time::Instant::now();
            loop {
                use rusqlite::backup::StepResult;
                match backup.step(128)? {
                    StepResult::Done => break,
                    StepResult::More => {}
                    StepResult::Busy | StepResult::Locked => {
                        std::thread::sleep(std::time::Duration::from_millis(5))
                    }
                    _ => return Err(io_error()),
                }
                if started.elapsed() > std::time::Duration::from_secs(120) {
                    return Err(busy());
                }
            }
            drop(backup);
            // O pacote é autossuficiente, sem dependência de WAL/SHM externos.
            output.pragma_update(None, "journal_mode", "DELETE")?;
        }

        FileOptions::new()
            .read(true)
            .write(true)
            .open(&database)
            .map_err(io_detail)?
            .sync_all()
            .map_err(io_detail)?;
        for f in &files {
            copy_verified(&data.join(&f.path), &stage.join(&f.path), f, io)?;
        }

        let mut db = hash_file(&database)?;
        db.path = "planner.sqlite3".into();
        files.insert(0, db);
        let manifest = BackupManifest {
            kind: "gmad-social-planner-backup".into(),
            format: 1,
            id,
            app_version: env!("CARGO_PKG_VERSION").into(),
            schema_version: MIGRATIONS.len() as i64,
            created_at: chrono::Utc::now().to_rfc3339(),
            files,
        };
        sync_json(&stage.join("manifesto.json"), &manifest)?;
        let report = inspect(&stage)?;

        if linked(&dest) || dest.canonicalize().map_err(io_detail)? != dest {
            return Err(invalid());
        }
        Ok(report)
    })();
    if result.is_err() {
        cleanup_partial(&stage);
    }
    result
}
fn rename_directory(from: &Path, to: &Path) -> Result<(), AppError> {
    if to.exists() || linked(from) || linked(to) {
        return Err(invalid());
    }
    for attempt in 0..40 {
        match fs::rename(from, to) {
            Ok(()) => return Ok(()),
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::PermissionDenied | std::io::ErrorKind::WouldBlock
                ) && attempt < 39 =>
            {
                std::thread::sleep(std::time::Duration::from_millis(50))
            }
            Err(e) => return Err(io_detail(e)),
        }
    }
    Err(io())
}
fn cleanup_partial(stage: &Path) {
    if linked(stage) || stage.parent().is_some_and(linked) || linked(&stage.join("media")) {
        return;
    }
    if let Ok(files) = fs::read_dir(stage.join("media")) {
        for f in files.flatten() {
            if !linked(&f.path()) && f.path().is_file() {
                let _ = fs::remove_file(f.path());
            }
        }
    }
    let _ = fs::remove_dir(stage.join("media"));
    for name in ["planner.sqlite3", "manifesto.json"] {
        let path = stage.join(name);
        if !linked(&path) {
            let _ = fs::remove_file(path);
        }
    }
    let _ = fs::remove_dir(stage);
}
fn profile_key(data: &Path) -> String {
    digest(data.to_string_lossy().to_ascii_lowercase().as_bytes())[..16].to_string()
}
fn journal_path(data: &Path) -> Result<PathBuf, AppError> {
    Ok(data
        .parent()
        .ok_or_else(invalid)?
        .join(format!(".gmad-restore-{}.json", profile_key(data))))
}
fn slot(data: &Path, id: &str, label: &str) -> Result<PathBuf, AppError> {
    if Uuid::parse_str(id).is_err() {
        return Err(invalid());
    }
    Ok(data
        .parent()
        .ok_or_else(invalid)?
        .join(format!(".gmad-{}-{label}-{id}", profile_key(data))))
}
pub fn recover(data: &Path) -> Result<(), AppError> {
    let journal = journal_path(data)?;
    if !journal.exists() {
        return Ok(());
    }
    if linked(&journal) || fs::metadata(&journal).map_err(|_| io())?.len() > 100 {
        return Err(invalid());
    }
    let id: String =
        serde_json::from_slice(&fs::read(&journal).map_err(|_| io())?).map_err(|_| invalid())?;
    let old = slot(data, &id, "before")?;
    if linked(data) || linked(&old) {
        return Err(invalid());
    }
    if old.exists() {
        if data.exists() {
            rename_directory(data, &slot(data, &id, "aborted")?)?;
        }
        rename_directory(&old, data)?;
    } else if !data.join("planner.sqlite3").is_file() {
        return Err(io());
    }
    fs::remove_file(journal).map_err(|_| io())?;
    Ok(())
}
pub fn profile_lock(data: &Path) -> Result<File, AppError> {
    let path = data
        .parent()
        .ok_or_else(invalid)?
        .join(format!(".gmad-{}.lock", profile_key(data)));
    let mut options = FileOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.share_mode(0);
    }
    options.open(path).map_err(|_| busy())
}
pub fn restore(
    state: &mut Result<EditorialRepository, AppError>,
    chosen: &Inspection,
    confirmed: bool,
) -> Result<RestoreReport, AppError> {
    restore_with(state, chosen, confirmed, 0)
}
pub(crate) fn restore_with(
    state: &mut Result<EditorialRepository, AppError>,
    chosen: &Inspection,
    confirmed: bool,
    fault: u8,
) -> Result<RestoreReport, AppError> {
    if !confirmed {
        return Err(invalid());
    }
    let checked = inspect(&chosen.path)?;
    if checked.manifest_sha256 != chosen.fingerprint {
        return Err(invalid());
    }
    let current = state.as_ref().map_err(Clone::clone)?;
    let data = current
        .storage
        .directory
        .parent()
        .ok_or_else(invalid)?
        .canonicalize()
        .map_err(|_| io())?;
    if chosen.path.starts_with(&data) || data.starts_with(&chosen.path) {
        return Err(invalid());
    }
    let id = Uuid::new_v4().to_string();
    let stage = slot(&data, &id, "stage")?;
    let previous = slot(&data, &id, "before")?;
    let recovery_parent = slot(&data, &id, "recovery")?;
    let total = checked.manifest.files.iter().map(|f| f.bytes).sum::<u64>();
    free_space(data.parent().ok_or_else(invalid)?, total, &LocalIo)?;
    fs::create_dir(&stage).map_err(|_| io())?;
    fs::create_dir(stage.join("media")).map_err(|_| io())?;
    let prepared = (|| {
        for f in &checked.manifest.files {
            copy_verified(
                &chosen.path.join(&f.path),
                &stage.join(&f.path),
                f,
                &LocalIo,
            )?;
        }
        sync_json(&stage.join("manifesto.json"), &checked.manifest)?;
        inspect(&stage)?;
        Ok::<_, AppError>(())
    })();
    if let Err(e) = prepared {
        cleanup_partial(&stage);
        return Err(e);
    }
    fs::create_dir(&recovery_parent).map_err(|_| io())?;
    let recovery = match create(current, &recovery_parent) {
        Ok(v) => v,
        Err(e) => {
            cleanup_partial(&stage);
            return Err(e);
        }
    };
    // O banco foi validado com schema exato antes de executar qualquer SQL do pacote.
    {
        let c = Connection::open(stage.join("planner.sqlite3"))?;
        c.pragma_update(None, "foreign_keys", "ON")?;
        c.execute_batch("BEGIN IMMEDIATE; UPDATE preparations SET status='outdated' WHERE status IN ('preparing','prepared'); UPDATE assisted_confirmations SET status='requires_review',version=version+1 WHERE status='confirmed_manually'; UPDATE assisted_settings SET root_id=NULL WHERE id=1; COMMIT;")?;
    }
    fs::remove_file(stage.join("manifesto.json")).map_err(|_| io())?;
    let journal = journal_path(&data)?;
    sync_json(&journal, &id)?;
    drop(std::mem::replace(state, Err(busy())));
    let promoted = (|| {
        rename_directory(&data, &previous)?;
        if fault == 1 {
            return Err(io());
        }
        rename_directory(&stage, &data)?;
        if fault == 2 {
            return Err(io());
        }
        let r = EditorialRepository::open(&data)?;
        r.trello_recover()?;
        inspect_database(&data.join("planner.sqlite3"))?;
        fs::remove_file(&journal).map_err(|_| io())?;
        Ok(r)
    })();
    match promoted {
        Ok(r) => {
            *state = Ok(r);
            Ok(RestoreReport {
                recovery_path: recovery.path,
                previous_directory: previous.to_string_lossy().into_owned(),
                restart_required: false,
            })
        }
        Err(e) => {
            recover(&data)?;
            *state = EditorialRepository::open(&data);
            Err(e)
        }
    }
}

#[cfg(test)]
mod recovery_tests {
    use super::*;
    #[test]
    fn startup_rolls_back_interrupted_directory_swap_without_deleting_either_dataset() {
        for after_promotion in [false, true] {
            let temp = tempfile::tempdir().unwrap();
            let data = temp.path().join("profile");
            fs::create_dir(&data).unwrap();
            let data = data.canonicalize().unwrap();
            let r = EditorialRepository::open(&data).unwrap();
            drop(r);
            fs::write(data.join("original-proof"), b"preserve").unwrap();
            let id = Uuid::new_v4().to_string();
            let stage = slot(&data, &id, "stage").unwrap();
            fs::create_dir(&stage).unwrap();
            fs::write(stage.join("candidate-proof"), b"candidate").unwrap();
            sync_json(&journal_path(&data).unwrap(), &id).unwrap();
            fs::rename(&data, slot(&data, &id, "before").unwrap()).unwrap();
            if after_promotion {
                fs::rename(&stage, &data).unwrap();
            }
            recover(&data).unwrap();
            assert_eq!(fs::read(data.join("original-proof")).unwrap(), b"preserve");
            assert!(EditorialRepository::open(&data).is_ok());
            let preserved = if after_promotion {
                slot(&data, &id, "aborted").unwrap()
            } else {
                stage
            };
            assert!(preserved.join("candidate-proof").exists());
            assert!(!journal_path(&data).unwrap().exists());
        }
    }
    #[test]
    fn forged_recovery_journal_cannot_redirect_directory_operations() {
        let temp = tempfile::tempdir().unwrap();
        let data = temp.path().join("profile");
        fs::create_dir(&data).unwrap();
        let data = data.canonicalize().unwrap();
        let r = EditorialRepository::open(&data).unwrap();
        drop(r);
        sync_json(&journal_path(&data).unwrap(), &"../elsewhere").unwrap();
        assert!(recover(&data).is_err());
        assert!(data.join("planner.sqlite3").exists());
    }
    #[cfg(windows)]
    #[test]
    fn one_windows_process_handle_per_profile_prevents_concurrent_restore() {
        let temp = tempfile::tempdir().unwrap();
        let data = temp.path().join("profile");
        let first = profile_lock(&data).unwrap();
        assert!(profile_lock(&data).is_err());
        drop(first);
        assert!(profile_lock(&data).is_ok());
    }
}
