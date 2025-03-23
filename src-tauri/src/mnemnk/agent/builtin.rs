use anyhow::Result;
use serde_json::{json, Value};
use std::collections::HashMap;
use tauri::AppHandle;

use super::agent::{AgentConfig, AgentData, AsAgent};
use super::definition::{AgentDefaultConfigEntry, AgentDefinition, AgentDefinitions};
use super::message;

pub struct DatabaseAgent {
    data: AgentData,
}

impl AsAgent for DatabaseAgent {
    fn data(&self) -> &AgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AgentData {
        &mut self.data
    }

    fn input(&mut self, app: &AppHandle, source: String, kind: String, value: Value) -> Result<()> {
        message::send_store(app, source, kind, value)
    }
}

impl DatabaseAgent {
    pub fn new(id: String, def_name: String, config: Option<AgentConfig>) -> Result<Self> {
        Ok(Self {
            data: AgentData {
                id,
                def_name,
                config,
            },
        })
    }
}

pub fn builtin_agent_defs() -> AgentDefinitions {
    let mut defs: AgentDefinitions = Default::default();

    // BoardAgent
    defs.insert(
        "$board".into(),
        AgentDefinition::new("Board", "$board")
            .with_title("Board")
            .with_inputs(vec!["*"])
            .with_outputs(vec!["*"])
            .with_default_config(HashMap::from([(
                "board_name".into(),
                AgentDefaultConfigEntry::new(json!(""), "string?").with_title("Board Name"),
            )])),
    );

    // DatabaseAgent
    defs.insert(
        "$database".into(),
        AgentDefinition::new("Database", "$database")
            .with_title("Database")
            .with_inputs(vec!["*"]),
    );

    defs
}
