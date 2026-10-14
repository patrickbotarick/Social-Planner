use crate::{
    error::AppError,
    media_store::MediaStore,
    trello_api::*,
    trello_domain::*,
    trello_repository::AttachmentResult,
    trello_vault::{Vault, WindowsVault},
    DatabaseState,
};
use serde::Serialize;
use std::{
    fs::{self, OpenOptions},
    path::Path,
    sync::{
        atomic::{AtomicBool, Ordering},
        Mutex,
    },
    time::{Duration, Instant},
};
use tauri::Manager;

pub struct TrelloState {
    busy: AtomicBool,
    cancel: AtomicBool,
    preview: Mutex<Option<(BoardPreview, Instant)>>,
    progress: Mutex<Option<Job>>,
}
impl Default for TrelloState {
    fn default() -> Self {
        Self {
            busy: AtomicBool::new(false),
            cancel: AtomicBool::new(false),
            preview: Mutex::new(None),
            progress: Mutex::new(None),
        }
    }
}
struct Operation<'a>(&'a TrelloState);
impl Drop for Operation<'_> {
    fn drop(&mut self) {
        self.0.busy.store(false, Ordering::SeqCst);
    }
}
impl TrelloState {
    fn enter(&self) -> Result<Operation<'_>, AppError> {
        if self
            .busy
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return Err(error("TRELLO_BUSY"));
        }
        self.cancel.store(false, Ordering::SeqCst);
        Ok(Operation(self))
    }
    fn update(&self, j: &Job) {
        if let Ok(mut p) = self.progress.lock() {
            *p = Some(j.clone());
        }
    }
}
fn repo<T>(
    app: &tauri::AppHandle,
    op: impl FnOnce(&mut crate::editorial_repository::EditorialRepository) -> Result<T, AppError>,
) -> Result<T, AppError> {
    let state = app.state::<DatabaseState>();
    let mut lock = state.0.lock().map_err(|_| AppError::database())?;
    match &mut *lock {
        Ok(r) => op(r),
        Err(e) => Err(e.clone()),
    }
}
async fn blocking<T: Send + 'static>(
    f: impl FnOnce() -> Result<T, AppError> + Send + 'static,
) -> Result<T, AppError> {
    tauri::async_runtime::spawn_blocking(f)
        .await
        .map_err(|_| error("TRELLO_RESPONSE"))?
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionStatus {
    configured: bool,
    account: Option<Account>,
}
#[tauri::command]
pub async fn trello_status() -> Result<ConnectionStatus, AppError> {
    blocking(|| {
        Ok(ConnectionStatus {
            configured: WindowsVault.load()?.is_some(),
            account: None,
        })
    })
    .await
}
#[tauri::command]
pub async fn trello_connect(
    app: tauri::AppHandle,
    key: String,
    token: String,
) -> Result<ConnectionStatus, AppError> {
    blocking(move || {
        let state = app.state::<TrelloState>();
        let _operation = state.enter()?;
        let credentials = Credentials { key, token };
        let mut api = OfficialTransport::new(&credentials, 30)?;
        let account = account(&mut api)?;
        WindowsVault.save(&credentials)?;
        *state.preview.lock().map_err(|_| error("TRELLO_BUSY"))? = None;
        Ok(ConnectionStatus {
            configured: true,
            account: Some(account),
        })
    })
    .await
}
#[tauri::command]
pub async fn trello_test(app: tauri::AppHandle) -> Result<ConnectionStatus, AppError> {
    blocking(move || {
        let state = app.state::<TrelloState>();
        let _operation = state.enter()?;
        let credentials = WindowsVault.load()?.ok_or_else(|| error("TRELLO_AUTH"))?;
        let mut api = OfficialTransport::new(&credentials, 30)?;
        Ok(ConnectionStatus {
            configured: true,
            account: Some(account(&mut api)?),
        })
    })
    .await
}
#[tauri::command]
pub async fn trello_disconnect(app: tauri::AppHandle) -> Result<(), AppError> {
    blocking(move || {
        let state = app.state::<TrelloState>();
        let _operation = state.enter()?;
        WindowsVault.remove()?;
        *state.preview.lock().map_err(|_| error("TRELLO_BUSY"))? = None;
        Ok(())
    })
    .await
}
#[tauri::command]
pub async fn trello_authorization_url(key: String) -> Result<String, AppError> {
    if key.len() != 32 || !key.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(error("TRELLO_AUTH"));
    }
    Ok(format!("https://trello.com/1/authorize?expiration=30days&scope=read&response_type=token&name=GMAD%20Social%20Planner&key={key}"))
}
#[tauri::command]
pub async fn trello_preview(app: tauri::AppHandle, url: String) -> Result<BoardPreview, AppError> {
    blocking(move || {
        let state = app.state::<TrelloState>();
        let _operation = state.enter()?;
        *state.preview.lock().map_err(|_| error("TRELLO_BUSY"))? = None;
        let short = board_short_link(&url)?;
        let credentials = WindowsVault.load()?.ok_or_else(|| error("TRELLO_AUTH"))?;
        let mut api = OfficialTransport::new(&credentials, 30)?;
        let (board, lists, cards) = read_board(&mut api, &short, &state.cancel)?;
        let preview = repo(&app, |r| r.trello_preview(board, lists, cards))?;
        *state.preview.lock().map_err(|_| error("TRELLO_BUSY"))? =
            Some((preview.clone(), Instant::now()));
        Ok(preview)
    })
    .await
}
#[derive(Serialize)]
pub struct ImportHistory {
    pub boards: Vec<BoardHistory>,
    pub jobs: Vec<Job>,
    pub active: Option<Job>,
}
#[tauri::command]
pub async fn trello_history(app: tauri::AppHandle) -> Result<ImportHistory, AppError> {
    blocking(move || {
        let state = app.state::<TrelloState>();
        let active = state
            .progress
            .lock()
            .map_err(|_| error("TRELLO_BUSY"))?
            .clone();
        repo(&app, |r| {
            Ok(ImportHistory {
                boards: r.trello_history()?,
                jobs: r.trello_jobs()?,
                active,
            })
        })
    })
    .await
}
#[tauri::command]
pub async fn trello_source(
    app: tauri::AppHandle,
    publication_id: String,
) -> Result<Option<SourceInfo>, AppError> {
    blocking(move || repo(&app, |r| r.trello_source(&publication_id))).await
}
#[tauri::command]
pub fn trello_cancel(state: tauri::State<'_, TrelloState>) {
    state.cancel.store(true, Ordering::SeqCst);
}

