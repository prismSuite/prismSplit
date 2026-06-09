use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineEvent {
    pub event: String,
    pub job_id: Option<String>,
    pub message: Option<String>,
    pub percent: Option<f32>,
    pub payload: Option<Value>,
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

    pub async fn spawn_command(
        &self,
        command: &str,
        payload: Value,
    ) -> Result<tokio::process::Child> {
        let mut child = Command::new(&self.python_exe)
            .arg(&self.engine_script)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .with_context(|| {
                format!(
                    "Failed to spawn engine {} {}",
                    self.python_exe.display(),
                    self.engine_script.display()
                )
            })?;

        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("Failed to open engine stdin"))?;
        let request = serde_json::json!({
            "command": command,
            "payload": payload,
        });

        stdin
            .write_all(format!("{}\n", serde_json::to_string(&request)?).as_bytes())
            .await?;
        stdin.flush().await?;

        Ok(child)
    }

    pub async fn run_command_collect(
        &self,
        command: &str,
        payload: Value,
    ) -> Result<(Vec<EngineEvent>, Child)> {
        let mut child = self.spawn_command(command, payload).await?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Failed to capture engine stdout"))?;

        let mut events = Vec::new();
        let mut lines = BufReader::new(stdout).lines();

        while let Some(line) = lines.next_line().await? {
            if line.trim().is_empty() {
                continue;
            }

            let event = parse_event_line(&line)?;
            let is_terminal = matches!(event.event.as_str(), "result" | "error");
            events.push(event);

            if is_terminal {
                break;
            }
        }

        Ok((events, child))
    }

    pub async fn stream_stdout<F>(
        stdout: tokio::process::ChildStdout,
        mut on_event: F,
    ) -> Result<EngineEvent>
    where
        F: FnMut(&EngineEvent) + Send + 'static,
    {
        let mut lines = BufReader::new(stdout).lines();

        while let Some(line) = lines.next_line().await? {
            if line.trim().is_empty() {
                continue;
            }

            let event = parse_event_line(&line)?;
            let is_terminal = matches!(event.event.as_str(), "result" | "error");
            on_event(&event);

            if is_terminal {
                return Ok(event);
            }
        }

        Err(anyhow!("Engine returned no terminal event"))
    }

    pub async fn run_command_stream<F>(
        &self,
        command: &str,
        payload: Value,
        on_event: F,
    ) -> Result<(EngineEvent, Child)>
    where
        F: FnMut(&EngineEvent) + Send + 'static,
    {
        let mut child = self.spawn_command(command, payload).await?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("Failed to capture engine stdout"))?;

        let event = Self::stream_stdout(stdout, on_event).await?;
        Ok((event, child))
    }
}
