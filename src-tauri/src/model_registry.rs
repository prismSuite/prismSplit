// src-tauri/src/model_registry.rs
use crate::models::ModelCatalogEntry;
use anyhow::Result;

pub fn load_catalog_from_str(contents: &str) -> Result<Vec<ModelCatalogEntry>> {
    Ok(serde_json::from_str(contents)?)
}
