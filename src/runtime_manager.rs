use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::io::AsyncWriteExt;
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
        let runtime_ready = self.bootstrap_python_available().await;
        let dependencies_ready = self.engine_dependencies_ready().await;
        let model_catalog_ready = self.paths.manifest_catalog_path().is_file();
        let installed_model_count = if self.paths.models_dir.exists() {
            std::fs::read_dir(&self.paths.models_dir)?
                .filter_map(|entry| entry.ok())
                .filter(|entry| entry.path().is_file())
                .count()
        } else {
            0
        };

        let gpu_devices = self.get_gpu_devices().await;

        Ok(EngineHealth {
            runtime_ready,
            dependencies_ready,
            ffmpeg_ready: self.command_exists("ffmpeg").await,
            model_catalog_ready,
            installed_model_count,
            active_job_count: 0,
            gpu_devices,
        })
    }

    async fn get_gpu_devices(&self) -> Vec<String> {
        if cfg!(target_os = "windows") {
            let output = Command::new("wmic")
                .args(["path", "win32_VideoController", "get", "name"])
                .output()
                .await;

            if let Ok(output) = output {
                let stdout = String::from_utf8_lossy(&output.stdout);
                return stdout
                    .lines()
                    .skip(1) // Skip header "Name"
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty())
                    .collect();
            }
        }
        vec!["CPU Only".to_string()]
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
        self.ensure_supported_python(&bootstrap_python).await?;
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

    pub async fn smart_repair(&self) -> Result<SetupStatus> {
        let _lock = self.prepare_lock.lock().await;
        let mut completed_stages = Vec::new();

        self.ensure_layout()?;
        completed_stages.push("smartRepair:createDirectories".into());

        // Sync catalog and scripts
        self.sync_engine_assets()?;
        completed_stages.push("smartRepair:syncEngineAssets".into());

        let bootstrap_python = self
            .bootstrap_python_path()
            .ok_or_else(|| anyhow::anyhow!(self.missing_python_message()))?;
        self.ensure_supported_python(&bootstrap_python).await?;

        let venv_python = self.paths.venv_python_executable();
        if !self.paths.venv_dir.is_dir() || !venv_python.is_file() {
            // Re-create entire venv if physical directory is broken
            self.create_venv(&bootstrap_python).await?;
            completed_stages.push("smartRepair:rebuildVenv".into());
        }

        // Test if pip is functioning, if not re-bootstrap it
        let pip_check = Command::new(&venv_python)
            .arg("-m")
            .arg("pip")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await;

        match pip_check {
            Ok(status) if status.success() => {}
            _ => {
                // Pip is broken, force upgrade packaging tools
                let _ = Command::new(&venv_python)
                    .arg("-m")
                    .arg("pip")
                    .arg("install")
                    .arg("--upgrade")
                    .arg("pip")
                    .arg("setuptools")
                    .arg("wheel")
                    .status()
                    .await;
                completed_stages.push("smartRepair:repairedPipBootstrap".into());
            }
        }

        // Diagnos and repair broken dependencies
        let engine_script = self.paths.installed_engine_script();
        if engine_script.is_file() {
            self.diagnose_and_repair_dependencies(&venv_python, &engine_script).await?;
            completed_stages.push("smartRepair:repairedDependencies".into());
        } else {
            self.install_dependencies().await?;
            completed_stages.push("smartRepair:fullDependencyInstall".into());
        }

        // Validate model catalog JSON
        let catalog_path = self.paths.manifest_catalog_path();
        let catalog_corrupted = if catalog_path.is_file() {
            if let Ok(metadata) = std::fs::metadata(&catalog_path) {
                metadata.len() == 0 || std::fs::read_to_string(&catalog_path).map(|content| serde_json::from_str::<serde_json::Value>(&content).is_err()).unwrap_or(true)
            } else {
                true
            }
        } else {
            true
        };

        if catalog_corrupted {
            // Restore catalog from bundled engine source
            let source_models_dir = if self.paths.bundled_engine_models_dir().exists() {
                self.paths.resource_dir.clone()
            } else {
                self.paths.workspace_engine_dir()
            };
            let source_catalog = source_models_dir.join("models").join("catalog.json");
            if source_catalog.is_file() {
                let _ = std::fs::copy(source_catalog, &catalog_path);
                completed_stages.push("smartRepair:restoredCatalog".into());
            }
        }

        let health = self.doctor().await?;
        if !health.runtime_ready || !health.dependencies_ready || !health.model_catalog_ready {
            bail!("Engine smart repair finished but the runtime is still unhealthy.");
        }

        Ok(SetupStatus {
            ready: true,
            current_stage: None,
            completed_stages,
            last_error: None,
        })
    }

    async fn diagnose_and_repair_dependencies(&self, python_exe: &Path, engine_script: &Path) -> Result<()> {
        let mut child = Command::new(python_exe)
            .arg(engine_script)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()?;

        if let Some(mut stdin) = child.stdin.take() {
            let _ = stdin.write_all(br#"{"command":"doctor","payload":{"ping":true}}"#).await;
            let _ = stdin.write_all(b"\n").await;
        }

        let output = child.wait_with_output().await?;
        if !output.status.success() {
            return self.install_dependencies().await;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let Some(line) = stdout.lines().find(|l| !l.trim().is_empty()) else {
            return self.install_dependencies().await;
        };

        let value: serde_json::Value = serde_json::from_str(line)?;
        let payload = match value.get("payload") {
            Some(val) => val,
            None => return self.install_dependencies().await,
        };

        if let Some(imports) = payload.get("backend_imports").and_then(|v| v.as_object()) {
            let mut broken_deps = Vec::new();
            for (dep, status) in imports {
                if status.get("ok").and_then(|v| v.as_bool()) == Some(false) {
                    broken_deps.push(dep.clone());
                }
            }

            if !broken_deps.is_empty() {
                // Reinstall select broken dependencies
                for dep in broken_deps {
                    let mut pip_dep = dep.clone();
                    if pip_dep == "onnxruntime" {
                        pip_dep = "onnxruntime>=1.14.1".to_string();
                    } else if pip_dep == "soundfile" {
                        pip_dep = "soundfile==0.11.0".to_string();
                    } else if pip_dep == "librosa" {
                        pip_dep = "librosa==0.9.2".to_string();
                    } else if pip_dep == "numpy" {
                        pip_dep = "numpy==1.23.5".to_string();
                    }
                    let status = Command::new(python_exe)
                        .arg("-m")
                        .arg("pip")
                        .arg("install")
                        .arg("--upgrade")
                        .arg(&pip_dep)
                        .status()
                        .await?;
                    if !status.success() {
                        bail!("Smart repair failed to install dependency: {}", dep);
                    }
                }
                // Secure editable link
                let _ = Command::new(python_exe)
                    .arg("-m")
                    .arg("pip")
                    .arg("install")
                    .arg("--no-build-isolation")
                    .arg("-e")
                    .arg(&self.paths.engine_dir)
                    .status()
                    .await;
            }
        } else {
            return self.install_dependencies().await;
        }

        Ok(())
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

    async fn bootstrap_python_available(&self) -> bool {
        match self.bootstrap_python_path() {
            Some(path) if path.is_file() => true,
            Some(path) if path == PathBuf::from("python") => self.command_exists("python").await,
            Some(_) => false,
            None => false,
        }
    }

    async fn engine_dependencies_ready(&self) -> bool {
        let venv_python = self.paths.venv_python_executable();
        let engine_script = self.paths.installed_engine_script();

        if !venv_python.is_file() || !engine_script.is_file() {
            return false;
        }

        self.invoke_engine_doctor(&venv_python, &engine_script)
            .await
            .unwrap_or(false)
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

        let source_uvr_dir = if source_engine_dir == self.paths.resource_dir {
            self.paths.resource_dir.parent()
                .map(|p| p.join("uvr"))
                .unwrap_or_else(|| self.paths.resource_dir.join("uvr"))
        } else {
            self.paths.workspace_uvr_dir()
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

        if source_uvr_dir.exists() {
            Self::copy_dir_recursive(&source_uvr_dir, &self.paths.installed_uvr_dir())?;
        }

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

        let bootstrap_status = Command::new(&venv_python)
            .arg("-m")
            .arg("pip")
            .arg("install")
            .arg("--upgrade")
            .arg("pip")
            .arg("setuptools")
            .arg("wheel")
            .status()
            .await
            .with_context(|| {
                format!(
                    "Failed to bootstrap pip/setuptools/wheel with {}",
                    venv_python.display()
                )
            })?;

        if !bootstrap_status.success() {
            bail!("Failed to bootstrap PrismSplit Python packaging tools");
        }

        let status = Command::new(&venv_python)
            .arg("-m")
            .arg("pip")
            .arg("install")
            .arg("--no-build-isolation")
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
        let version_arg = match command {
            "python" | "python.exe" => "--version",
            _ => "-version",
        };

        Command::new(command)
            .arg(version_arg)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await
            .map(|status| status.success())
            .unwrap_or(false)
    }

    async fn ensure_supported_python(&self, python_exe: &Path) -> Result<()> {
        let output = Command::new(python_exe)
            .arg("-c")
            .arg("import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .await
            .with_context(|| format!("Failed to query Python version from {}", python_exe.display()))?;

        if !output.status.success() {
            bail!(
                "Failed to query Python version from {}: {}",
                python_exe.display(),
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }

        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let supported = matches!(version.as_str(), "3.9" | "3.10" | "3.11");
        if !supported {
            bail!(
                "PrismSplit's UVR backend currently requires Python 3.9, 3.10, or 3.11. Resolved {} at {}, which is unsupported for the current engine dependencies. Install a compatible Python and point PRISMSPLIT_DEV_PYTHON to it for local development.",
                version,
                python_exe.display()
            );
        }

        Ok(())
    }

    async fn invoke_engine_doctor(&self, python_exe: &Path, engine_script: &Path) -> Result<bool> {
        let mut child = Command::new(python_exe)
            .arg(engine_script)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .with_context(|| {
                format!(
                    "Failed to spawn engine doctor with {} {}",
                    python_exe.display(),
                    engine_script.display()
                )
            })?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(br#"{"command":"doctor","payload":{"ping":true}}"#)
                .await?;
            stdin.write_all(b"\n").await?;
        }

        let output = child.wait_with_output().await?;
        if !output.status.success() {
            return Ok(false);
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let Some(line) = stdout.lines().find(|line| !line.trim().is_empty()) else {
            return Ok(false);
        };

        let value: Value = serde_json::from_str(line)?;
        Ok(value
            .get("payload")
            .and_then(|payload| payload.get("ready"))
            .and_then(Value::as_bool)
            .unwrap_or(false))
    }

    fn missing_python_message(&self) -> String {
        format!(
            "Embedded Python runtime is not available. Expected bundled runtime at {}. In local development you can set PRISMSPLIT_DEV_PYTHON to a compatible Python 3.9, 3.10, or 3.11 executable.",
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
