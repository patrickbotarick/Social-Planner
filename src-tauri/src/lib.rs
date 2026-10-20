mod trello_api;
mod trello_domain;
mod trello_redirect;
mod trello_repository;
mod trello_service;
#[cfg(test)]
mod trello_tests;
mod trello_vault;
use trello_service::*;
mod database;
mod editorial_domain;
mod editorial_repository;
#[cfg(test)]
mod editorial_tests;
mod error;
mod media_store;
mod review;
#[cfg(test)]
mod review_tests;
use review::*;
mod calendar;
use calendar::*;
mod assisted;
mod assisted_files;
mod backup;
#[cfg(test)]
mod backup_tests;
#[cfg(test)]
mod calendar_tests;
use assisted::*;
#[cfg(test)]
mod assisted_tests;

#[tauri::command]
async fn assisted_list(
    app: tauri::AppHandle,
    query: AssistedQuery,
) -> Result<AssistedPage, AppError> {
    with_repository(app, move |r| r.assisted_list(query)).await
}
#[tauri::command]
async fn assisted_prepare(
    app: tauri::AppHandle,
    targets: Vec<PreparationTarget>,
) -> Result<Vec<PreparationResult>, AppError> {
    with_repository(app, move |r| r.assisted_prepare(targets)).await
}
#[tauri::command]
async fn assisted_confirm(
    app: tauri::AppHandle,
    input: ConfirmationInput,
) -> Result<String, AppError> {
    with_repository(app, move |r| r.assisted_confirm(input)).await
}
#[tauri::command]
async fn assisted_reconcile(app: tauri::AppHandle, input: ReconcileInput) -> Result<(), AppError> {
    with_repository(app, move |r| r.assisted_reconcile(input)).await
}
#[tauri::command]
async fn assisted_history(app: tauri::AppHandle, offset: u32) -> Result<AssistedHistory, AppError> {
    with_repository(app, move |r| r.assisted_history(offset)).await
}
#[tauri::command]
async fn assisted_caption(app: tauri::AppHandle, id: String) -> Result<String, AppError> {
    with_repository(app, move |r| r.assisted_caption(&id)).await
}
async fn assisted_system_open(app: tauri::AppHandle, target: String) -> Result<(), AppError> {
    let (sender, receiver) = std::sync::mpsc::channel();
    app.run_on_main_thread(move || {
        let _ = sender.send(assisted_files::system_open(&target));
    })
    .map_err(|_| AppError::file())?;
    tauri::async_runtime::spawn_blocking(move || receiver.recv().map_err(|_| AppError::file())?)
        .await
        .map_err(|_| AppError::file())?
}
#[tauri::command]
async fn assisted_folder_open(app: tauri::AppHandle, id: String) -> Result<(), AppError> {
    let path = with_repository(app.clone(), move |r| r.assisted_folder(&id)).await?;
    assisted_system_open(app, path).await
}
#[tauri::command]
async fn assisted_business_open(app: tauri::AppHandle) -> Result<(), AppError> {
    assisted_system_open(app, "https://business.facebook.com/".into()).await
}
#[tauri::command]
async fn assisted_choose_root(app: tauri::AppHandle) -> Result<Option<String>, AppError> {
    let picker = app.clone();
    let selected = tauri::async_runtime::spawn_blocking(move || {
        picker
            .dialog()
            .file()
            .set_title("Escolha a pasta local de exportações GMAD")
            .blocking_pick_folder()
    })
    .await
    .map_err(|_| AppError::file())?;
    let Some(selected) = selected else {
        return Ok(None);
    };
    let path = selected.into_path().map_err(|_| AppError::file())?;
    with_repository(app, move |r| r.assisted_choose_root(&path).map(Some)).await
}

use database::FoundationStatus;
use editorial_domain::*;
use editorial_repository::EditorialRepository;
use error::AppError;
use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

