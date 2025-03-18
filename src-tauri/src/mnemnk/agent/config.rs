use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager, State};

use crate::mnemnk::settings;

static AGENT_CONFIG_DIR: &str = "agents";

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AgentConfig {
    pub name: String,
    pub path: String,
    pub inputs: Option<Vec<String>>,
    pub outputs: Option<Vec<String>>,
    pub default_config: Option<Value>,
    pub schema: Option<Value>,
}

pub type AgentConfigs = HashMap<String, AgentConfig>;

pub(super) fn init_agent_configs(app: &AppHandle) -> Result<()> {
    let agent_configs = read_agent_configs(app)?;
    app.manage(Mutex::new(agent_configs));
    Ok(())
}

fn read_agent_configs(app: &AppHandle) -> Result<AgentConfigs> {
    let dir = agents_dir(app);
    if dir.is_none() {
        return Err(anyhow::anyhow!("Agents directory not found"));
    }
    let mut agents = HashMap::new();
    for entry in std::fs::read_dir(dir.unwrap())? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().unwrap_or_default() == "json" {
            let name = path.file_stem().unwrap().to_string_lossy().to_string();
            let content = std::fs::read_to_string(&path)?;
            let config: AgentConfig = serde_json::from_str(&content)?;
            agents.insert(name, config);
        }
    }
    Ok(agents)
}

fn agents_dir(app: &AppHandle) -> Option<PathBuf> {
    let mnemnk_dir = settings::mnemnk_dir(app);
    if mnemnk_dir.is_none() {
        return None;
    }
    let agents_dir = PathBuf::from(mnemnk_dir.unwrap()).join(AGENT_CONFIG_DIR);
    if !agents_dir.exists() {
        std::fs::create_dir(&agents_dir).expect("Failed to create agents directory");
    }
    Some(agents_dir)
}

#[tauri::command]
pub fn get_agent_configs_cmd(agent_configs: State<Mutex<AgentConfigs>>) -> Result<Value, String> {
    let agent_flows;
    {
        let agent_configs = agent_configs.lock().unwrap();
        agent_flows = agent_configs.clone();
    }
    let value = serde_json::to_value(&agent_flows).map_err(|e| e.to_string())?;
    Ok(value)
}
