use std::vec;

use anyhow::{bail, Context as _, Result};
use serde_json::json;
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentConfigEntry, AgentData, AgentDefinition, AgentDefinitions, AgentValue,
    AsAgent, AsAgentData,
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
        store::create_event(self.app(), data)
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

    fn process(&mut self, _ch: String, data: AgentData) -> Result<()> {
        let config = self.data.config.as_ref().context("Missing config")?;
        let db = config
            .get("db")
            .context("Missing db config")?
            .as_str()
            .context("db is not a string")?;
        if db.is_empty() {
            // db is not set
            bail!("db is not set");
        }

        let table = config
            .get("table")
            .context("Missing table config")?
            .as_str()
            .context("table is not a string")?;
        let table = if table.is_empty() {
            // table is not set
            bail!("table is not set");
        } else {
            table.to_string()
        };

        let key = if let Some(key) = &data
            .as_object()
            .context("data is not an object")?
            .get("key")
            .cloned()
        {
            key.as_str().context("key is not a string")?.to_string()
        } else {
            bail!("key not found");
        };

        let Some(value) = &data
            .as_object()
            .context("data is not an object")?
            .get("value")
            .cloned()
        else {
            bail!("value not found");
        };
        if !value.is_object() {
            bail!("value is not an object");
        }

        store::insert(self.app(), db.to_string(), table, key, value.clone())
    }
}

// Database Select
struct DatabaseSelectAgent {
    data: AsAgentData,
}

impl AsAgent for DatabaseSelectAgent {
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
        let config = self.data.config.as_ref().context("Missing config")?;
        let db = config
            .get("db")
            .context("Missing db config")?
            .as_str()
            .context("db is not a string")?;
        if db.is_empty() {
            bail!("db is not set");
        }

        let table = config
            .get("table")
            .context("Missing table config")?
            .as_str()
            .context("table is not a string")?;
        let table = if table.is_empty() {
            bail!("table is not set");
        } else {
            table.to_string()
        };

        let key = data.as_str().context("key is not a string")?;
        if key.is_empty() {
            bail!("key is not set");
        }

        let result = store::select(self.app(), db.to_string(), table, key.to_string())?;
        if let Some(value) = result {
            let data = AgentData::new_object(json!({
                "key": key,
                "value": value,
            }));
            self.try_output("kv".to_string(), data)?;
        } else {
            // value is empty
            self.try_output("kv".to_string(), AgentData::new_unit())?;
        }
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
        .with_category("Core/Database")
        .with_inputs(vec!["kv"])
        .with_default_config(vec![
            (
                "db".into(),
                AgentConfigEntry::new(AgentValue::new_string(""), "string"),
            ),
            (
                "table".into(),
                AgentConfigEntry::new(AgentValue::new_string(""), "string"),
            ),
        ]),
    );

    // Database Select
    defs.insert(
        "$database_select".into(),
        AgentDefinition::new(
            "Database",
            "$database_select",
            Some(new_boxed::<DatabaseSelectAgent>),
        )
        .with_title("Database Select")
        .with_category("Core/Database")
        .with_inputs(vec!["key"])
        .with_outputs(vec!["kv"])
        .with_default_config(vec![
            (
                "db".into(),
                AgentConfigEntry::new(AgentValue::new_string(""), "string"),
            ),
            (
                "table".into(),
                AgentConfigEntry::new(AgentValue::new_string(""), "string"),
            ),
        ]),
    );
}
