use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};
use tokio::process::Command;
use tokio::sync::Mutex;

use crate::app_paths::AppPaths;
use crate::models::{EngineHealth, SetupStatus};

#[derive(Debug)]
pub struct RuntimeManager {
    paths: AppPaths,
    prepare_lock: Mutex<()>,
}

impl RuntimeManager {
    pub fn new(paths: AppPaths) -> Self {
        Self {
            paths,
            prepare_lock: Mutex::new(()),
        }
    }

    pub fn paths(&self) -> &AppPaths {
        &self.paths
    }

    pub async fn doctor(&self) -> Result<EngineHealth> {
        let runtime_ready = self.bootstrap_python_path().is_some();
        let dependencies_ready = self.paths.venv_python_executable().is_file();
        let model_catalog_ready = self.paths.manifest_catalog_path().is_file();
        let installed_model_count = if self.paths.models_dir.exists() {
            std::fs::read_dir(&self.paths.models_dir)?
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.path().is_file())
                .count()
        } else {
            0
        };

        Ok(EngineHealth {
            runtime_ready,
            dependencies_ready,
            ffmpeg_ready: self.command_exists("ffmpeg").await,
            model_catalog_ready,
            installed_model_count,
            active_job_count: 0,
        })
    }

    pub async fn prepare(&self) -> Result<SetupStatus> {
        let _lock = self.prepare_lock.lock().await;
        let mut completed_stages = Vec::new();

        self.ensure_layout()?;
        completed_stages.push("createDirectories".into());

        self.sync_engine_assets()?;
        completed_stages.push("syncEngineAssets".into());

        let bootstrap_python = self
            .bootstrap_python_path()
            .ok_or_else(|| anyhow::anyhow!(self.missing_python_message()))?;
        completed_stages.push("resolvePython".into());

        if !self.paths.venv_python_executable().is_file() {
            self.create_venv(&bootstrap_python).await?;
            completed_stages.push("createVenv".into());
        }

        self.install_dependencies().await?;
        completed_stages.push("installDependencies".into());

        let health = self.doctor().await?;
        if !health.runtime_ready || !health.dependencies_ready || !health.model_catalog_ready {
            bail!("Engine prepare completed without producing a healthy runtime");
        }

        Ok(SetupStatus {
            ready: true,
            current_stage: None,
            completed_stages,
            last_error: None,
        })
    }

    fn ensure_layout(&self) -> Result<()> {
        for dir in [
            &self.paths.root,
            &self.paths.runtime_dir,
            &self.paths.models_dir,
            &self.paths.jobs_dir,
            &self.paths.logs_dir,
            &self.paths.cache_dir,
            &self.paths.manifests_dir,
            &self.paths.engine_dir,
        ] {
            std::fs::create_dir_all(dir)?;
        }

        Ok(())
    }

    fn sync_engine_assets(&self) -> Result<()> {
        let source_engine_dir = if self.paths.bundled_engine_python_dir().exists()
            && self.paths.bundled_engine_models_dir().exists()
        {
            self.paths.resource_dir.clone()
        } else {
            self.paths.workspace_engine_dir()
        };

        let source_python_dir = if source_engine_dir == self.paths.resource_dir {
            self.paths.bundled_engine_python_dir()
        } else {
            self.paths.workspace_engine_python_dir()
        };

        let source_models_dir = if source_engine_dir == self.paths.resource_dir {
            self.paths.bundled_engine_models_dir()
        } else {
            self.paths.workspace_engine_models_dir()
        };

        let source_pyproject = if source_engine_dir == self.paths.resource_dir {
            self.paths.bundled_engine_pyproject()
        } else {
            self.paths.workspace_engine_pyproject()
        };

        if !source_python_dir.exists() {
            bail!(
                "Engine python sources were not found at {}",
                source_python_dir.display()
            );
        }

        if !source_models_dir.exists() {
            bail!(
                "Engine model catalog sources were not found at {}",
                source_models_dir.display()
            );
        }

        Self::copy_dir_recursive(
            &source_python_dir,
            &self.paths.installed_engine_python_dir(),
        )?;
        Self::copy_dir_recursive(&source_models_dir, &self.paths.engine_dir.join("models"))?;

        if source_pyproject.is_file() {
            std::fs::copy(
                source_pyproject,
                self.paths.engine_dir.join("pyproject.toml"),
            )?;
        }

        let source_catalog = source_models_dir.join("catalog.json");
        if !source_catalog.is_file() {
            bail!(
                "catalog.json was not found in {}",
                source_models_dir.display()
            );
        }
        std::fs::copy(source_catalog, self.paths.manifest_catalog_path())?;

        Ok(())
    }

    fn bootstrap_python_path(&self) -> Option<PathBuf> {
        if self.paths.runtime_bootstrap_python().is_file() {
            return Some(self.paths.runtime_bootstrap_python());
        }

        if cfg!(debug_assertions) {
            if let Some(dev_python) = std::env::var_os("PRISMSPLIT_DEV_PYTHON") {
                let path = PathBuf::from(dev_python);
                if path.is_file() {
                    return Some(path);
                }
            }

            return Some(PathBuf::from("python"));
        }

        None
    }

    async fn create_venv(&self, bootstrap_python: &Path) -> Result<()> {
        let status = Command::new(bootstrap_python)
            .arg("-m")
            .arg("venv")
            .arg(&self.paths.venv_dir)
            .status()
            .await
            .with_context(|| format!("Failed to launch {}", bootstrap_python.display()))?;

        if !status.success() {
            bail!("Failed to create PrismSplit venv");
        }

        Ok(())
    }

    async fn install_dependencies(&self) -> Result<()> {
        let venv_python = self.paths.venv_python_executable();
        if !venv_python.is_file() {
            bail!(
                "PrismSplit venv python executable was not found at {}",
                venv_python.display()
            );
        }

        let status = Command::new(&venv_python)
            .arg("-m")
            .arg("pip")
            .arg("install")
            .arg("-e")
            .arg(&self.paths.engine_dir)
            .status()
            .await
            .with_context(|| {
                format!(
                    "Failed to install engine dependencies with {}",
                    venv_python.display()
                )
            })?;

        if !status.success() {
            bail!("Failed to install PrismSplit engine dependencies");
        }

        Ok(())
    }

    async fn command_exists(&self, command: &str) -> bool {
        Command::new(command)
            .arg("-version")
            .status()
            .await
            .map(|status| status.success())
            .unwrap_or(false)
    }

    fn missing_python_message(&self) -> String {
        format!(
            "Embedded Python runtime is not available. Expected bundled runtime at {}. In local development you can set PRISMSPLIT_DEV_PYTHON to a Python executable.",
            self.paths.runtime_bootstrap_python().display()
        )
    }

    fn copy_dir_recursive(source: &Path, destination: &Path) -> Result<()> {
        std::fs::create_dir_all(destination)?;

        for entry in std::fs::read_dir(source)? {
            let entry = entry?;
            let entry_path = entry.path();
            let destination_path = destination.join(entry.file_name());

            if entry_path.is_dir() {
                Self::copy_dir_recursive(&entry_path, &destination_path)?;
            } else {
                std::fs::copy(&entry_path, &destination_path)?;
            }
        }

        Ok(())
    }
}
