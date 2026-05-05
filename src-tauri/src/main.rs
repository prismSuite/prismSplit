#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod agent;
mod app_paths;
mod download_manager;
mod engine_bridge;
mod model_registry;
mod models;
mod registry;
mod runtime_manager;

use registry::AgentRegistry;
use serde_json::json;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

struct AppState {
    registry: Arc<Mutex<AgentRegistry>>,
}

#[tauri::command]
async fn get_available_models() -> Result<Vec<String>, String> {
    // Mock response for frontend
    Ok(vec![
        "Demucs v4 (htdemucs)".into(),
        "MDX-Net (UVR-MDX-NET-Inst_HQ_1)".into(),
        "VR Architecture (UVR_v5_Vocal_Only)".into(),
    ])
}

#[tauri::command]
async fn process_audio(
    state: State<'_, AppState>,
    file_path: String,
    model: String,
    output_dir: String,
    quality: String,
) -> Result<String, String> {
    // In a real app, we would look up the stem_separation or gain_staging agent
    let mut registry = state.registry.lock().await;

    // Simulate IPC call to external agent
    println!(
        "Processing audio: {} with model {} at quality {} to {}",
        file_path, model, quality, output_dir
    );

    // Attempting to use a hypothetical "stem_separation" agent
    if let Some(agent_arc) = registry.get_agent("stem_separation") {
        let mut agent = agent_arc.lock().await;
        // execute agent payload
        let result = agent
            .execute(
                "separate",
                json!({
                    "input": file_path,
                    "model": model,
                    "output": output_dir,
                    "quality": quality
                }),
            )
            .await
            .map_err(|e| e.to_string())?;

        return Ok(format!("Success: {:?}", result));
    }

    // For the UI preview to work without real agents:
    Ok("Procesamiento simulado en entorno de desarrollo. Para separacion real, ejecute la app compilada localmente.".into())
}

#[tauri::command]
async fn check_system() -> Result<String, String> {
    Ok("Sistema Listo - PrismSplit".into())
}

fn main() {
    let registry = Arc::new(Mutex::new(AgentRegistry::new()));

    // In a full implementation, we'd spawn a task to discover agents
    let reg_clone = registry.clone();
    tauri::async_runtime::spawn(async move {
        let mut reg = reg_clone.lock().await;
        let _ = reg.discover().await;
    });

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState { registry })
        .invoke_handler(tauri::generate_handler![
            get_available_models,
            process_audio,
            check_system
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
