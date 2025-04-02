use anyhow::{Context as _, Result};
use jsonpath_rust::JsonPath;
use serde_json::{json, Value};
use std::collections::HashMap;
use tauri::AppHandle;

use super::agent::{Agent, AgentConfig, AgentData, AsAgent};
use super::definition::{AgentConfigEntry, AgentDefinition, AgentDefinitions};
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

    fn input(&mut self, kind: String, value: Value) -> Result<()> {
        message::try_send_store(self.app(), kind, value)
    }
}

impl DatabaseAgent {
    pub fn new(
        app: AppHandle,
        id: String,
        def_name: String,
        config: Option<AgentConfig>,
    ) -> Result<Self> {
        Ok(Self {
            data: AgentData {
                app,
                id,
                status: Default::default(),
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

    fn input(&mut self, kind: String, value: Value) -> Result<()> {
        let config = self.data.config.as_ref().context("Missing config")?;
        let jsonpath = config
            .get("jsonpath")
            .context("Missing jsonpath")?
            .as_str()
            .context("jsonpath is not a string")?;
        let data = json!(vec![value]);
        let result: Vec<&Value> = data.query(jsonpath).context("Failed to query jsonpath")?;

        for r in result {
            self.try_output(kind.clone(), r.clone())
                .context("Failed to output jsonpath result")?;
        }
        Ok(())
    }
}

impl JsonPathAgent {
    pub fn new(
        app: AppHandle,
        id: String,
        def_name: String,
        config: Option<AgentConfig>,
    ) -> Result<Self> {
        Ok(Self {
            data: AgentData {
                app,
                id,
                status: Default::default(),
                def_name,
                config,
            },
        })
    }
}

pub fn builtin_agent_defs() -> AgentDefinitions {
    let mut defs: AgentDefinitions = Default::default();

    // BoardInAgent
    defs.insert(
        "$board_in".into(),
        AgentDefinition::new("BoardIn", "$board_in")
            .with_title("Board In")
            .with_inputs(vec!["*"])
            .with_default_config(HashMap::from([(
                "board_name".into(),
                AgentConfigEntry::new(json!(""), "string")
                    .with_title("Board Name")
                    .with_description("* = source kind"),
            )])),
    );

    // BoardOutAgent
    defs.insert(
        "$board_out".into(),
        AgentDefinition::new("BoardOut", "$board_out")
            .with_title("Board Out")
            .with_outputs(vec!["*"])
            .with_default_config(HashMap::from([(
                "board_name".into(),
                AgentConfigEntry::new(json!(""), "string").with_title("Board Name"),
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
                AgentConfigEntry::new(json!("$[*]"), "string")
                    .with_title("JSON Path")
                    .with_description(r#"ex. $[?search(@.url, "https://github.com/.*")]"#),
            )])),
    );

    defs
}
