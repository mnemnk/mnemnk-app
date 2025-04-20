use anyhow::Result;
use serde::Serialize;
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentData, AgentDefinition, AgentDefinitions, AgentDisplayConfigEntry,
    AsAgent, AsAgentData,
};

// Display Data
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
        self.emit_display("data".to_string(), data)
    }
}

// Debug Data
struct DebugDataAgent {
    data: AsAgentData,
}

impl AsAgent for DebugDataAgent {
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

    fn input(&mut self, ch: String, data: AgentData) -> Result<()> {
        #[derive(Serialize)]
        struct DebugData {
            ch: String,
            data: AgentData,
        }
        let debug_data = DebugData { ch, data };
        let data = AgentData::new_object(serde_json::to_value(&debug_data)?);
        self.emit_display("data".to_string(), data)
    }
}

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    // Display Data
    defs.insert(
        "$display_data".into(),
        AgentDefinition::new(
            "Builtin",
            "$display_data",
            Some(new_boxed::<DisplayDataAgent>),
        )
        .with_title("Display Data")
        .with_category("Core/Display")
        .with_inputs(vec!["data"])
        .with_display_config(vec![(
            "data".into(),
            AgentDisplayConfigEntry::new("*").with_hide_title(),
        )]),
    );

    // Debug Data
    defs.insert(
        "$debug_data".into(),
        AgentDefinition::new("Builtin", "$debug_data", Some(new_boxed::<DebugDataAgent>))
            .with_title("Debug Data")
            .with_category("Core/Display")
            .with_inputs(vec!["*"])
            .with_display_config(vec![(
                "data".into(),
                AgentDisplayConfigEntry::new("object").with_hide_title(),
            )]),
    );
}
