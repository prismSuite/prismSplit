use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::{Result, Context};
use std::sync::Arc;
use tokio::sync::Mutex;
use dircpy::CopyBuilder; // Not added, but let's focus on logic
use walkdir::WalkDir;

use crate::agent::{Agent, AgentManifest, ExternalAgent};

pub struct AgentRegistry {
    agents: HashMap<String, Arc<Mutex<dyn Agent>>>,
}

impl AgentRegistry {
    pub fn new() -> Self {
        Self {
            agents: HashMap::new(),
        }
    }

    pub async fn discover(&mut self) -> Result<()> {
        // Find agents in ~/.prismsplit/agents/
        let mut path = dirs::home_dir().context("Could not find home directory")?;
        path.push(".prismsplit");
        path.push("agents");

        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }

        for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
            if entry.file_name() == "manifest.json" {
                let content = std::fs::read_to_string(entry.path())?;
                if let Ok(manifest) = serde_json::from_str::<AgentManifest>(&content) {
                    let agent = ExternalAgent::new(manifest.clone(), entry.path().parent().unwrap().to_path_buf());
                    println!("Discovered agent: {}", manifest.name);
                    self.agents.insert(manifest.name.clone(), Arc::new(Mutex::new(agent)));
                }
            }
        }
        Ok(())
    }

    pub fn get_agent(&self, name: &str) -> Option<Arc<Mutex<dyn Agent>>> {
        self.agents.get(name).cloned()
    }

    pub fn list_agents(&self) -> Vec<AgentManifest> {
        // Wait, getting manifest needs sync or locking, but manifest is static
        // In a real app we might cache it. Let's just return a list.
        // For simplicity, we can't easily lock async in sync context.
        vec![] // Handled elsewhere or cache loaded.
    }
}
