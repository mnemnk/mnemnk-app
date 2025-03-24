use anyhow::{Context as _, Result};
use jsonpath_rust::JsonPath;
use serde_json::{json, Value};
use std::collections::HashMap;
use tauri::{AppHandle, Manager};

use super::agent::{AgentConfig, AgentData, AsAgent};
use super::definition::{AgentDefaultConfigEntry, AgentDefinition, AgentDefinitions};
use super::env::AgentEnv;
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

    fn input(&mut self, app: &AppHandle, kind: String, value: Value) -> Result<()> {
        message::try_send_store(app, kind, value)
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

pub struct JsonPathAgent {
    data: AgentData,
}

impl AsAgent for JsonPathAgent {
    fn data(&self) -> &AgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AgentData {
        &mut self.data
    }

    fn input(&mut self, app: &AppHandle, kind: String, value: Value) -> Result<()> {
        let config = self.data.config.as_ref().context("Missing config")?;
        let jsonpath = config
            .get("jsonpath")
            .context("Missing jsonpath")?
            .as_str()
            .context("jsonpath is not a string")?;
        let data = json!(vec![value]);
        let result: Vec<&Value> = data.query(jsonpath).map_err(|e| {
            log::error!("Failed to query jsonpath: {}", e);
            e
        })?;

        let env = app.state::<AgentEnv>();
        for r in result {
            message::try_send_agent_out(&env, self.data.id.clone(), kind.clone(), r.clone());
        }
        Ok(())
    }
}

impl JsonPathAgent {
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
                AgentDefaultConfigEntry::new(json!(""), "string?")
                    .with_title("Board Name")
                    .with_description("If empty, the source label wll be used"),
            )])),
    );

    // DatabaseAgent
    defs.insert(
        "$database".into(),
        AgentDefinition::new("Database", "$database")
            .with_title("Database")
            .with_description("Store data")
            .with_inputs(vec!["*"]),
    );

    // JsonPathAgent
    defs.insert(
        "$jsonpath".into(),
        AgentDefinition::new("JsonPath", "$jsonpath")
            .with_title("JsonPath")
            .with_inputs(vec!["*"])
            .with_outputs(vec!["*"])
            .with_default_config(HashMap::from([(
                "jsonpath".into(),
                AgentDefaultConfigEntry::new(json!("$[*]"), "string")
                    .with_title("JSON Path")
                    .with_description(r#"ex. $[?search(@.url, "https://github.com/.*")]"#),
            )])),
    );

    defs
}
