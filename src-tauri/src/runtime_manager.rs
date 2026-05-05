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
        let mut completed_stages = Vec::new();

        std::fs::create_dir_all(&self.paths.root)?;
        std::fs::create_dir_all(&self.paths.runtime_dir)?;
        std::fs::create_dir_all(&self.paths.models_dir)?;
        completed_stages.push("create_directories".into());

        // Unpack python if missing
        if !self.paths.python_dir.exists() {
            self.unpack_embedded_python().await?;
            completed_stages.push("unpack_python".into());
        }

        // Create venv if missing
        if !self.paths.venv_dir.exists() {
            self.create_venv().await?;
            completed_stages.push("create_venv".into());
        }

        Ok(SetupStatus {
            ready: self.paths.venv_dir.exists(),
            current_stage: None,
            completed_stages,
            last_error: None,
        })
    }

    async fn unpack_embedded_python(&self) -> Result<()> {
        // Placeholder: in a real app, this would unzip a bundled asset
        std::fs::create_dir_all(&self.paths.python_dir)?;
        Ok(())
    }

    async fn create_venv(&self) -> Result<()> {
        // In a real app, we would run: python -m venv venv
        std::fs::create_dir_all(&self.paths.venv_dir)?;
        Ok(())
    }
}
