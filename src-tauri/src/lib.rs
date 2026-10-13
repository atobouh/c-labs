mod catalog;
mod learning;
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
    learner: learning::Learner,
    runner: Arc<runner::Runner>,
    resources: Result<std::path::PathBuf, String>,
}

#[tauri::command]
fn get_pilot(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let connection = state.catalog.lock().map_err(|e| e.to_string())?;
    learning::pilot(
        connection.as_ref().map_err(Clone::clone)?,
        state.resources.as_deref().map_err(Clone::clone)?,
    )
}
#[tauri::command]
fn get_learning_state(state: State<'_, AppState>) -> Result<serde_json::Value, String> {
    let connection = state.learner.lock().map_err(|e| e.to_string())?;
    learning::state(connection.as_ref().map_err(Clone::clone)?)
}
#[tauri::command]
fn get_lesson_example(
    id: String,
    state: State<'_, AppState>,
) -> Result<Vec<runner::SourceFile>, String> {
    let connection = state.catalog.lock().map_err(|e| e.to_string())?;
    learning::example(
        connection.as_ref().map_err(Clone::clone)?,
        state.resources.as_deref().map_err(Clone::clone)?,
        &id,
    )
}
#[tauri::command]
fn save_learning_progress(
    id: String,
    value: serde_json::Value,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let connection = state.learner.lock().map_err(|e| e.to_string())?;
    learning::save_progress(connection.as_ref().map_err(Clone::clone)?, &id, &value)
}
fn learning_package(state: &AppState, id: &str) -> Result<learning::Package, String> {
    let mut connection = state.catalog.lock().map_err(|e| e.to_string())?;
    let root = state.resources.as_deref().map_err(Clone::clone)?;
    let connection = connection.as_mut().map_err(|e| e.clone())?;
    catalog::sync(connection, root)?;
    learning::load_package(connection, root, id)
}
#[tauri::command]
fn get_learning_package(
    id: String,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let package = learning_package(&state, &id)?;
    let connection = state.learner.lock().map_err(|e| e.to_string())?;
    learning::package_view(connection.as_ref().map_err(Clone::clone)?, &package)
}
#[tauri::command]
fn save_package_draft(
    id: String,
    files: Vec<runner::SourceFile>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let package = learning_package(&state, &id)?;
    let connection = state.learner.lock().map_err(|e| e.to_string())?;
    learning::save_submission(connection.as_ref().map_err(Clone::clone)?, &package, &files)
}
#[tauri::command]
async fn start_grade(
    id: String,
    files: Vec<runner::SourceFile>,
    self_review: bool,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let package = learning_package(&state, &id)?;
    {
        let connection = state.learner.lock().map_err(|e| e.to_string())?;
        learning::save_submission(connection.as_ref().map_err(Clone::clone)?, &package, &files)?;
    }
    let runner = state.runner.clone();
    let learner = state.learner.clone();
    tauri::async_runtime::spawn_blocking(move || {
        learning::start_grade(runner, learner, package, files, self_review)
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
fn submit_exercise(
    id: String,
    answer: serde_json::Value,
    hint_level: u32,
    state: State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let record = {
        let connection = state.catalog.lock().map_err(|e| e.to_string())?;
        catalog::item(connection.as_ref().map_err(Clone::clone)?, &id)?
            .ok_or("Unknown exercise.")?
    };
    let mut connection = state.learner.lock().map_err(|e| e.to_string())?;
    learning::exercise(
        connection.as_mut().map_err(|e| e.clone())?,
        &record,
        answer,
        hint_level,
    )
}
#[tauri::command]
fn mark_reviewed(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let connection = state.learner.lock().map_err(|e| e.to_string())?;
    learning::review(connection.as_ref().map_err(Clone::clone)?, &id)
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
            let resources = resource_root.clone().map(Path::to_owned);
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
                learner: Arc::new(Mutex::new(learner)),
                runner: Arc::new(runner::Runner::new(job_root)),
                resources,
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
            get_pilot,
            get_lesson_example,
            get_learning_state,
            save_learning_progress,
            get_learning_package,
            save_package_draft,
            start_grade,
            submit_exercise,
            mark_reviewed,
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
    connection.execute("UPDATE attempts SET result_json=json_set(result_json,'$.status','interrupted','$.message','The application closed before this attempt completed. Run the saved draft again.') WHERE json_extract(result_json,'$.status')='running'",[]).map_err(|e|e.to_string())?;
    Ok(connection)
}
