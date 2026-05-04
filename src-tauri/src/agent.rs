use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use anyhow::{Result, Context};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct AgentManifest {
    pub name: String,
    pub description: String,
    pub version: String,
    pub executable: String,
}

#[derive(Serialize, Debug)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: serde_json::Value,
    pub id: u64,
}

#[derive(Deserialize, Debug)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<serde_json::Value>,
    pub id: u64,
}

#[async_trait]
pub trait Agent: Send + Sync {
    fn manifest(&self) -> AgentManifest;
    async fn execute(&mut self, method: &str, params: serde_json::Value) -> Result<serde_json::Value>;
}

pub struct ExternalAgent {
    manifest: AgentManifest,
    process: Option<Child>,
    request_id: u64,
    directory: std::path::PathBuf,
}

impl ExternalAgent {
    pub fn new(manifest: AgentManifest, directory: std::path::PathBuf) -> Self {
        Self {
            manifest,
            process: None,
            request_id: 1,
            directory,
        }
    }

    async fn ensure_running(&mut self) -> Result<()> {
        if self.process.is_none() {
            let mut cmd = Command::new(self.directory.join(&self.manifest.executable));
            cmd.stdin(Stdio::piped())
               .stdout(Stdio::piped())
               .stderr(Stdio::inherit());

            let child = cmd.spawn().context("Failed to spawn agent process")?;
            self.process = Some(child);
        }
        Ok(())
    }
}

#[async_trait]
impl Agent for ExternalAgent {
    fn manifest(&self) -> AgentManifest {
        self.manifest.clone()
    }

    async fn execute(&mut self, method: &str, params: serde_json::Value) -> Result<serde_json::Value> {
        self.ensure_running().await?;

        let child = self.process.as_mut().unwrap();
        let stdin = child.stdin.as_mut().unwrap();
        let stdout = child.stdout.as_mut().unwrap();

        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: self.request_id,
        };
        self.request_id += 1;

        let mut req_str = serde_json::to_string(&req)?;
        req_str.push('\n');

        stdin.write_all(req_str.as_bytes()).await?;
        stdin.flush().await?;

        // Read response
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();
        reader.read_line(&mut line).await?;

        let res: JsonRpcResponse = serde_json::from_str(&line).context("Failed to parse agent response")?;
        
        if let Some(error) = res.error {
            anyhow::bail!("Agent error: {:?}", error);
        }

        Ok(res.result.unwrap_or(serde_json::Value::Null))
    }
}
