// src-tauri/src/runtime_manager.rs
use anyhow::Result;

use crate::app_paths::AppPaths;
use crate::models::EngineHealth;

#[derive(Debug, Clone)]
pub struct RuntimeManager {
    paths: AppPaths,
}

impl RuntimeManager {
    pub fn new(paths: AppPaths) -> Self {
        Self { paths }
    }

    pub async fn doctor(&self) -> Result<EngineHealth> {
        Ok(EngineHealth {
            runtime_ready: self.paths.python_dir.exists(),
            dependencies_ready: self.paths.venv_dir.exists(),
            ffmpeg_ready: false,
            model_catalog_ready: self.paths.manifests_dir.exists(),
            installed_model_count: 0,
            active_job_count: 0,
        })
    }
}
