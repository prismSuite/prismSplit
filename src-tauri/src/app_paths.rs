// src-tauri/src/app_paths.rs
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub root: PathBuf,
    pub resource_dir: PathBuf,
    pub runtime_dir: PathBuf,
    pub python_dir: PathBuf,
    pub venv_dir: PathBuf,
    pub wheels_dir: PathBuf,
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
            wheels_dir: root.join("runtime").join("wheels"),
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
}
