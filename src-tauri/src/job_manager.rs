// src-tauri/src/job_manager.rs
use crate::models::SeparationRequest;
use anyhow::{bail, Result};

pub fn validate_request(request: &SeparationRequest) -> Result<()> {
    if request.input_path.trim().is_empty() {
        bail!("Input path is required");
    }
    if request.model_id.trim().is_empty() {
        bail!("Model id is required");
    }
    if request.output_dir.trim().is_empty() {
        bail!("Output directory is required");
    }
    Ok(())
}
