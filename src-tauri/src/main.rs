#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app_paths;
mod download_manager;
mod engine_bridge;
mod job_manager;
mod model_registry;
mod models;
mod runtime_manager;

use app_paths::AppPaths;
use download_manager::{download_file, verify_sha256};
use engine_bridge::EngineBridge;
use model_registry::ModelRegistry;
use models::{
    EngineHealth, ModelCatalogEntry, ProcessAudioResponse, SeparationRequest, SetupStatus,
};
use runtime_manager::RuntimeManager;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{Manager, State};
use tokio::sync::Mutex;

struct AppState {
    runtime_manager: RuntimeManager,
    model_registry: Arc<ModelRegistry>,
    active_jobs: Arc<Mutex<HashMap<String, tokio::process::Child>>>,
}

#[tauri::command]
async fn cancel_job(state: State<'_, AppState>, job_id: String) -> Result<(), String> {
    let mut jobs = state.active_jobs.lock().await;
    if let Some(mut child) = jobs.remove(&job_id) {
        let _ = child.kill().await;
    }
    Ok(())
}

#[tauri::command]
async fn get_engine_health(state: State<'_, AppState>) -> Result<EngineHealth, String> {
    state
        .runtime_manager
        .doctor()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn prepare_engine(state: State<'_, AppState>) -> Result<SetupStatus, String> {
    state
        .runtime_manager
        .prepare()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_model_catalog(state: State<'_, AppState>) -> Result<Vec<ModelCatalogEntry>, String> {
    state
        .model_registry
        .load_catalog()
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn download_model(
    state: State<'_, AppState>,
    model_id: String,
) -> Result<ModelCatalogEntry, String> {
    let entry = state
        .model_registry
        .get_entry(&model_id)
        .map_err(|e| e.to_string())?;
    state
        .model_registry
        .validate_downloadable(&entry)
        .map_err(|e| e.to_string())?;

    std::fs::create_dir_all(&state.model_registry.models_dir).map_err(|e| e.to_string())?;

    let destination = state.model_registry.installed_model_path(&entry);
    let temp_destination = destination.with_extension("download");

    if let Err(e) = download_file(&entry.url, &temp_destination).await {
        let _ = std::fs::remove_file(&temp_destination);
        return Err(e.to_string());
    }

    if let Err(e) = verify_sha256(&temp_destination, &entry.sha256) {
        let _ = std::fs::remove_file(&temp_destination);
        return Err(e.to_string());
    }

    if let Err(e) = std::fs::rename(&temp_destination, &destination) {
        let _ = std::fs::remove_file(&temp_destination);
        return Err(e.to_string());
    }

    Ok(entry)
}

#[tauri::command]
async fn process_audio(
    state: State<'_, AppState>,
    file_path: String,
    model: String,
    output_dir: String,
    _quality: String,
) -> Result<ProcessAudioResponse, String> {
    let request = SeparationRequest {
        input_path: file_path.clone(),
        model_id: model.clone(),
        output_dir: output_dir.clone(),
        format: "wav".into(),
    };
    job_manager::validate_request(&request).map_err(|e| e.to_string())?;

    let entry = state
        .model_registry
        .get_entry(&model)
        .map_err(|e| e.to_string())?;
    let model_path = state.model_registry.installed_model_path(&entry);
    if !model_path.is_file() {
        return Err(format!(
            "Model `{}` is not installed yet. Download it from the Model Registry tab first.",
            entry.name
        ));
    }

    let python_exe = state.runtime_manager.paths().venv_python_executable();
    let engine_script = state.runtime_manager.paths().installed_engine_script();
    if !python_exe.is_file() {
        return Err(format!(
            "Engine runtime is not ready. Missing {}",
            python_exe.display()
        ));
    }
    if !engine_script.is_file() {
        return Err(format!(
            "Engine script is not ready. Missing {}",
            engine_script.display()
        ));
    }

    std::fs::create_dir_all(&output_dir).map_err(|e| e.to_string())?;

    let bridge = EngineBridge::new(python_exe, engine_script);
    let payload = serde_json::json!({
        "job_id": "job-local",
        "backend": entry.backend,
        "input_path": file_path,
        "model_path": model_path,
        "output_dir": output_dir,
    });

    let (events, mut child) = bridge
        .run_command_collect("separate", payload)
        .await
        .map_err(|e| e.to_string())?;
    let _ = child.wait().await;

    let terminal = events
        .last()
        .ok_or_else(|| "Engine returned no events".to_string())?;

    match terminal.event.as_str() {
        "result" => {
            let payload = terminal
                .payload
                .clone()
                .ok_or_else(|| "Engine result did not include payload".to_string())?;
            let vocals_path = payload
                .get("vocals_path")
                .and_then(|value| value.as_str())
                .ok_or_else(|| "Engine result missing vocals_path".to_string())?;
            let instrumental_path = payload
                .get("instrumental_path")
                .and_then(|value| value.as_str())
                .ok_or_else(|| "Engine result missing instrumental_path".to_string())?;

            Ok(ProcessAudioResponse {
                job_id: terminal
                    .job_id
                    .clone()
                    .unwrap_or_else(|| "job-local".into()),
                vocals_path: vocals_path.into(),
                instrumental_path: instrumental_path.into(),
                backend: entry.backend,
            })
        }
        "error" => Err(terminal
            .message
            .clone()
            .unwrap_or_else(|| "Engine returned an unspecified error".into())),
        other => Err(format!("Unexpected terminal engine event `{}`", other)),
    }
}

#[tauri::command]
async fn check_system(state: State<'_, AppState>) -> Result<String, String> {
    let health = state
        .runtime_manager
        .doctor()
        .await
        .map_err(|e| e.to_string())?;

    if health.runtime_ready && health.dependencies_ready {
        Ok("Sistema listo - PrismSplit".into())
    } else {
        Ok("Motor no preparado - ejecute Prepare Engine".into())
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to get app data dir");
            let resource_dir = app
                .path()
                .resource_dir()
                .expect("failed to get resource dir");
            let paths = AppPaths::new(app_data_dir, resource_dir);
            let runtime_manager = RuntimeManager::new(paths.clone());
            let model_registry = Arc::new(ModelRegistry::new(
                paths.models_dir.clone(),
                paths.manifest_catalog_path(),
            ));

            let active_jobs = Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));

            app.manage(AppState {
                runtime_manager,
                model_registry,
                active_jobs,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_model_catalog,
            download_model,
            process_audio,
            check_system,
            get_engine_health,
            prepare_engine,
            cancel_job
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
