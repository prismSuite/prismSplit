// src-tauri/src/runtime_manager.rs
use anyhow::Result;

use crate::app_paths::AppPaths;
use crate::models::{EngineHealth, SetupStatus};

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

    pub async fn prepare(&self) -> Result<SetupStatus> {
        std::fs::create_dir_all(&self.paths.root)?;
        std::fs::create_dir_all(&self.paths.runtime_dir)?;
        std::fs::create_dir_all(&self.paths.models_dir)?;

        Ok(SetupStatus {
            ready: false,
            current_stage: None,
            completed_stages: vec!["create_directories".into()],
            last_error: None,
        })
    }
}
