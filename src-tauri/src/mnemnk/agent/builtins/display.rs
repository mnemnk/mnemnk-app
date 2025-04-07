use anyhow::Result;
use serde::Serialize;
use serde_json::{json, Value};
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AsAgentData, AgentDefinition, AgentDefinitions, AgentDisplayConfigEntry,
    AsAgent,
};

// Display Value
struct DisplayValueAgent {
    data: AsAgentData,
}

impl AsAgent for DisplayValueAgent {
    fn new(
        app: AppHandle,
        id: String,
        def_name: String,
        config: Option<AgentConfig>,
    ) -> Result<Self> {
        Ok(Self {
            data: AsAgentData {
                app,
                id,
                status: Default::default(),
                def_name,
                config,
            },
        })
    }

    fn data(&self) -> &AsAgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AsAgentData {
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

// Display Messages
struct DisplayMessagesAgent {
    data: AsAgentData,
}

impl AsAgent for DisplayMessagesAgent {
    fn new(
        app: AppHandle,
        id: String,
        def_name: String,
        config: Option<AgentConfig>,
    ) -> Result<Self> {
        Ok(Self {
            data: AsAgentData {
                app,
                id,
                status: Default::default(),
                def_name,
                config,
            },
        })
    }

    fn data(&self) -> &AsAgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AsAgentData {
        &mut self.data
    }

    fn input(&mut self, _kind: String, value: Value) -> Result<()> {
        let display_value = DisplayValue {
            kind: "messages".to_string(),
            value,
        };
        self.emit_display("messages".to_string(), json!(display_value))
    }
}

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
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
        .with_category("Display")
        .with_inputs(vec!["*"])
        .with_display_config(vec![(
            "value".into(),
            AgentDisplayConfigEntry::new("object"),
        )]),
    );

    // Display Messages
    defs.insert(
        "$display_messages".into(),
        AgentDefinition::new(
            "DisplayMessages",
            "$display_messages",
            Some(new_boxed::<DisplayMessagesAgent>),
        )
        .with_title("Display Messages")
        .with_description("Display messages on the node")
        .with_category("LLM")
        .with_inputs(vec!["*"])
        .with_display_config(vec![(
            "messages".into(),
            AgentDisplayConfigEntry::new("messages"),
        )]),
    );
}
