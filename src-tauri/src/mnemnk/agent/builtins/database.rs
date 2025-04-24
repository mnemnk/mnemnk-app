use std::vec;

use anyhow::{bail, Context as _, Ok, Result};
use serde_json::json;
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentConfigEntry, AgentContext, AgentData, AgentDefinition,
    AgentDefinitions, AgentValue, AsAgent, AsAgentData,
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

// Database Delete
struct DatabaseDeleteAgent {
    data: AsAgentData,
}

impl AsAgent for DatabaseDeleteAgent {
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

        let return_before = config
            .get("return_before")
            .context("Missing return_before")?
            .as_bool()
            .context("return_before is not a boolean")?;

        let key = data.as_str().context("key is not a string")?;
        if key.is_empty() {
            bail!("key is empty");
        }

        let result = store::delete(
            self.app(),
            db.to_string(),
            table,
            key.to_string(),
            return_before,
        )?;
        if return_before {
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
        } else {
            // return_before is false
            self.try_output("kv".to_string(), AgentData::new_unit())?;
        }

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
        if key.is_empty() {
            bail!("key is empty");
        }

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

        store::insert(self.app(), db.to_string(), table, key, value.clone())?;

        self.try_output(ch, data)
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
            bail!("key is empty");
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

// Database Update
struct DatabaseUpdateAgent {
    data: AsAgentData,
}

impl AsAgent for DatabaseUpdateAgent {
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
        if key.is_empty() {
            bail!("key is empty");
        }

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

        store::update(self.app(), db.to_string(), table, key, value.clone())?;

        self.try_output(ch, data)
    }
}

// Database Update Merge
struct DatabaseUpdateMergeAgent {
    data: AsAgentData,
}

impl AsAgent for DatabaseUpdateMergeAgent {
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
        if key.is_empty() {
            bail!("key is empty");
        }

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

        let return_after = config
            .get("return_after")
            .context("Missing return_after")?
            .as_bool()
            .context("return_after is not a boolean")?;

        let result = store::update_merge(
            self.app(),
            db.to_string(),
            table,
            key.to_string(),
            value.clone(),
            return_after,
        )?;
        if return_after {
            if let Some(value) = result {
                let data = AgentData::new_object(json!({
                    "key": key,
                    "value": value,
                }));
                self.try_output(ch, data)?;
            } else {
                // value is empty
                self.try_output(ch, AgentData::new_unit())?;
            }
        } else {
            // return_after is false
            self.try_output(ch, AgentData::new_unit())?;
        }

        Ok(())
    }
}

// Database Upsert
struct DatabaseUpsertAgent {
    data: AsAgentData,
}

impl AsAgent for DatabaseUpsertAgent {
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
        if key.is_empty() {
            bail!("key is empty");
        }

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

        store::upsert(self.app(), db.to_string(), table, key, value.clone())?;

        self.try_output(ch, data)
    }
}

// Database Upsert Merge
struct DatabaseUpsertMergeAgent {
    data: AsAgentData,
}

impl AsAgent for DatabaseUpsertMergeAgent {
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
        if key.is_empty() {
            bail!("key is empty");
        }

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

        let return_after = config
            .get("return_after")
            .context("Missing return_after")?
            .as_bool()
            .context("return_after is not a boolean")?;

        let result = store::upsert_merge(
            self.app(),
            db.to_string(),
            table,
            key.to_string(),
            value.clone(),
            return_after,
        )?;
        if return_after {
            if let Some(value) = result {
                let data = AgentData::new_object(json!({
                    "key": key,
                    "value": value,
                }));
                self.try_output(ch, data)?;
            } else {
                // value is empty
                self.try_output(ch, AgentData::new_unit())?;
            }
        } else {
            // return_after is false
            self.try_output(ch, AgentData::new_unit())?;
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

    // Database Delete
    defs.insert(
        "$database_delete".into(),
        AgentDefinition::new(
            "Database",
            "$database_delete",
            Some(new_boxed::<DatabaseDeleteAgent>),
        )
        .with_title("Database Delete")
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
            (
                "return_before".into(),
                AgentConfigEntry::new(AgentValue::new_boolean(false), "boolean"),
            ),
        ]),
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

    // Database Update
    defs.insert(
        "$database_update".into(),
        AgentDefinition::new(
            "Database",
            "$database_update",
            Some(new_boxed::<DatabaseUpdateAgent>),
        )
        .with_title("Database Update")
        .with_category("Core/Database")
        .with_inputs(vec!["kv"])
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

    // Database Update Merge
    defs.insert(
        "$database_update_merge".into(),
        AgentDefinition::new(
            "Database",
            "$database_update_merge",
            Some(new_boxed::<DatabaseUpdateMergeAgent>),
        )
        .with_title("Database Update Merge")
        .with_category("Core/Database")
        .with_inputs(vec!["kv"])
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
            (
                "return_after".into(),
                AgentConfigEntry::new(AgentValue::new_boolean(false), "boolean"),
            ),
        ]),
    );

    // Database Upsert
    defs.insert(
        "$database_upsert".into(),
        AgentDefinition::new(
            "Database",
            "$database_upsert",
            Some(new_boxed::<DatabaseUpsertAgent>),
        )
        .with_title("Database Upsert")
        .with_category("Core/Database")
        .with_inputs(vec!["kv"])
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

    // Database Upsert Merge
    defs.insert(
        "$database_upsert_merge".into(),
        AgentDefinition::new(
            "Database",
            "$database_upsert_merge",
            Some(new_boxed::<DatabaseUpsertMergeAgent>),
        )
        .with_title("Database Upsert Merge")
        .with_category("Core/Database")
        .with_inputs(vec!["kv"])
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
            (
                "return_after".into(),
                AgentConfigEntry::new(AgentValue::new_boolean(false), "boolean"),
            ),
        ]),
    );
}
