// src-tauri/src/model_registry.rs
use crate::models::ModelCatalogEntry;
use anyhow::Result;
use std::path::PathBuf;

pub struct ModelRegistry {
    pub models_dir: PathBuf,
}

impl ModelRegistry {
    pub fn new(models_dir: PathBuf) -> Self {
        Self { models_dir }
    }

    pub fn is_model_installed(&self, entry: &ModelCatalogEntry) -> bool {
        self.models_dir.join(&entry.filename).exists()
    }
}

pub fn load_catalog_from_str(contents: &str) -> Result<Vec<ModelCatalogEntry>> {
    Ok(serde_json::from_str(contents)?)
}
