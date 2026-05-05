// src-tauri/src/models.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SetupStatus {
    pub ready: bool,
    pub current_stage: Option<String>,
    pub completed_stages: Vec<String>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineHealth {
    pub runtime_ready: bool,
    pub dependencies_ready: bool,
    pub ffmpeg_ready: bool,
    pub model_catalog_ready: bool,
    pub installed_model_count: usize,
    pub active_job_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeparationRequest {
    pub input_path: String,
    pub model_id: String,
    pub output_dir: String,
    pub format: String,
}
