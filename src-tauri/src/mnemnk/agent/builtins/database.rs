use anyhow::Result;
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentData, AgentDefinition, AgentDefinitions, AsAgent, AsAgentData,
};
use crate::mnemnk::store;

// Event Database

struct EventDatabaseAgent {
    data: AsAgentData,
}

impl AsAgent for EventDatabaseAgent {
    fn new(
        app: AppHandle,
        id: String,
        def_name: String,
        config: Option<AgentConfig>,
    ) -> Result<Self> {
        Ok(Self {
            data: AsAgentData::new(app, id, def_name, config),
        })
    }

    fn data(&self) -> &AsAgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AsAgentData {
        &mut self.data
    }

    fn process(&mut self, _ch: String, data: AgentData) -> Result<()> {
        store::store(self.app(), data);
        Ok(())
    }
}

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    // DatabaseAgent
    defs.insert(
        "$event_database".into(),
        AgentDefinition::new(
            "Database",
            "$event_database",
            Some(new_boxed::<EventDatabaseAgent>),
        )
        .with_title("Event Database")
        .with_description("Store events in the database")
        .with_category("Core/Database")
        .with_inputs(vec!["event"]),
    );
}
