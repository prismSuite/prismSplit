use crate::models::ModelCatalogEntry;
use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct ModelRegistry {
    pub models_dir: PathBuf,
    pub catalog_path: PathBuf,
}

impl ModelRegistry {
    pub fn new(models_dir: PathBuf, catalog_path: PathBuf) -> Self {
        Self {
            models_dir,
            catalog_path,
        }
    }

    pub fn load_catalog(&self) -> Result<Vec<ModelCatalogEntry>> {
        let contents = std::fs::read_to_string(&self.catalog_path).with_context(|| {
            format!("Failed to read model catalog from {}", self.catalog_path.display())
        })?;
        load_catalog_from_str(&contents)
    }

    pub fn get_entry(&self, model_id: &str) -> Result<ModelCatalogEntry> {
        self.load_catalog()?
            .into_iter()
            .find(|entry| entry.id == model_id)
            .ok_or_else(|| anyhow::anyhow!("Model `{}` was not found in the catalog", model_id))
    }

    pub fn is_model_installed(&self, entry: &ModelCatalogEntry) -> bool {
        self.models_dir.join(&entry.filename).exists()
    }

    pub fn installed_model_path(&self, entry: &ModelCatalogEntry) -> PathBuf {
        self.models_dir.join(&entry.filename)
    }

    pub fn validate_downloadable(&self, entry: &ModelCatalogEntry) -> Result<()> {
        if entry.url.trim().is_empty() {
            bail!("Model `{}` does not define a download URL", entry.id);
        }

        Ok(())
    }
}

pub fn load_catalog_from_path(path: &Path) -> Result<Vec<ModelCatalogEntry>> {
    let contents = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read catalog from {}", path.display()))?;
    load_catalog_from_str(&contents)
}

pub fn load_catalog_from_str(contents: &str) -> Result<Vec<ModelCatalogEntry>> {
    Ok(serde_json::from_str(contents)?)
}
