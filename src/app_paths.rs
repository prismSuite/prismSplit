// src-tauri/src/app_paths.rs
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub root: PathBuf,
    pub resource_dir: PathBuf,
    pub app_data_dir: PathBuf,
    pub runtime_dir: PathBuf,
    pub python_dir: PathBuf,
    pub venv_dir: PathBuf,
    pub engine_dir: PathBuf,
    pub models_dir: PathBuf,
    pub manifests_dir: PathBuf,
    pub jobs_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub cache_dir: PathBuf,
}

impl AppPaths {
    pub fn new(root: PathBuf, resource_dir: PathBuf) -> Self {
        let app_data_dir = root.clone();
        let app_cache_dir = if cfg!(test) {
            root.clone()
        } else {
            dirs::cache_dir()
                .unwrap_or_else(|| root.clone())
                .join("PrismSplit")
        };

        Self {
            runtime_dir: app_data_dir.join("runtime"),
            python_dir: app_data_dir.join("runtime").join("python"),
            venv_dir: app_data_dir.join("runtime").join("venv"),
            engine_dir: app_data_dir.join("engine"),
            models_dir: app_data_dir.join("models"),
            manifests_dir: app_data_dir.join("manifests"),
            jobs_dir: app_data_dir.join("jobs"),
            logs_dir: app_data_dir.join("logs"),
            cache_dir: app_cache_dir.join("cache"),
            root,
            resource_dir,
            app_data_dir,
        }
    }

    pub fn bundled_engine_python_dir(&self) -> PathBuf {
        self.resource_dir.join("python")
    }

    pub fn bundled_engine_models_dir(&self) -> PathBuf {
        self.resource_dir.join("models")
    }

    pub fn bundled_engine_pyproject(&self) -> PathBuf {
        self.resource_dir.join("pyproject.toml")
    }

    pub fn workspace_engine_dir(&self) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("engine")
    }

    pub fn workspace_engine_python_dir(&self) -> PathBuf {
        self.workspace_engine_dir().join("python")
    }

    pub fn workspace_engine_models_dir(&self) -> PathBuf {
        self.workspace_engine_dir().join("models")
    }

    pub fn workspace_engine_pyproject(&self) -> PathBuf {
        self.workspace_engine_dir().join("pyproject.toml")
    }

    pub fn workspace_uvr_dir(&self) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("uvr")
    }

    pub fn installed_engine_python_dir(&self) -> PathBuf {
        self.engine_dir.join("python")
    }

    pub fn installed_uvr_dir(&self) -> PathBuf {
        self.app_data_dir.join("uvr")
    }

    pub fn installed_engine_script(&self) -> PathBuf {
        self.installed_engine_python_dir()
            .join("prismsplit_engine.py")
    }

    pub fn manifest_catalog_path(&self) -> PathBuf {
        self.manifests_dir.join("catalog.json")
    }

    pub fn venv_python_executable(&self) -> PathBuf {
        if cfg!(target_os = "windows") {
            self.venv_dir.join("Scripts").join("python.exe")
        } else {
            self.venv_dir.join("bin").join("python")
        }
    }

    pub fn runtime_bootstrap_python(&self) -> PathBuf {
        if cfg!(target_os = "windows") {
            self.python_dir.join("python.exe")
        } else {
            self.python_dir.join("python")
        }
    }
}
