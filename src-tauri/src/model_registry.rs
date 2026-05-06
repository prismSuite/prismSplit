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
            format!(
                "Failed to read model catalog from {}",
                self.catalog_path.display()
            )
        })?;
        let mut entries: Vec<ModelCatalogEntry> = serde_json::from_str(&contents)?;

        for entry in &mut entries {
            if let Some(local_path) = &entry.local_path {
                if Path::new(local_path).is_file() {
                    entry.is_installed = true;
                }
            } else if self.is_model_installed(entry) {
                entry.is_installed = true;
            }
        }

        Ok(entries)
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

    pub fn save_catalog(&self, catalog: &[ModelCatalogEntry]) -> Result<()> {
        let contents = serde_json::to_string_pretty(catalog)?;
        std::fs::write(&self.catalog_path, contents).with_context(|| {
            format!(
                "Failed to write model catalog to {}",
                self.catalog_path.display()
            )
        })?;
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
