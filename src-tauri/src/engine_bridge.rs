// src-tauri/src/engine_bridge.rs
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineEvent {
    pub event: String,
    pub job_id: Option<String>,
    pub message: Option<String>,
    pub percent: Option<f32>,
}

pub fn parse_event_line(line: &str) -> Result<EngineEvent> {
    Ok(serde_json::from_str(line)?)
}
