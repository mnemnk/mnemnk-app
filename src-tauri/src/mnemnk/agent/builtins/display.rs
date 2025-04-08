use anyhow::Result;
use serde::Serialize;
use serde_json::{json, Value};
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentData, AgentDefinition, AgentDefinitions, AgentDisplayConfigEntry,
    AsAgent, AsAgentData,
};

// Display Value
struct DisplayDataAgent {
    data: AsAgentData,
}

impl AsAgent for DisplayDataAgent {
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

    fn input(&mut self, _ch: String, data: AgentData) -> Result<()> {
        let display_data = DisplayData {
            kind: data.kind,
            value: data.value,
        };
        self.emit_display("data".to_string(), json!(display_data))
    }
}

#[derive(Debug, Clone, Serialize)]
struct DisplayData {
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

    fn input(&mut self, _ch: String, data: AgentData) -> Result<()> {
        let display_data = DisplayData {
            kind: "messages".to_string(),
            value: data.value,
        };
        self.emit_display("messages".to_string(), json!(display_data))
    }
}

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    // Display Data
    defs.insert(
        "$display_data".into(),
        AgentDefinition::new(
            "DisplayData",
            "$display_data",
            Some(new_boxed::<DisplayDataAgent>),
        )
        .with_title("Display Data")
        // .with_description("Display value on the node")
        .with_category("Display")
        .with_inputs(vec!["*"])
        .with_display_config(vec![(
            "data".into(),
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
        .with_inputs(vec!["messages"])
        .with_display_config(vec![(
            "messages".into(),
            AgentDisplayConfigEntry::new("messages"),
        )]),
    );
}
