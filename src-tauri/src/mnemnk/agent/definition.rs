use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::AppHandle;
use thiserror::Error;

use super::{builtin::builtin_agent_defs, command::CommandAgent};
use crate::mnemnk::settings;

static AGENTS_DIR: &str = "agents";
static MNEMNK_JSON: &str = "mnemnk.json";
static MNEMNK_LOCAL_JSON: &str = "mnemnk.local.json";

#[derive(Debug, Error)]
pub enum AgentDefinitionError {
    #[error("{0}: Agent definition \"{1}\" is missing")]
    MissingEntry(String, String),

    #[error("{0}: Agent definition \"{1}\" is invalid")]
    InvalidEntry(String, String),
}

pub type AgentDefinitions = HashMap<String, AgentDefinition>;

#[derive(Deserialize)]
pub struct MnemnkJson {
    pub agents: Option<Vec<AgentDefinition>>,
}

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
    pub command: Option<CommandConfig>,
}

pub type AgentDefaultConfig = HashMap<String, AgentDefaultConfigEntry>;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AgentDefaultConfigEntry {
    pub value: Value,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub title: Option<String>,
    pub description: Option<String>,
    // pub scope: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct CommandConfig {
    pub cmd: String,
    pub args: Option<Vec<String>>,
    // pub env: Option<HashMap<String, String>>,
    // pub shell: Option<bool>,

    // set in read_def
    pub dir: Option<String>,
}

impl AgentDefinition {
    pub fn new(kind: &str, name: &str) -> Self {
        Self {
            kind: kind.into(),
            name: name.into(),
            ..Default::default()
        }
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.into());
        self
    }

    pub fn with_inputs(mut self, inputs: Vec<&str>) -> Self {
        self.inputs = Some(inputs.into_iter().map(|x| x.into()).collect());
        self
    }

    pub fn with_outputs(mut self, outputs: Vec<&str>) -> Self {
        self.outputs = Some(outputs.into_iter().map(|x| x.into()).collect());
        self
    }

    pub fn with_default_config(mut self, config: HashMap<String, AgentDefaultConfigEntry>) -> Self {
        self.default_config = Some(config);
        self
    }
}

impl AgentDefaultConfigEntry {
    pub fn new(value: Value, type_: &str) -> Self {
        Self {
            value,
            type_: Some(type_.into()),
            ..Default::default()
        }
    }

    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.into());
        self
    }

    // pub fn with_scope(mut self, scope: &str) -> Self {
    //     self.scope = Some(scope.into());
    //     self
    // }
}

pub fn agents_dir(app: &AppHandle) -> Option<PathBuf> {
    let mnemnk_dir = settings::mnemnk_dir(app);
    if mnemnk_dir.is_none() {
        return None;
    }
    let agents_dir = PathBuf::from(mnemnk_dir.unwrap()).join(AGENTS_DIR);
    if !agents_dir.exists() {
        if let Err(e) = std::fs::create_dir(&agents_dir) {
            log::error!("Failed to create agents directory: {}", e);
            return None;
        };
    }
    Some(agents_dir)
}

pub(super) fn init_agent_defs(app: &AppHandle) -> Result<AgentDefinitions> {
    let mut defs = builtin_agent_defs();
    defs.extend(read_mnemnk_jsons(app)?);
    Ok(defs)
}

fn read_mnemnk_jsons(app: &AppHandle) -> Result<AgentDefinitions> {
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
        let Some(mnemnk_json) = read_mnemnk_json(&agent_dir) else {
            continue;
        };
        for def in mnemnk_json.agents.unwrap_or_default() {
            let mut def = def;
            post_process_agent_def(&mut def, &agent_dir)?;
            defs.insert(def.name.clone(), def);
        }
    }

    Ok(defs)
}

fn read_mnemnk_json(agent_dir: &PathBuf) -> Option<MnemnkJson> {
    // If mnemnk.local.json exists, prioritize reading it
    let mnemnk_local_json_file = agent_dir.join(MNEMNK_LOCAL_JSON);
    if mnemnk_local_json_file.exists() {
        let content = match std::fs::read_to_string(&mnemnk_local_json_file) {
            Ok(ret) => ret,
            Err(e) => {
                log::error!("I/O Error {}: {}", mnemnk_local_json_file.display(), e);
                return None;
            }
        };
        let def: MnemnkJson = match serde_json::from_str(&content) {
            Ok(def) => def,
            Err(e) => {
                log::error!("Invalid JSON {}: {}", mnemnk_local_json_file.display(), e);
                return None;
            }
        };
        return Some(def);
    }

    let mnemnk_json_file = agent_dir.join(MNEMNK_JSON);
    if !mnemnk_json_file.exists() {
        return None;
    }
    let content = match std::fs::read_to_string(&mnemnk_json_file) {
        Ok(ret) => ret,
        Err(e) => {
            log::warn!("Failed to read agent definition file: {}", e);
            return None;
        }
    };
    let mnemnk_json: MnemnkJson = match serde_json::from_str(&content) {
        Ok(json) => json,
        Err(e) => {
            log::warn!("Failed to parse agent definition file: {}", e);
            return None;
        }
    };
    Some(mnemnk_json)
}

fn post_process_agent_def(def: &mut AgentDefinition, agent_dir: &PathBuf) -> Result<()> {
    match def.kind.as_str() {
        "Command" => CommandAgent::read_def(def, agent_dir)?,
        _ => {}
    }
    Ok(())
}