// Temporário de download com nome UUID, independente de nomes remotos. RAII remove somente este arquivo.
struct DownloadFile(std::path::PathBuf);
impl Drop for DownloadFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}
pub fn download_asset(
    api: &mut dyn Transport,
    storage: &mut MediaStore,
    card: &str,
    a: &Attachment,
    limits: &DownloadLimits,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(u64),
) -> Result<crate::editorial_domain::MediaAsset, AppError> {
    let ext = a.extension().ok_or_else(AppError::media)?;
    let maximum = if ext == "mp4" {
        limits.video_mib
    } else {
        limits.image_mib
    } * 1024
        * 1024;
    let file = DownloadFile(storage.staging_directory.join(format!(
        "download-{}.{}",
        uuid::Uuid::new_v4(),
        ext
    )));
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&file.0)
        .map_err(|_| AppError::file())?;
    download(api, card, a, maximum, &mut output, cancel, progress)?;
    output.sync_all().map_err(|_| AppError::file())?;
    drop(output);
    cancelled(cancel)?;
    let mut m = storage.stage(&file.0)?;
    m.filename = a.name.clone();
    storage.staged.insert(m.id.clone(), m.clone());
    let (relative, _) = storage.install(&m)?;
    m.storage_path = relative;
    m.preview_path = None;
    storage.discard(&[m.id.clone()])?;
    Ok(m)
}
pub struct ImportInput<'a> {
    pub data: &'a Path,
    pub preview: &'a BoardPreview,
    pub selected: &'a [String],
    pub limits: &'a DownloadLimits,
    pub cancel: &'a AtomicBool,
}
pub fn import_cards(
    api: &mut dyn Transport,
    input: ImportInput<'_>,
    job: &mut Job,
    mut access: impl FnMut(
        &mut dyn FnMut(
            &mut crate::editorial_repository::EditorialRepository,
        ) -> Result<(), AppError>,
    ) -> Result<(), AppError>,
    progress: &mut dyn FnMut(&Job),
) -> Result<(), AppError> {
    let ImportInput {
        data,
        preview,
        selected,
        limits,
        cancel,
    } = input;
    let mut storage = MediaStore::open(data)?;
    for item in &preview.cards {
        if !selected.contains(&item.card.id) {
            continue;
        }
        cancelled(cancel)?;
        let card = &item.card;
        job.current_card = Some(card.name.clone());
        job.downloaded_bytes = 0;
        progress(job);
        let mut results = Vec::new();
        let mut new_downloads = 0;
        let mut failures = 0;
        for a in &card.attachments {
            cancelled(cancel)?;
            let mut existing = None;
            access(&mut |r| {
                existing = r.trello_existing_media(a)?;
                Ok(())
            })?;
            let mut result = AttachmentResult {
                attachment: a.clone(),
                media: existing,
                status: "available".into(),
                error_message: None,
            };
            if !a.is_upload {
                result.status = "reference".into();
            } else if a.extension().is_none() {
                result.status = "unsupported".into();
            } else if result.media.is_none() {
                match download_asset(
                    api,
                    &mut storage,
                    &card.id,
                    a,
                    limits,
                    cancel,
                    &mut |bytes| {
                        job.downloaded_bytes = bytes;
                        progress(job)
                    },
                ) {
                    Ok(m) => {
                        new_downloads += 1;
                        result.media = Some(m);
                    }
                    Err(e) if e.code == "TRELLO_CANCELLED" => return Err(e),
                    Err(e) => {
                        failures += 1;
                        result.status = "failed".into();
                        result.error_message = Some(e.message.into());
                    }
                }
            }
            results.push(result);
        }
        cancelled(cancel)?;
        let list = preview
            .lists
            .iter()
            .find(|l| l.id == card.id_list)
            .map(|l| l.name.as_str())
            .unwrap_or("Lista indisponível");
        let mut outcome = (false, false);
        match access(&mut |r| {
            outcome = r.trello_commit_card(&preview.board, card, list, &results)?;
            Ok(())
        }) {
            Ok(()) => {
                job.successful_cards += 1;
                job.new_publications += u32::from(outcome.0);
                job.changed_sources += u32::from(outcome.1);
                job.downloaded_attachments += new_downloads;
                job.attachment_failures += failures;
            }
            Err(e) => {
                job.failed_cards += 1;
                job.error_message = Some(e.message.into());
            }
        }
        job.processed_cards += 1;
        access(&mut |r| r.trello_job_save(job))?;
        progress(job);
    }
    cancelled(cancel)?;
    access(&mut |r| {
        r.trello_finish_snapshot(
            &preview.board,
            &preview.cards,
            job.failed_cards == 0 && job.attachment_failures == 0,
        )
    })?;
    Ok(())
}
#[tauri::command]
pub async fn trello_import(
    app: tauri::AppHandle,
    preview_id: String,
    selected: Vec<String>,
    limits: DownloadLimits,
) -> Result<Job, AppError> {
    blocking(move || {
        limits.validate()?;
        let state = app.state::<TrelloState>();
        let _operation = state.enter()?;
        let preview = {
            let cache = state.preview.lock().map_err(|_| error("TRELLO_BUSY"))?;
            let (p, time) = cache.as_ref().ok_or_else(|| error("TRELLO_PREVIEW"))?;
            if p.preview_id != preview_id || time.elapsed() > Duration::from_secs(15 * 60) {
                return Err(error("TRELLO_PREVIEW"));
            }
            p.clone()
        };
        let unique: std::collections::HashSet<_> = selected.iter().collect();
        if selected.len() > 10000
            || unique.len() != selected.len()
            || selected.iter().any(|id| {
                !preview
                    .cards
                    .iter()
                    .any(|c| &c.card.id == id && !c.card.closed)
            })
        {
            return Err(error("TRELLO_PREVIEW"));
        }
        let credentials = WindowsVault.load()?.ok_or_else(|| error("TRELLO_AUTH"))?;
        let mut api = OfficialTransport::new(&credentials, limits.timeout_seconds)?;
        let mut job = repo(&app, |r| {
            r.trello_begin(&preview.board, selected.len() as u32)
        })?;
        state.update(&job);
        let data = repo(&app, |r| {
            r.storage
                .directory
                .parent()
                .map(Path::to_path_buf)
                .ok_or_else(AppError::file)
        })?;
        let result = import_cards(
            &mut api,
            ImportInput {
                data: &data,
                preview: &preview,
                selected: &selected,
                limits: &limits,
                cancel: &state.cancel,
            },
            &mut job,
            |op| repo(&app, |r| op(r)),
            &mut |j| state.update(j),
        );
        job.current_card = None;
        job.status = match result {
            Ok(()) => {
                if job.failed_cards > 0 || job.attachment_failures > 0 {
                    "partial"
                } else {
                    "completed"
                }
            }
            Err(e) => {
                job.error_message = Some(e.message.into());
                if e.code == "TRELLO_CANCELLED" {
                    "cancelled"
                } else {
                    "failed"
                }
            }
        }
        .into();
        state.update(&job);
        repo(&app, |r| r.trello_job_save(&job))?;
        *state.preview.lock().map_err(|_| error("TRELLO_BUSY"))? = None;
        Ok(job)
    })
    .await
}
