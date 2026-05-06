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
use download_manager::{download_file_with_progress, verify_sha256};
use engine_bridge::EngineBridge;
use model_registry::ModelRegistry;
use models::{
    AppConfig, DownloadProgressEvent, EngineHealth, ModelCatalogEntry, ProcessAudioResponse,
    SeparationRequest, SetupStatus,
};
use runtime_manager::RuntimeManager;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::{Emitter, Manager, State};
use tokio::sync::Mutex;

struct AppState {
    runtime_manager: Arc<RuntimeManager>,
    model_registry: Arc<ModelRegistry>,
    active_jobs: Arc<Mutex<HashMap<String, tokio::process::Child>>>,
    config_path: PathBuf,
}

fn load_config(path: &Path) -> AppConfig {
    if let Ok(content) = std::fs::read_to_string(path) {
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        AppConfig::default()
    }
}

fn save_config(path: &Path, config: &AppConfig) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(path, content).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    Ok(load_config(&state.config_path))
}

#[tauri::command]
async fn update_config(state: State<'_, AppState>, config: AppConfig) -> Result<(), String> {
    let old_config = load_config(&state.config_path);
    let models_dir_changed = config.models_dir != old_config.models_dir;

    save_config(&state.config_path, &config)?;

    // If models directory changed, we should probably update the active paths
    // and scan the new directory automatically.
    if models_dir_changed {
        if let Some(new_path) = &config.models_dir {
            // Update the model registry's internal path for the current session dynamically
            state.model_registry.set_models_dir(PathBuf::from(new_path));

            // Trigger an auto-scan of the new directory
            let _ = scan_local_models(state, new_path.clone()).await;
        }
    }

    Ok(())
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
    app_handle: tauri::AppHandle,
    model_id: String,
) -> Result<ModelCatalogEntry, String> {
    println!("[INFO] Starting download for model: {}", model_id);
    let entry = state
        .model_registry
        .get_entry(&model_id)
        .map_err(|e| e.to_string())?;
    state
        .model_registry
        .validate_downloadable(&entry)
        .map_err(|e| e.to_string())?;

    {
        let dir = state
            .model_registry
            .models_dir
            .lock()
            .map_err(|_| "Failed to lock models_dir".to_string())?;
        println!("[INFO] Creating models directory: {}", dir.display());
        std::fs::create_dir_all(&*dir).map_err(|e| e.to_string())?;
    }

    let destination = state.model_registry.installed_model_path(&entry);
    let temp_destination = destination.with_extension("download");

    println!("[INFO] Downloading to: {}", temp_destination.display());

    if let Err(e) = download_file_with_progress(&entry.url, &temp_destination, {
        let app_handle = app_handle.clone();
        let model_id = model_id.clone();
        move |dl, total| {
            let progress = (dl as f32 / total as f32) * 100.0;
            let _ = app_handle.emit(
                "download_progress",
                DownloadProgressEvent {
                    model_id: model_id.clone(),
                    progress,
                },
            );
        }
    })
    .await
    {
        println!("[ERROR] Download failed: {}", e);
        let _ = std::fs::remove_file(&temp_destination);
        return Err(e.to_string());
    }

    if let Err(e) = verify_sha256(&temp_destination, &entry.sha256) {
        println!("[ERROR] SHA256 verification failed: {}", e);
        let _ = std::fs::remove_file(&temp_destination);
        return Err(e.to_string());
    }

    if let Err(e) = std::fs::copy(&temp_destination, &destination) {
        println!("[ERROR] Failed to copy model file: {}", e);
        let _ = std::fs::remove_file(&temp_destination);
        return Err(e.to_string());
    }
    let _ = std::fs::remove_file(&temp_destination);

    println!(
        "[INFO] Download complete and verified: {}",
        destination.display()
    );
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

    let model_path = if let Some(lp) = &entry.local_path {
        PathBuf::from(lp)
    } else {
        state.model_registry.installed_model_path(&entry)
    };

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

#[tauri::command]
async fn sync_uvr_catalog(state: State<'_, AppState>) -> Result<usize, String> {
    let client = reqwest::Client::builder()
        .user_agent("PrismSplit/0.1.0")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;
    let url = "https://raw.githubusercontent.com/TRvlvr/application_data/main/filelists/download_checks.json";
    let response = client.get(url).send().await.map_err(|e| e.to_string())?;
    let data: serde_json::Value = response.json().await.map_err(|e| e.to_string())?;

    let mut new_entries = Vec::new();
    let lists = [
        ("mdx_download_list", "mdx"),
        ("vr_download_list", "vr"),
        ("mdx23_download_list", "mdx"),
        ("mdx23c_download_list", "mdx"),
        ("roformer_download_list", "mdx"),
    ];

    for (list_key, backend) in lists {
        if let Some(list) = data.get(list_key).and_then(|v| v.as_object()) {
            for (name, filename) in list {
                if let Some(filename_str) = filename.as_str() {
                    let id = format!("{}_{}", backend, filename_str.replace(".", "_"));
                    new_entries.push(ModelCatalogEntry {
                        id,
                        name: name.replace("MDX-Net Model: ", "").replace("VR Arch Single Model v5: ", "").replace("VR Arch Single Model v4: ", ""),
                        backend: backend.into(),
                        output_kind: "vocals_instrumental".into(),
                        url: format!("https://github.com/TRvlvr/model_repo/releases/download/all_public_uvr_models/{}", filename_str),
                        sha256: "replace-with-real-sha256".into(),
                        size_bytes: 0,
                        filename: filename_str.into(),
                        version: "1.0.0".into(),
                        is_installed: false,
                        local_path: None,
                    });
                }
            }
        }
    }

    // Special handling for Demucs
    if let Some(demucs_list) = data.get("demucs_download_list").and_then(|v| v.as_object()) {
        for (name, files) in demucs_list {
            if let Some(files_obj) = files.as_object() {
                if let Some((filename, url)) = files_obj.iter().find(|(k, _)| k.ends_with(".th")) {
                    if let Some(url_str) = url.as_str() {
                        let id = format!("demucs_{}", filename.replace(".", "_"));
                        new_entries.push(ModelCatalogEntry {
                            id,
                            name: name
                                .replace("Demucs v4: ", "")
                                .replace("Demucs v3: ", "")
                                .replace("Demucs v2: ", "")
                                .replace("Demucs v1: ", ""),
                            backend: "demucs".into(),
                            output_kind: "stems".into(),
                            url: url_str.into(),
                            sha256: "replace-with-real-sha256".into(),
                            size_bytes: 0,
                            filename: filename.into(),
                            version: "1.0.0".into(),
                            is_installed: false,
                            local_path: None,
                        });
                    }
                }
            }
        }
    }

    let mut current_catalog = state.model_registry.load_catalog().unwrap_or_default();

    use futures_util::stream::{self, StreamExt};

    let new_entries = Arc::new(new_entries);
    let current_catalog_ids: Vec<String> = current_catalog.iter().map(|e| e.id.clone()).collect();
    let current_catalog_filenames: Vec<String> =
        current_catalog.iter().map(|e| e.filename.clone()).collect();

    // Identify which entries need size fetching (new or existing with 0 size)
    let mut entries_to_process = Vec::new();

    // Add new ones
    for entry in new_entries.iter() {
        if !current_catalog_ids.contains(&entry.id)
            && !current_catalog_filenames.contains(&entry.filename)
        {
            entries_to_process.push(entry.clone());
        }
    }

    // Check existing ones with 0 size (limit to avoid too many requests)
    let mut fix_count = 0;
    for entry in &mut current_catalog {
        if entry.size_bytes == 0 && !entry.url.is_empty() && fix_count < 100 {
            entries_to_process.push(entry.clone());
            fix_count += 1;
        }
    }

    let client_ref = client.clone();

    let fetched_entries = stream::iter(entries_to_process)
        .map(|mut entry| {
            let client = client_ref.clone();
            async move {
                if let Ok(resp) = client.head(&entry.url).send().await {
                    if let Some(len) = resp.content_length() {
                        entry.size_bytes = len;
                    }
                }
                entry
            }
        })
        .buffer_unordered(10)
        .collect::<Vec<_>>()
        .await;

    let mut added_count = 0;
    for entry in fetched_entries {
        if let Some(existing) = current_catalog.iter_mut().find(|e| e.id == entry.id) {
            existing.size_bytes = entry.size_bytes;
        } else {
            current_catalog.push(entry);
            added_count += 1;
        }
    }

    state
        .model_registry
        .save_catalog(&current_catalog)
        .map_err(|e| e.to_string())?;

    Ok(added_count)
}

#[tauri::command]
async fn scan_local_models(state: State<'_, AppState>, path: String) -> Result<usize, String> {
    println!("[INFO] Scanning local directory: {}", path);
    let scan_path = Path::new(&path);
    if !scan_path.is_dir() {
        return Err(format!("Path `{}` is not a valid directory", path));
    }

    let mdx_data_url = "https://raw.githubusercontent.com/TRvlvr/application_data/main/mdx_model_data/model_data_new.json";
    let vr_data_url = "https://raw.githubusercontent.com/TRvlvr/application_data/main/vr_model_data/model_data_new.json";

    let client = reqwest::Client::builder()
        .user_agent("PrismSplit/0.1.0")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    println!("[INFO] Fetching remote model metadata for identification...");
    let mdx_data: serde_json::Value = client
        .get(mdx_data_url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .unwrap_or_default();
    let vr_data: serde_json::Value = client
        .get(vr_data_url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .unwrap_or_default();

    let mut added_count = 0;
    let mut current_catalog = state.model_registry.load_catalog().unwrap_or_default();

    for entry in walkdir::WalkDir::new(scan_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let file_path = entry.path();
        if file_path.is_file() {
            let ext = file_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_lowercase();
            if ext == "onnx" || ext == "pth" || ext == "th" || ext == "ckpt" {
                println!("[DEBUG] Found model candidate: {}", file_path.display());
                if let Ok(hash) = download_manager::md5_file(file_path) {
                    println!("[DEBUG]   Hash: {}", hash);
                    let mut model_name = file_path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Unknown")
                        .to_string();
                    let mut backend = match ext.as_str() {
                        "onnx" => "mdx",
                        "pth" => "vr",
                        "ckpt" => "vr",
                        _ => "demucs",
                    };

                    if let Some(info) = mdx_data.get(&hash) {
                        backend = "mdx";
                        if let Some(stem) = info.get("primary_stem") {
                            model_name =
                                format!("{} ({})", model_name, stem.as_str().unwrap_or(""));
                        }
                    } else if let Some(info) = vr_data.get(&hash) {
                        backend = "vr";
                        if let Some(stem) = info.get("primary_stem") {
                            model_name =
                                format!("{} ({})", model_name, stem.as_str().unwrap_or(""));
                        }
                    }

                    let id = format!("local_{}", hash);
                    if !current_catalog.iter().any(|e| e.id == id) {
                        println!("[INFO] Registering local model: {}", model_name);
                        current_catalog.push(ModelCatalogEntry {
                            id,
                            name: format!("[LOCAL] {}", model_name),
                            backend: backend.into(),
                            output_kind: "vocals_instrumental".into(),
                            url: "".into(),
                            sha256: "replace-with-real-sha256".into(),
                            size_bytes: std::fs::metadata(file_path).map(|m| m.len()).unwrap_or(0),
                            filename: file_path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("")
                                .into(),
                            version: "local".into(),
                            is_installed: true,
                            local_path: Some(file_path.to_string_lossy().into()),
                        });
                        added_count += 1;
                    }
                }
            }
        }
    }

    state
        .model_registry
        .save_catalog(&current_catalog)
        .map_err(|e| e.to_string())?;
    println!(
        "[INFO] Scan finished. Added {} new local models.",
        added_count
    );
    Ok(added_count)
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
            let config_dir = app
                .path()
                .app_config_dir()
                .expect("failed to get app config dir");
            let config_path = config_dir.join("config.json");

            let config = load_config(&config_path);

            let mut paths = AppPaths::new(app_data_dir, resource_dir);
            if let Some(custom_models) = config.models_dir {
                paths.models_dir = PathBuf::from(custom_models);
            }
            if let Some(custom_cache) = config.cache_dir {
                paths.cache_dir = PathBuf::from(custom_cache);
            }

            let runtime_manager = Arc::new(RuntimeManager::new(paths.clone()));
            let model_registry = Arc::new(ModelRegistry::new(
                paths.models_dir.clone(),
                paths.manifest_catalog_path(),
            ));

            let active_jobs = Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new()));

            app.manage(AppState {
                runtime_manager,
                model_registry,
                active_jobs,
                config_path,
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
            cancel_job,
            sync_uvr_catalog,
            scan_local_models,
            get_config,
            update_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
