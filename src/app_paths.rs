// src-tauri/src/app_paths.rs
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub root: PathBuf,
    pub resource_dir: PathBuf,
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
        Self {
            runtime_dir: root.join("runtime"),
            python_dir: root.join("runtime").join("python"),
            venv_dir: root.join("runtime").join("venv"),
            engine_dir: root.join("engine"),
            models_dir: root.join("models"),
            manifests_dir: root.join("manifests"),
            jobs_dir: root.join("jobs"),
            logs_dir: root.join("logs"),
            cache_dir: root.join("cache"),
            root,
            resource_dir,
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
        self.root.join("uvr")
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
