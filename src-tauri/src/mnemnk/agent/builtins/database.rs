use anyhow::Result;
use serde_json::Value;
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::{
    try_send_store, Agent, AgentConfig, AgentData, AgentDefinition, AgentDefinitions, AsAgent,
};

// Database

struct DatabaseAgent {
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
        try_send_store(self.app(), kind, value)
    }
}

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    // DatabaseAgent
    defs.insert(
        "$database".into(),
        AgentDefinition::new("Database", "$database", Some(new_boxed::<DatabaseAgent>))
            .with_title("Database")
            .with_description("Store data")
            .with_category("Core/Database")
            .with_inputs(vec!["*"]),
    );
}
