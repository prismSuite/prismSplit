// src-tauri/src/engine_bridge.rs
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineEvent {
    pub event: String,
    pub job_id: Option<String>,
    pub message: Option<String>,
    pub percent: Option<f32>,
    pub payload: Option<serde_json::Value>,
}

pub fn parse_event_line(line: &str) -> Result<EngineEvent> {
    Ok(serde_json::from_str(line)?)
}

pub struct EngineBridge {
    pub python_exe: std::path::PathBuf,
    pub engine_script: std::path::PathBuf,
}

impl EngineBridge {
    pub fn new(python_exe: std::path::PathBuf, engine_script: std::path::PathBuf) -> Self {
        Self {
            python_exe,
            engine_script,
        }
    }

    pub async fn run_command(
        &self,
        command: &str,
        payload: serde_json::Value,
    ) -> Result<tokio::process::Child> {
        let mut child = Command::new(&self.python_exe)
            .arg(&self.engine_script)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("Failed to open stdin"))?;
        let request = serde_json::json!({
            "command": command,
            "payload": payload,
        });

        let line = serde_json::to_string(&request)? + "\n";
        stdin.write_all(line.as_bytes()).await?;
        stdin.flush().await?;

        Ok(child)
    }
}
