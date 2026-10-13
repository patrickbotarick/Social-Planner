mod database;
mod error;

use database::{FoundationStatus, Repository};
use error::AppError;
use std::sync::Mutex;
use tauri::Manager;

struct DatabaseState(Mutex<Result<Repository, AppError>>);

#[tauri::command]
async fn foundation_status(
    state: tauri::State<'_, DatabaseState>,
) -> Result<FoundationStatus, AppError> {
    let repository = state.0.lock().map_err(|_| AppError::database())?;
    match &*repository {
        Ok(repository) => repository.status(),
        Err(error) => Err(error.clone()),
    }
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let database = (|| -> Result<Repository, AppError> {
                let data_dir = app.path().app_data_dir().map_err(|_| AppError::database())?;
                std::fs::create_dir_all(&data_dir).map_err(|_| AppError::database())?;
                Repository::open(&data_dir.join("planner.sqlite3"))
            })();
            match &database {
                Ok(_) => eprintln!("[foundation] DATABASE_READY"),
                Err(error) => eprintln!("[foundation] {}", error.code),
            }
            // A janela permanece disponível para mostrar o erro, sem resetar dados.
            app.manage(DatabaseState(Mutex::new(database)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![foundation_status])
        .run(tauri::generate_context!())
        .expect("Não foi possível iniciar a janela desktop do GMAD Social Planner");
}