struct DatabaseState(Mutex<Result<EditorialRepository, AppError>>);
struct ProfileLock(#[allow(dead_code)] std::fs::File);
#[derive(Default)]
struct BackupSelection(Mutex<Option<backup::Inspection>>);
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct BackupInspection {
    ticket: String,
    report: backup::BackupReport,
}

async fn backup_picker(
    app: tauri::AppHandle,
    title: &'static str,
) -> Result<Option<std::path::PathBuf>, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut picker = app.dialog().file().set_title(title);
        if let Some(parent) = app.get_webview_window("main") {
            picker = picker.set_parent(&parent);
        }
        picker
            .blocking_pick_folder()
            .map(|f| f.into_path().map_err(|_| backup::invalid()))
            .transpose()
    })
    .await
    .map_err(|_| backup::io())?
}
#[tauri::command]
async fn backup_create(app: tauri::AppHandle) -> Result<Option<backup::BackupReport>, AppError> {
    let Some(path) = backup_picker(app.clone(), "Escolha o destino do novo backup GMAD").await?
    else {
        return Ok(None);
    };

    tauri::async_runtime::spawn_blocking(move || {
        let trello = app.state::<TrelloState>();
        let _operation = trello.enter()?;

        let state = app.state::<DatabaseState>();
        let lock = state.0.lock().map_err(|_| backup::busy())?;

        backup::create(lock.as_ref().map_err(Clone::clone)?, &path).map(Some)
    })
    .await
    .map_err(|_| backup::io())?
}
#[tauri::command]
async fn backup_inspect(app: tauri::AppHandle) -> Result<Option<BackupInspection>, AppError> {
    let Some(path) =
        backup_picker(app.clone(), "Selecione a pasta GMAD-Backup a inspecionar").await?
    else {
        return Ok(None);
    };
    tauri::async_runtime::spawn_blocking(move || {
        let report = backup::inspect(&path)?;
        let ticket = uuid::Uuid::new_v4().to_string();
        *app.state::<BackupSelection>()
            .0
            .lock()
            .map_err(|_| backup::busy())? = Some(backup::Inspection {
            path: std::path::PathBuf::from(&report.path),
            fingerprint: report.manifest_sha256.clone(),
            id: ticket.clone(),
        });
        Ok(Some(BackupInspection { ticket, report }))
    })
    .await
    .map_err(|_| backup::io())?
}
#[tauri::command]
async fn backup_restore(
    app: tauri::AppHandle,
    ticket: String,
    confirmed: bool,
) -> Result<backup::RestoreReport, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        let trello = app.state::<TrelloState>();
        let _operation = trello.enter()?;
        let selection = app.state::<BackupSelection>();
        let mut chosen = selection.0.lock().map_err(|_| backup::busy())?;
        let selected = chosen
            .as_ref()
            .filter(|s| s.id == ticket)
            .ok_or_else(backup::invalid)?;
        let database = app.state::<DatabaseState>();
        let mut lock = database.0.lock().map_err(|_| backup::busy())?;
        let mut report = backup::restore(&mut lock, selected, confirmed)?;
        if let Ok(r) = &*lock {
            report.restart_required = app
                .asset_protocol_scope()
                .allow_directory(&r.storage.staging_directory, false)
                .is_err();
        }
        trello.reset_after_restore();
        *chosen = None;
        Ok(report)
    })
    .await
    .map_err(|_| backup::io())?
}

#[tauri::command]
async fn calendar_list(
    app: tauri::AppHandle,
    query: CalendarQuery,
) -> Result<CalendarPage, AppError> {
    with_repository(app, move |r| r.calendar_list(query)).await
}
#[tauri::command]
async fn calendar_available(
    app: tauri::AppHandle,
    query: AvailableQuery,
) -> Result<AvailablePage, AppError> {
    with_repository(app, move |r| r.calendar_available(query)).await
}
#[tauri::command]
async fn schedule_save(app: tauri::AppHandle, input: ScheduleInput) -> Result<String, AppError> {
    with_repository(app, move |r| r.schedule_save(input)).await
}
#[tauri::command]
async fn schedule_cancel(
    app: tauri::AppHandle,
    id: String,
    expected_version: u32,
) -> Result<(), AppError> {
    with_repository(app, move |r| r.schedule_cancel(&id, expected_version)).await
}
#[tauri::command]
async fn schedule_settings_save(
    app: tauri::AppHandle,
    input: SettingsInput,
) -> Result<(), AppError> {
    with_repository(app, move |r| r.schedule_settings_save(input)).await
}
#[tauri::command]
async fn planning_dashboard(
    app: tauri::AppHandle,
    timezone: String,
) -> Result<PlanningDashboard, AppError> {
    with_repository(app, move |r| r.planning_dashboard(&timezone)).await
}
#[tauri::command]
async fn schedule_history(app: tauri::AppHandle, offset: u32) -> Result<PlanningHistory, AppError> {
    with_repository(app, move |r| r.schedule_history(offset)).await
}

