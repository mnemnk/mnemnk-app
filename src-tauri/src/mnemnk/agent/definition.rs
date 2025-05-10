use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use thiserror::Error;

use super::agent::AsyncAgent;
use super::builtins;
use super::config::AgentConfig;
use super::data::AgentValue;
use crate::mnemnk::settings;

static AGENTS_DIR: &str = "agents";
static MNEMNK_JSON: &str = "mnemnk.json";
static MNEMNK_LOCAL_JSON: &str = "mnemnk.local.json";

pub static AGENT_KIND_BUILTIN: &str = "Builtin";
pub static AGENT_KIND_DATABASE: &str = "Database";

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
    pub category: Option<String>,
    pub inputs: Option<Vec<String>>,
    pub outputs: Option<Vec<String>>,
    pub default_config: Option<AgentDefaultConfig>,
    pub global_config: Option<AgentGlobalConfig>,
    pub display_config: Option<AgentDisplayConfig>,

    // CommandAgent
    pub command: Option<CommandConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub native_thread: Option<bool>,

    #[serde(skip)]
    pub new_boxed: Option<AgentNewBoxedFn>,
}

pub type AgentDefaultConfig = Vec<(String, AgentConfigEntry)>;
pub type AgentGlobalConfig = Vec<(String, AgentConfigEntry)>;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AgentConfigEntry {
    pub value: AgentValue,

    #[serde(rename = "type")]
    pub type_: Option<String>,

    pub title: Option<String>,

    pub description: Option<String>,

    /// Indicates whether this configuration entry should be hidden from the user interface.
    /// If set to `Some(true)`, the entry will be hidden. If `None`, the default behavior is to show the entry.
    pub hidden: Option<bool>,
}

pub type AgentDisplayConfig = Vec<(String, AgentDisplayConfigEntry)>;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AgentDisplayConfigEntry {
    #[serde(rename = "type")]
    pub type_: Option<String>,

    pub title: Option<String>,
    pub description: Option<String>,
    pub hide_title: Option<bool>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct CommandConfig {
    pub cmd: String,
    pub args: Option<Vec<String>>,

    pub dir: Option<String>,
}

pub type AgentNewBoxedFn = fn(
    app: AppHandle,
    id: String,
    def_name: String,
    config: Option<AgentConfig>,
) -> Result<Box<dyn AsyncAgent>>;

impl AgentDefinition {
    pub fn new(
        kind: impl Into<String>,
        name: impl Into<String>,
        new_boxed: Option<AgentNewBoxedFn>,
    ) -> Self {
        Self {
            kind: kind.into(),
            name: name.into(),
            new_boxed,
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

    pub fn with_category(mut self, category: &str) -> Self {
        self.category = Some(category.into());
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

    pub fn with_default_config(mut self, config: AgentDefaultConfig) -> Self {
        self.default_config = Some(config);
        self
    }

    #[allow(unused)]
    pub fn with_global_config(mut self, config: AgentGlobalConfig) -> Self {
        self.global_config = Some(config);
        self
    }

    pub fn with_display_config(mut self, config: AgentDisplayConfig) -> Self {
        self.display_config = Some(config);
        self
    }

    pub fn use_native_thread(mut self) -> Self {
        self.native_thread = Some(true);
        self
    }
}

impl AgentConfigEntry {
    pub fn new(value: AgentValue, type_: &str) -> Self {
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

    pub fn with_hidden(mut self, hidden: bool) -> Self {
        self.hidden = Some(hidden);
        self
    }
}

impl AgentDisplayConfigEntry {
    pub fn new(type_: &str) -> Self {
        Self {
            type_: Some(type_.into()),
            ..Default::default()
        }
    }

    pub fn with_hide_title(mut self) -> Self {
        self.hide_title = Some(true);
        self
    }

    #[allow(unused)]
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = Some(title.into());
        self
    }

    #[allow(unused)]
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.into());
        self
    }
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
    let mut defs: AgentDefinitions = Default::default();

    builtins::init_agent_defs(&mut defs);
    read_mnemnk_jsons(app, &mut defs)?;

    Ok(defs)
}

fn read_mnemnk_jsons(app: &AppHandle, defs: &mut AgentDefinitions) -> Result<()> {
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
            if let Err(e) = post_process_agent_def(&mut def, &agent_dir) {
                log::error!("Failed to post process agent definition: {}", e);
                continue;
            }
            defs.insert(def.name.clone(), def);
        }
    }

    Ok(())
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
        let mnemnk_json: MnemnkJson = match serde_json::from_str(&content) {
            Ok(json) => json,
            Err(e) => {
                log::error!("Invalid JSON {}: {}", mnemnk_local_json_file.display(), e);
                return None;
            }
        };
        return Some(mnemnk_json);
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
        "Command" => builtins::CommandAgent::read_def(def, agent_dir)?,
        _ => {}
    }
    Ok(())
}
