use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::AppHandle;

use super::bultin::builtin_agent_defs;
use crate::mnemnk::settings;

static AGENT_CONFIG_DIR: &str = "agents";

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AgentDefinition {
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub path: Option<String>,
    pub inputs: Option<Vec<String>>,
    pub outputs: Option<Vec<String>>,
    pub default_config: Option<HashMap<String, AgentDefaultConfig>>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AgentDefaultConfig {
    pub value: Value,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub scope: Option<String>,
}

pub type AgentDefinitions = HashMap<String, AgentDefinition>;

pub(super) fn init_agent_defs(app: &AppHandle) -> Result<AgentDefinitions> {
    let mut defs = builtin_agent_defs();
    defs.extend(read_agent_defs(app)?);
    Ok(defs)
}

fn read_agent_defs(app: &AppHandle) -> Result<AgentDefinitions> {
    let mut defs: AgentDefinitions = Default::default();

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
        let content = match std::fs::read_to_string(&mnemnk_json) {
            Ok(ret) => ret,
            Err(e) => {
                log::warn!("Failed to read agent config file: {}", e);
                continue;
            }
        };
        let def: AgentDefinition = match serde_json::from_str(&content) {
            Ok(def) => def,
            Err(e) => {
                log::warn!("Failed to parse agent config file: {}", e);
                continue;
            }
        };
        // check if name and def.name match
        if def.name != name {
            log::warn!(
                "Agent name and config name mismatch: {} != {}",
                name,
                def.name
            );
            continue;
        }
        defs.insert(name, def);
    }
    Ok(defs)
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
