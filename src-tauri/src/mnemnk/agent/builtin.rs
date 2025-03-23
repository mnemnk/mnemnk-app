use anyhow::Result;
use std::collections::HashMap;
use tauri::AppHandle;

use serde_json::Value;

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

    fn input(&self, app: &AppHandle, source: String, kind: String, value: Value) -> Result<()> {
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

    defs.insert(
        "$board".to_string(),
        AgentDefinition {
            kind: "board".to_string(),
            name: "$board".to_string(),
            title: Some("Board".to_string()),
            description: None,
            inputs: Some(vec!["*".to_string()]),
            outputs: Some(vec!["*".to_string()]),
            default_config: Some(HashMap::from([(
                "board_name".to_string(),
                AgentDefaultConfigEntry {
                    value: Value::String("".to_string()),
                    type_: Some("string?".to_string()),
                    title: Some("Board Name".to_string()),
                    description: None,
                    scope: None,
                },
            )])),
            path: None,
        },
    );

    defs.insert(
        "$database".to_string(),
        AgentDefinition {
            kind: "database".to_string(),
            name: "$database".to_string(),
            title: Some("Database".to_string()),
            description: None,
            inputs: Some(vec!["*".to_string()]),
            outputs: None,
            default_config: None,
            path: None,
        },
    );

    defs
}
