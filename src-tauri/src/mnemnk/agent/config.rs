use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::AppHandle;

use super::bultin::builtin_agent_configs;
use crate::mnemnk::settings;

static AGENT_CONFIG_DIR: &str = "agents";

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AgentConfig {
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub path: Option<String>,
    pub inputs: Option<Vec<String>>,
    pub outputs: Option<Vec<String>>,
    pub default_config: Option<HashMap<String, AgentDefaultConfigEntry>>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AgentDefaultConfigEntry {
    pub value: Value,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub scope: Option<String>,
}

pub type AgentConfigs = HashMap<String, AgentConfig>;

pub(super) fn init_agent_configs(app: &AppHandle) -> Result<AgentConfigs> {
    let mut agent_configs = builtin_agent_configs();
    read_agent_configs(app, &mut agent_configs)?;
    Ok(agent_configs)
}

fn read_agent_configs(app: &AppHandle, agent_configs: &mut AgentConfigs) -> Result<()> {
    let dir = agents_dir(app);
    if dir.is_none() {
        return Err(anyhow::anyhow!("Agents directory not found"));
    }
    for entry in std::fs::read_dir(dir.unwrap())? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let mnemnk_json = path.join("mnemnk.json");
        if !mnemnk_json.exists() {
            continue;
        }
        let name = path.file_name().unwrap().to_string_lossy().to_string();
        if agent_configs.contains_key(&name) {
            log::warn!("Duplicate agent name: {}", name);
            continue;
        }
        let content = match std::fs::read_to_string(&mnemnk_json) {
            Ok(ret) => ret,
            Err(e) => {
                log::warn!("Failed to read agent config file: {}", e);
                continue;
            }
        };
        let config: AgentConfig = match serde_json::from_str(&content) {
            Ok(config) => config,
            Err(e) => {
                log::warn!("Failed to parse agent config file: {}", e);
                continue;
            }
        };
        // check if name and config.name match
        if config.name != name {
            log::warn!(
                "Agent name and config name mismatch: {} != {}",
                name,
                config.name
            );
            continue;
        }
        agent_configs.insert(name, config);
    }
    Ok(())
}

pub fn agents_dir(app: &AppHandle) -> Option<PathBuf> {
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
