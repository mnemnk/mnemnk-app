use anyhow::{bail, Context as _, Result};
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
        store::create_event(self.app(), data);
        Ok(())
    }
}

// Database Insert
struct DatabaseInsertAgent {
    data: AsAgentData,
}

impl AsAgent for DatabaseInsertAgent {
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

    fn process(&mut self, ch: String, data: AgentData) -> Result<()> {
        let Some(json_value) = data.value.as_object().cloned() else {
            bail!("data is not an object");
        };

        let key = if let Some(key) = json_value.get("key").cloned() {
            key.as_str().context("key is not a string")?.to_string()
        } else {
            bail!("key not found");
        };

        store::insert(self.app(), ch, key, json_value);
        Ok(())
    }
}

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    // Event Database
    defs.insert(
        "$event_database".into(),
        AgentDefinition::new(
            "Database",
            "$event_database",
            Some(new_boxed::<EventDatabaseAgent>),
        )
        .with_title("Event Database")
        .with_category("Core/Database")
        .with_inputs(vec!["event"]),
    );

    // Database Insert
    defs.insert(
        "$database_insert".into(),
        AgentDefinition::new(
            "Database",
            "$database_insert",
            Some(new_boxed::<DatabaseInsertAgent>),
        )
        .with_title("Database Insert")
        .with_description("Insert data into ch table. The data must be a JSON object with a key.")
        .with_category("Core/Database")
        .with_inputs(vec!["*"]),
    );
}
