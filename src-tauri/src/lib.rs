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
#[cfg(test)]
mod calendar_tests;

use database::FoundationStatus;
use editorial_domain::*;
use editorial_repository::EditorialRepository;
use error::AppError;
use std::sync::Mutex;
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

struct DatabaseState(Mutex<Result<EditorialRepository, AppError>>);

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
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
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
