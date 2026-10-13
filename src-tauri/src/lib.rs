mod catalog;
pub mod runner;
mod storage;

use std::{
    path::Path,
    sync::{Arc, Mutex},
};

use rusqlite::Connection;
use tauri::{Manager, State};

struct AppState {
    catalog: Mutex<Result<Connection, String>>,
    learner: Mutex<Result<Connection, String>>,
    runner: Arc<runner::Runner>,
}

#[tauri::command]
async fn toolchain_status(state: State<'_, AppState>) -> Result<runner::Compiler, String> {
    let runner = state.runner.clone();
    tauri::async_runtime::spawn_blocking(move || runner.toolchain())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_compiler_path(
    path: String,
    state: State<'_, AppState>,
) -> Result<runner::Compiler, String> {
    let runner = state.runner.clone();
    tauri::async_runtime::spawn_blocking(move || runner.configure_compiler(&path))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
async fn start_run(
    request: runner::RunRequest,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let runner = state.runner.clone();
    tauri::async_runtime::spawn_blocking(move || runner.start_open(request))
        .await
        .map_err(|e| e.to_string())?
}

#[tauri::command]
fn get_run(job_id: String, state: State<'_, AppState>) -> Result<runner::JobSnapshot, String> {
    state.runner.snapshot(&job_id)
}

#[tauri::command]
fn cancel_run(job_id: String, state: State<'_, AppState>) -> Result<(), String> {
    state.runner.cancel(&job_id)
}

#[tauri::command]
fn catalog_overview(state: State<'_, AppState>) -> Result<catalog::CatalogOverview, String> {
    let connection = state
        .catalog
        .lock()
        .map_err(|_| "Catalog storage is unavailable after an internal error.".to_owned())?;
    catalog::overview(connection.as_ref().map_err(Clone::clone)?)
}

#[tauri::command]
fn search_catalog(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<catalog::SearchHit>, String> {
    let connection = state
        .catalog
        .lock()
        .map_err(|_| "Catalog storage is unavailable after an internal error.".to_owned())?;
    catalog::search(connection.as_ref().map_err(Clone::clone)?, &query)
}

#[tauri::command]
fn get_catalog_item(
    id: String,
    state: State<'_, AppState>,
) -> Result<Option<catalog::CatalogItemDetail>, String> {
    let connection = state
        .catalog
        .lock()
        .map_err(|_| "Catalog storage is unavailable after an internal error.".to_owned())?;
    catalog::item(connection.as_ref().map_err(Clone::clone)?, &id)
}

#[tauri::command]
fn list_lexicon(state: State<'_, AppState>) -> Result<Vec<catalog::LexiconEntry>, String> {
    let connection = state
        .catalog
        .lock()
        .map_err(|_| "Catalog storage is unavailable after an internal error.".to_owned())?;
    catalog::lexicon(connection.as_ref().map_err(Clone::clone)?)
}

#[tauri::command]
fn get_open_lab_draft(state: State<'_, AppState>) -> Result<Option<storage::Draft>, String> {
    let connection = state
        .learner
        .lock()
        .map_err(|_| "Learner storage is unavailable after an internal error.".to_owned())?;
    storage::get_draft(connection.as_ref().map_err(Clone::clone)?)
}

#[tauri::command]
fn save_open_lab_draft(
    source: String,
    language: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let connection = state
        .learner
        .lock()
        .map_err(|_| "Learner storage is unavailable after an internal error.".to_owned())?;
    storage::save_draft(
        connection.as_ref().map_err(Clone::clone)?,
        &source,
        &language,
    )
}

#[tauri::command]
fn import_browser_draft(source: String, state: State<'_, AppState>) -> Result<bool, String> {
    let connection = state
        .learner
        .lock()
        .map_err(|_| "Learner storage is unavailable after an internal error.".to_owned())?;
    storage::import_browser_draft(connection.as_ref().map_err(Clone::clone)?, &source)
}

pub fn run() {
    tauri::Builder::default()
        .on_window_event(|window, event| {
            if matches!(event, tauri::WindowEvent::CloseRequested { .. }) {
                window.state::<AppState>().runner.cancel_all();
            }
        })
        .setup(|app| {
            let resource_root = app
                .path()
                .resource_dir()
                .map(|path| path.join("course-data"));
            let resource_root = resource_root
                .as_deref()
                .map_err(|error| format!("Could not locate bundled course data: {error}"));
            let data_path = match std::env::var_os("CLAB_DATA_DIR") {
                Some(path) => Ok(std::path::PathBuf::from(path)),
                None => app.path().app_data_dir(),
            };
            let job_root = data_path
                .as_ref()
                .map(|p| p.join("jobs"))
                .unwrap_or_else(|_| std::env::temp_dir().join("machine-room-jobs"));
            let (catalog, learner) = match data_path {
                Ok(data_dir) => match std::fs::create_dir_all(&data_dir) {
                    Ok(()) => {
                        let catalog_result = initialize_catalog(
                            &data_dir.join("catalog.sqlite"),
                            resource_root.clone(),
                        );
                        let learner_result = initialize_learner(&data_dir.join("learner.sqlite"));
                        (catalog_result, learner_result)
                    }
                    Err(error) => {
                        let message = format!("Could not create app data directory: {error}");
                        (Err(message.clone()), Err(message))
                    }
                },
                Err(error) => {
                    let message = format!("Could not resolve app data directory: {error}");
                    (Err(message.clone()), Err(message))
                }
            };
            app.manage(AppState {
                catalog: Mutex::new(catalog),
                learner: Mutex::new(learner),
                runner: Arc::new(runner::Runner::new(job_root)),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            catalog_overview,
            search_catalog,
            get_catalog_item,
            list_lexicon,
            get_open_lab_draft,
            save_open_lab_draft,
            import_browser_draft,
            toolchain_status,
            set_compiler_path,
            start_run,
            get_run,
            cancel_run,
        ])
        .run(tauri::generate_context!())
        .expect("failed to start Machine Room");
}

fn initialize_catalog(
    path: &Path,
    resource_root: Result<&Path, String>,
) -> Result<Connection, String> {
    let resource_root = resource_root?;
    let mut connection = storage::open(path)?;
    catalog::sync(&mut connection, resource_root)?;
    Ok(connection)
}

fn initialize_learner(path: &Path) -> Result<Connection, String> {
    let mut connection = storage::open(path)?;
    storage::migrate_learner(&mut connection)?;
    Ok(connection)
}
