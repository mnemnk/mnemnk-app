use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::AppHandle;

use super::builtin::builtin_agent_defs;
use crate::mnemnk::settings;

static AGENTS_DIR: &str = "agents";

pub type AgentDefinitions = HashMap<String, AgentDefinition>;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AgentDefinition {
    pub kind: String,
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub inputs: Option<Vec<String>>,
    pub outputs: Option<Vec<String>>,
    pub default_config: Option<AgentDefaultConfig>,

    // CommandAgent
    pub path: Option<String>,
}

pub type AgentDefaultConfig = HashMap<String, AgentDefaultConfigEntry>;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AgentDefaultConfigEntry {
    pub value: Value,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub scope: Option<String>,
}

pub fn agents_dir(app: &AppHandle) -> Option<PathBuf> {
    let mnemnk_dir = settings::mnemnk_dir(app);
    if mnemnk_dir.is_none() {
        return None;
    }
    let agents_dir = PathBuf::from(mnemnk_dir.unwrap()).join(AGENTS_DIR);
    if !agents_dir.exists() {
        std::fs::create_dir(&agents_dir).expect("Failed to create agents directory");
    }
    Some(agents_dir)
}

pub(super) fn init_agent_defs(app: &AppHandle) -> Result<AgentDefinitions> {
    let mut defs = builtin_agent_defs();
    defs.extend(read_agent_defs(app)?);
    Ok(defs)
}

fn read_agent_defs(app: &AppHandle) -> Result<AgentDefinitions> {
    let mut defs: AgentDefinitions = Default::default();

    // read agent definitions from agents directory
    let dir = agents_dir(app);
    if dir.is_none() {
        return Err(anyhow::anyhow!("Agents directory not found"));
    }
    for entry in std::fs::read_dir(dir.unwrap())? {
        let entry = entry?;
        let agent_dir = entry.path();
        if !agent_dir.is_dir() {
            continue;
        }
        // read mnemnk.json, and post process it
        let def =
            read_agent_def(&agent_dir).and_then(|def| post_process_agent_def(def, &agent_dir));

        if let Some(def) = def {
            defs.insert(def.name.clone(), def);
        }
    }

    Ok(defs)
}

fn read_agent_def(agent_dir: &PathBuf) -> Option<AgentDefinition> {
    let mnemnk_json = agent_dir.join("mnemnk.json");
    if !mnemnk_json.exists() {
        return None;
    }
    let content = match std::fs::read_to_string(&mnemnk_json) {
        Ok(ret) => ret,
        Err(e) => {
            log::warn!("Failed to read agent definition file: {}", e);
            return None;
        }
    };
    let def: AgentDefinition = match serde_json::from_str(&content) {
        Ok(def) => def,
        Err(e) => {
            log::warn!("Failed to parse agent definition file: {}", e);
            return None;
        }
    };

    Some(def)
}

fn post_process_agent_def(
    mut def: AgentDefinition,
    agent_dir: &PathBuf,
) -> Option<AgentDefinition> {
    let agent_dir_name = agent_dir.file_name().unwrap().to_string_lossy().to_string();

    // check if name and def.name match
    if def.name != agent_dir_name {
        log::warn!(
            "Agent name and definition name mismatch: {} != {}",
            agent_dir_name,
            def.name
        );
        return None;
    }

    if def.kind == "command" {
        // set path
        let mut agent_path = def.path.clone().unwrap_or_default();
        if agent_path.is_empty() {
            agent_path = def.name.clone();
        }
        let mut path = PathBuf::from(&agent_path);
        if path.is_absolute() {
            log::warn!(
                "Absolute path is not allowed (agent: {}): {}",
                def.name,
                agent_path,
            );
            return None;
        }
        path = agent_dir
            .join(path)
            .with_extension(env::consts::EXE_EXTENSION);
        if !path.exists() {
            log::warn!(
                "Agent path not found (agent: {}): {}",
                def.name,
                path.display()
            );
            return None;
        }
        def.path = Some(path.to_string_lossy().to_string());
    }

    Some(def)
}
