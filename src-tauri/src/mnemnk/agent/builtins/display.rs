use anyhow::Result;
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

    fn input(&mut self, _ch: String, data: AgentData) -> Result<()> {
        self.emit_display("data".to_string(), data)
    }
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
        self.emit_display("messages".to_string(), data)
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
        .with_category("Display")
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
            .with_category("Display")
            .with_inputs(vec!["*"])
            .with_display_config(vec![(
                "data".into(),
                AgentDisplayConfigEntry::new("object").with_hide_title(),
            )]),
    );

    // Display Messages
    defs.insert(
        "$display_messages".into(),
        AgentDefinition::new(
            "Builtin",
            "$display_messages",
            Some(new_boxed::<DisplayMessagesAgent>),
        )
        .with_title("Display Messages")
        .with_description("Display messages on the node")
        .with_category("LLM")
        .with_inputs(vec!["messages"])
        .with_display_config(vec![(
            "messages".into(),
            AgentDisplayConfigEntry::new("messages").with_hide_title(),
        )]),
    );
}