#[tauri::command]
async fn review_list(app: tauri::AppHandle, query: ReviewQuery) -> Result<ReviewPage, AppError> {
    with_repository(app, move |r| r.review_list(query)).await
}
#[tauri::command]
async fn review_get(app: tauri::AppHandle, id: String) -> Result<ReviewItem, AppError> {
    with_repository(app, move |r| r.review_item(&id, true)).await
}
#[tauri::command]
async fn review_decide(
    app: tauri::AppHandle,
    input: ReviewDecision,
) -> Result<Vec<DecisionResult>, AppError> {
    with_repository(app, move |r| r.review_decide(input)).await
}
#[tauri::command]
async fn review_history(
    app: tauri::AppHandle,
    query: HistoryQuery,
) -> Result<HistoryPage, AppError> {
    with_repository(app, move |r| r.review_history(query)).await
}

#[tauri::command]
async fn foundation_status(
    state: tauri::State<'_, DatabaseState>,
) -> Result<FoundationStatus, AppError> {
    let repository = state.0.lock().map_err(|_| AppError::database())?;
    match &*repository {
        Ok(repository) => repository.database.status(),
        Err(error) => Err(error.clone()),
    }
}

async fn with_repository<T: Send + 'static>(
    app: tauri::AppHandle,
    operation: impl FnOnce(&mut EditorialRepository) -> Result<T, AppError> + Send + 'static,
) -> Result<T, AppError> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<DatabaseState>();
        let mut locked = state.0.lock().map_err(|_| AppError::database())?;
        match &mut *locked {
            Ok(repository) => operation(repository),
            Err(error) => Err(error.clone()),
        }
    })
    .await
    .map_err(|_| AppError::database())?
}

