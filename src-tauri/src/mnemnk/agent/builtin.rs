use anyhow::{Context as _, Result};
use regex::Regex;
use serde::Serialize;
use serde_json::{json, Value};
use tauri::AppHandle;

use super::agent::{new_boxed, Agent, AgentConfig, AgentData, AsAgent};
use super::definition::{
    AgentConfigEntry, AgentDefinition, AgentDefinitions, AgentDisplayConfigEntry,
};
use super::message;

// Database

pub struct DatabaseAgent {
    data: AgentData,
}

impl AsAgent for DatabaseAgent {
    fn new(
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

// Display Value

pub struct DisplayValueAgent {
    data: AgentData,
}

impl AsAgent for DisplayValueAgent {
    fn new(
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

    fn data(&self) -> &AgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AgentData {
        &mut self.data
    }

    fn input(&mut self, kind: String, value: Value) -> Result<()> {
        let display_value = DisplayValue { kind, value };
        self.emit_display("value".to_string(), json!(display_value))
    }
}

#[derive(Debug, Clone, Serialize)]
struct DisplayValue {
    kind: String,
    value: Value,
}

// Regex Filter

pub struct RegexFilterAgent {
    data: AgentData,
}

impl AsAgent for RegexFilterAgent {
    fn new(
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

    fn data(&self) -> &AgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AgentData {
        &mut self.data
    }

    fn input(&mut self, kind: String, value: Value) -> Result<()> {
        let config = self.data.config.as_ref().context("Missing config")?;

        let field = config
            .get("field")
            .context("Missing field")?
            .as_str()
            .context("field is not a string")?;
        if field.is_empty() {
            // field is not set
            return Ok(());
        }

        let regex = config
            .get("regex")
            .context("Missing regex")?
            .as_str()
            .context("regex is not a string")?;
        if regex.is_empty() {
            // regex is not set
            return Ok(());
        }
        let regex = Regex::new(regex).context("Failed to compile regex")?;

        let Some(field_value) = value.get(field) else {
            // value does not have the field
            return Ok(());
        };
        let field_value = field_value
            .as_str()
            .context("value is not a string")?
            .to_string();
        if regex.is_match(&field_value) {
            // value matches the regex
            self.try_output(kind.clone(), value.into())
                .context("Failed to output regex result")?;
        }

        Ok(())
    }
}

pub fn builtin_agent_defs() -> AgentDefinitions {
    let mut defs: AgentDefinitions = Default::default();

    // BoardInAgent
    defs.insert(
        "$board_in".into(),
        AgentDefinition::new(
            "BoardIn",
            "$board_in",
            Some(new_boxed::<super::board::BoardInAgent>),
        )
        .with_title("Board In")
        .with_inputs(vec!["*"])
        .with_default_config(vec![(
            "board_name".into(),
            AgentConfigEntry::new(json!(""), "string")
                .with_title("Board Name")
                .with_description("* = source kind"),
        )]),
    );

    // BoardOutAgent
    defs.insert(
        "$board_out".into(),
        AgentDefinition::new(
            "BoardOut",
            "$board_out",
            Some(new_boxed::<super::board::BoardOutAgent>),
        )
        .with_title("Board Out")
        .with_outputs(vec!["*"])
        .with_default_config(vec![(
            "board_name".into(),
            AgentConfigEntry::new(json!(""), "string").with_title("Board Name"),
        )]),
    );

    // DatabaseAgent
    defs.insert(
        "$database".into(),
        AgentDefinition::new("Database", "$database", Some(new_boxed::<DatabaseAgent>))
            .with_title("Database")
            .with_description("Store data")
            .with_inputs(vec!["*"]),
    );

    // Display Value
    defs.insert(
        "$display_value".into(),
        AgentDefinition::new(
            "DisplayValue",
            "$display_value",
            Some(new_boxed::<DisplayValueAgent>),
        )
        .with_title("Display Value")
        // .with_description("Display value on the node")
        .with_inputs(vec!["*"])
        .with_display_config(vec![(
            "value".into(),
            AgentDisplayConfigEntry::new("object"),
        )]),
    );

    // RegexFilterAgent
    defs.insert(
        "$regex_filter".into(),
        AgentDefinition::new(
            "RegexFilter",
            "$regex_filter",
            Some(new_boxed::<RegexFilterAgent>),
        )
        .with_title("Regex Filter")
        .with_inputs(vec!["*"])
        .with_outputs(vec!["*"])
        .with_default_config(vec![
            (
                "field".into(),
                AgentConfigEntry::new(json!(""), "string").with_title("Field"),
            ),
            (
                "regex".into(),
                AgentConfigEntry::new(json!(""), "string").with_title("Regex"),
            ),
        ]),
    );

    defs
}
