// src-tauri/src/models.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct SetupStatus {
    pub ready: bool,
    pub current_stage: Option<String>,
    pub completed_stages: Vec<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineHealth {
    pub runtime_ready: bool,
    pub dependencies_ready: bool,
    pub ffmpeg_ready: bool,
    pub model_catalog_ready: bool,
    pub installed_model_count: usize,
    pub active_job_count: usize,
    pub gpu_devices: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCatalogEntry {
    pub id: String,
    pub name: String,
    pub backend: String,
    pub output_kind: String,
    pub url: String,
    pub sha256: String,
    pub size_bytes: u64,
    pub filename: String,
    pub version: String,
    #[serde(default)]
    pub is_installed: bool,
    pub local_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeparationRequest {
    pub input_path: String,
    pub model_id: String,
    pub output_dir: String,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProcessAudioResponse {
    pub job_id: String,
    pub vocals_path: String,
    pub instrumental_path: String,
    pub backend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgressEvent {
    pub model_id: String,
    pub progress: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    #[serde(default)]
    pub version: u32,
    pub models_dir: Option<String>,
    pub cache_dir: Option<String>,
    pub last_input_file: Option<String>,
    pub last_output_dir: Option<String>,
    pub last_selected_model: Option<String>,
    pub last_quality: Option<String>,
    pub last_export_format: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: 1,
            models_dir: None,
            cache_dir: None,
            last_input_file: None,
            last_output_dir: None,
            last_selected_model: None,
            last_quality: None,
            last_export_format: None,
        }
    }
}