#[tauri::command]
async fn publication_list(
    app: tauri::AppHandle,
    query: LibraryQuery,
) -> Result<LibraryPage, AppError> {
    with_repository(app, move |r| r.list(query)).await
}
#[tauri::command]
async fn publication_get(app: tauri::AppHandle, id: String) -> Result<Publication, AppError> {
    with_repository(app, move |r| r.get(&id)).await
}
#[tauri::command]
async fn publication_save(
    app: tauri::AppHandle,
    input: SavePublication,
) -> Result<Publication, AppError> {
    with_repository(app, move |r| r.save(input)).await
}
#[tauri::command]
async fn publication_duplicate(app: tauri::AppHandle, id: String) -> Result<Publication, AppError> {
    with_repository(app, move |r| r.duplicate(&id)).await
}
#[tauri::command]
async fn publication_delete(
    app: tauri::AppHandle,
    id: String,
    expected_version: u32,
) -> Result<(), AppError> {
    with_repository(app, move |r| r.delete(&id, expected_version)).await
}
#[tauri::command]
async fn editorial_counts(app: tauri::AppHandle) -> Result<EditorialCounts, AppError> {
    with_repository(app, |r| r.counts()).await
}
#[tauri::command]
async fn media_discard(app: tauri::AppHandle, ids: Vec<String>) -> Result<(), AppError> {
    if ids.len() > 20 {
        return Err(AppError::validation());
    }
    with_repository(app, move |r| r.storage.discard(&ids)).await
}
#[tauri::command]
async fn media_select(
    app: tauri::AppHandle,
    format: PublicationFormat,
) -> Result<ImportResult, AppError> {
    let chooser = app.clone();
    let files = tauri::async_runtime::spawn_blocking(move || {
        let mut dialog = chooser
            .dialog()
            .file()
            .set_title("Selecionar mídias da publicação")
            .add_filter("Imagens e vídeos", &["jpg", "jpeg", "png", "webp", "mp4"]);
        if let Some(parent) = chooser.get_webview_window("main") {
            dialog = dialog.set_parent(&parent);
        }
        if format == PublicationFormat::Carousel {
            dialog.blocking_pick_files().unwrap_or_default()
        } else {
            dialog.blocking_pick_file().into_iter().collect()
        }
    })
    .await
    .map_err(|_| AppError::file())?;
    if files.len() > 20 {
        return Err(AppError::validation());
    }
    with_repository(app, move |r| {
        let mut result = ImportResult {
            media: Vec::new(),
            issues: Vec::new(),
        };
        for file in files {
            let path = file.into_path().map_err(|_| AppError::file())?;
            let filename = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            match r.storage.stage(&path) {
                Ok(media) if format.accepts(&media.mime_type) => result.media.push(media),
                Ok(media) => {
                    r.storage.discard(&[media.id])?;
                    result.issues.push(ImportIssue {
                        filename,
                        message: AppError::validation().message,
                    });
                }
                Err(error) => result.issues.push(ImportIssue {
                    filename,
                    message: error.message,
                }),
            }
        }
        Ok(result)
    })
    .await
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let database = (|| -> Result<EditorialRepository, AppError> {
                let mut data_dir = app
                    .path()
                    .app_data_dir()
                    .map_err(|_| AppError::database())?;
                // QA opt-in exclusivo de builds debug; não altera o perfil operacional.
                if cfg!(debug_assertions) {
                    if let Some(path) = std::env::var_os("GMAD_QA_DATA_DIR") {
                        data_dir = path.into();
                        if !data_dir.is_absolute() {
                            return Err(AppError::file());
                        }
                    }
                }
                std::fs::create_dir_all(data_dir.parent().ok_or_else(AppError::file)?)
                    .map_err(|_| AppError::database())?;
                // Normaliza a identidade antes do lock/journal, inclusive quando o diretório estiver ausente.
                data_dir = data_dir
                    .parent()
                    .ok_or_else(AppError::file)?
                    .canonicalize()
                    .map_err(|_| AppError::file())?
                    .join(data_dir.file_name().ok_or_else(AppError::file)?);
                app.manage(ProfileLock(backup::profile_lock(&data_dir)?));
                backup::recover(&data_dir)?;
                std::fs::create_dir_all(&data_dir).map_err(|_| AppError::database())?;
                let repository = EditorialRepository::open(&data_dir)?;
                repository.trello_recover()?;
                app.asset_protocol_scope()
                    .allow_directory(&repository.storage.directory, false)
                    .map_err(|_| AppError::file())?;
                app.asset_protocol_scope()
                    .allow_directory(&repository.storage.staging_directory, false)
                    .map_err(|_| AppError::file())?;
                Ok(repository)
            })();
            match &database {
                Ok(_) => eprintln!("[foundation] DATABASE_READY"),
                Err(error) => eprintln!("[foundation] {}", error.code),
            }
            // A janela permanece disponível para mostrar o erro, sem resetar dados.
            app.manage(DatabaseState(Mutex::new(database)));
            app.manage(TrelloState::default());
            app.manage(BackupSelection::default());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            backup_create,
            backup_inspect,
            backup_restore,
            assisted_list,
            assisted_prepare,
            assisted_confirm,
            assisted_reconcile,
            assisted_history,
            assisted_caption,
            assisted_folder_open,
            assisted_business_open,
            assisted_choose_root,
            calendar_list,
            calendar_available,
            schedule_save,
            schedule_cancel,
            schedule_settings_save,
            planning_dashboard,
            schedule_history,
            review_list,
            review_get,
            review_decide,
            review_history,
            foundation_status,
            publication_list,
            publication_get,
            publication_save,
            publication_duplicate,
            publication_delete,
            editorial_counts,
            media_select,
            media_discard,
            trello_status,
            trello_connect,
            trello_test,
            trello_disconnect,
            trello_authorization_url,
            trello_preview,
            trello_history,
            trello_source,
            trello_cancel,
            trello_import,
        ])
        .run(tauri::generate_context!())
        .expect("Não foi possível iniciar a janela desktop do GMAD Social Planner");
}
