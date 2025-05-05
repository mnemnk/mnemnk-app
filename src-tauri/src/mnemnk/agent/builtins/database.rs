use anyhow::{bail, Context as _, Ok, Result};
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::definition::AGENT_KIND_DATABASE;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentConfigEntry, AgentContext, AgentData, AgentDefinition,
    AgentDefinitions, AgentValue, AgentValueMap, AsAgent, AsAgentData,
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
        let config = self.config().context("Missing config")?;
        let (db, table) = get_db_table(config)?;
        let return_before = config.get_bool_or_default(CONFIG_RETURN_BEFORE);

        let key = data.as_str().context("key is not a string")?;
        if key.is_empty() {
            bail!("key is empty");
        }

        let result = store::delete(self.app(), db, table, key.to_string(), return_before)?;
        if return_before {
            if let Some(json_value) = result {
                let value = AgentValue::from_json_value(json_value)?;
                let kv_data = new_kv_data(key, value).from_meta(&data.metadata);
                self.try_output(CH_KV, kv_data)?;
            } else {
                // value is empty
                self.try_output(CH_KV, AgentData::new_unit().from_meta(&data.metadata))?;
            }
        } else {
            // return_before is false
            self.try_output(CH_KV, AgentData::new_unit().from_meta(&data.metadata))?;
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
        let config = self.config().context("Missing config")?;
        let (db, table) = get_db_table(config)?;
        let (key, value) = get_kv(&data)?;
        let json_value = value.to_json_value();
        store::insert(self.app(), db, table, key, json_value)?;

        self.try_output(ch, data)
    }
}

// Database Query
struct DatabaseQueryAgent {
    data: AsAgentData,
}

impl AsAgent for DatabaseQueryAgent {
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
        let config = self.config().context("Missing config")?;
        let db = get_db(config)?;

        let query;
        let mut bindings: Option<serde_json::Value> = None;
        if data.is_string() || data.is_text() {
            query = data.as_str().context("Failed as_str")?.to_string();
            if query.is_empty() {
                return Ok(());
            }
        } else if data.is_object() {
            let obj = data.as_object().context("Failed as_object")?;
            let Some(q) = obj.get("query") else {
                bail!("query not found");
            };
            query = q.as_str().context("query is not a string")?.to_string();
            if query.is_empty() {
                return Ok(());
            }
            if let Some(b) = obj.get("bindings") {
                if b.is_object() {
                    bindings = Some(b.to_json_value());
                } else {
                    bail!("bindings is not an object");
                }
            };
        } else {
            // TODO: add support for array
            bail!("data is not a string or object");
        }

        let result = store::query(self.app(), db, query, bindings)?;
        let mut arr = Vec::with_capacity(result.len());
        for r in result.into_iter() {
            let value = AgentValue::from_json_value(r)?;
            arr.push(value);
        }
        let out_data = AgentData::new_array("object", arr).from_meta(&data.metadata);
        self.try_output(CH_DATA, out_data)?;
        Ok(())
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
        let config = self.config().context("Missing config")?;
        let (db, table) = get_db_table(config)?;

        let key = data.as_str().context("key is not a string")?;
        if key.is_empty() {
            bail!("key is empty");
        }

        let result = store::select(self.app(), db, table, key.to_string())?;
        if let Some(json_value) = result {
            let value = AgentValue::from_json_value(json_value)?;
            let kv_data = new_kv_data(key, value);
            self.try_output(CH_KV, kv_data.from_meta(&data.metadata))?;
        } else {
            self.try_output(CH_KV, AgentData::new_unit().from_meta(&data.metadata))?;
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
        let config = self.config().context("Missing config")?;
        let (db, table) = get_db_table(config)?;
        let (key, value) = get_kv(&data)?;
        let json_value = value.to_json_value();
        store::update(self.app(), db, table, key, json_value)?;

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
        let config = self.config().context("Missing config")?;
        let (db, table) = get_db_table(config)?;
        let return_after = config.get_bool_or_default(CONFIG_RETURN_AFTER);
        let (key, value) = get_kv(&data)?;
        let json_value = value.to_json_value();

        if return_after {
            let result =
                store::update_merge(self.app(), db, table, key.clone(), json_value, return_after)?;
            if let Some(json_value) = result {
                let value = AgentValue::from_json_value(json_value)?;
                let kv_data = new_kv_data(key, value);
                self.try_output(ch, kv_data.from_meta(&data.metadata))?;
            } else {
                self.try_output(ch, AgentData::new_unit().from_meta(&data.metadata))?;
            }
        } else {
            // return_after is false
            store::update_merge(self.app(), db, table, key, json_value, return_after)?;
            self.try_output(ch, AgentData::new_unit().from_meta(&data.metadata))?;
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
        let config = self.config().context("Missing config")?;
        let (db, table) = get_db_table(config)?;
        let (key, value) = get_kv(&data)?;
        let json_value = value.to_json_value();
        store::upsert(self.app(), db, table, key, json_value)?;

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
        let config = self.config().context("Missing config")?;
        let (db, table) = get_db_table(config)?;
        let return_after = config.get_bool_or_default(CONFIG_RETURN_AFTER);
        let (key, value) = get_kv(&data)?;
        let json_value = value.to_json_value();

        if return_after {
            let result =
                store::upsert_merge(self.app(), db, table, key.clone(), json_value, return_after)?;
            if let Some(json_value) = result {
                let value = AgentValue::from_json_value(json_value)?;
                let kv_data = new_kv_data(key, value);
                self.try_output(ch, kv_data.from_meta(&data.metadata))?;
            } else {
                self.try_output(ch, AgentData::new_unit().from_meta(&data.metadata))?;
            }
        } else {
            // return_after is false
            store::upsert_merge(self.app(), db, table, key, json_value, return_after)?;
            self.try_output(ch, AgentData::new_unit().from_meta(&data.metadata))?;
        }

        Ok(())
    }
}

fn get_db(config: &AgentConfig) -> Result<String> {
    let db = config.get_string_or_default(CONFIG_DB);
    if db.is_empty() {
        bail!("db is not set");
    }
    Ok(db)
}

fn get_db_table(config: &AgentConfig) -> Result<(String, String)> {
    let db = get_db(config)?;

    let table = config.get_string_or_default(CONFIG_TABLE);
    if table.is_empty() {
        bail!("table is not set");
    };

    Ok((db, table))
}

fn get_kv(data: &AgentData) -> Result<(String, AgentValue)> {
    let obj = data.as_object().context("data is not an object")?;
    let key = if let Some(key) = obj.get("key").cloned() {
        key.as_str().context("key is not a string")?.to_string()
    } else {
        bail!("key not found");
    };
    if key.is_empty() {
        bail!("key is empty");
    }

    let Some(value) = obj.get("value").cloned() else {
        bail!("value not found");
    };

    Ok((key, value))
}

fn new_kv_data(key: impl Into<String>, value: AgentValue) -> AgentData {
    AgentData::new_object(AgentValueMap::from([
        ("key".to_string(), AgentValue::new_string(key.into())),
        ("value".to_string(), value),
    ]))
}

static CH_KEY: &str = "key";
static CH_KV: &str = "kv";
static CH_QUERY: &str = "query";
static CH_DATA: &str = "data";

static CONFIG_DB: &str = "db";
static CONFIG_TABLE: &str = "table";
static CONFIG_RETURN_AFTER: &str = "return_after";
static CONFIG_RETURN_BEFORE: &str = "return_before";

static CATEGORY: &str = "Core/Database";

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    // Event Database
    defs.insert(
        "$event_database".into(),
        AgentDefinition::new(
            AGENT_KIND_DATABASE,
            "$event_database",
            Some(new_boxed::<EventDatabaseAgent>),
        )
        .with_title("Event Database")
        .with_category(CATEGORY)
        .with_inputs(vec!["event"]),
    );

    // Database Delete
    defs.insert(
        "$database_delete".into(),
        AgentDefinition::new(
            AGENT_KIND_DATABASE,
            "$database_delete",
            Some(new_boxed::<DatabaseDeleteAgent>),
        )
        .with_title("Database Delete")
        .with_category(CATEGORY)
        .with_inputs(vec![CH_KEY])
        .with_outputs(vec![CH_KV])
        .with_default_config(vec![
            (
                CONFIG_DB.into(),
                AgentConfigEntry::new(AgentValue::new_string(""), "string"),
            ),
            (
                CONFIG_TABLE.into(),
                AgentConfigEntry::new(AgentValue::new_string(""), "string"),
            ),
            (
                CONFIG_RETURN_BEFORE.into(),
                AgentConfigEntry::new(AgentValue::new_boolean(false), "boolean"),
            ),
        ]),
    );

    // Database Insert
    defs.insert(
        "$database_insert".into(),
        AgentDefinition::new(
            AGENT_KIND_DATABASE,
            "$database_insert",
            Some(new_boxed::<DatabaseInsertAgent>),
        )
        .with_title("Database Insert")
        .with_category(CATEGORY)
        .with_inputs(vec![CH_KV])
        .with_outputs(vec![CH_KV])
        .with_default_config(vec![
            (
                CONFIG_DB.into(),
                AgentConfigEntry::new(AgentValue::new_string(""), "string"),
            ),
            (
                CONFIG_TABLE.into(),
                AgentConfigEntry::new(AgentValue::new_string(""), "string"),
            ),
        ]),
    );

    // Database Select
    defs.insert(
        "$database_select".into(),
        AgentDefinition::new(
            AGENT_KIND_DATABASE,
            "$database_select",
            Some(new_boxed::<DatabaseSelectAgent>),
        )
        .with_title("Database Select")
        .with_category(CATEGORY)
        .with_inputs(vec![CH_KEY])
        .with_outputs(vec![CH_KV])
        .with_default_config(vec![
            (
                CONFIG_DB.into(),
                AgentConfigEntry::new(AgentValue::new_string(""), "string"),
            ),
            (
                CONFIG_TABLE.into(),
                AgentConfigEntry::new(AgentValue::new_string(""), "string"),
            ),
        ]),
    );

    // Database Update
    defs.insert(
        "$database_update".into(),
        AgentDefinition::new(
            AGENT_KIND_DATABASE,
            "$database_update",
            Some(new_boxed::<DatabaseUpdateAgent>),
        )
        .with_title("Database Update")
        .with_category(CATEGORY)
        .with_inputs(vec![CH_KV])
        .with_outputs(vec![CH_KV])
        .with_default_config(vec![
            (
                CONFIG_DB.into(),
                AgentConfigEntry::new(AgentValue::new_string(""), "string"),
            ),
            (
                CONFIG_TABLE.into(),
                AgentConfigEntry::new(AgentValue::new_string(""), "string"),
            ),
        ]),
    );

    // Database Update Merge
    defs.insert(
        "$database_update_merge".into(),
        AgentDefinition::new(
            AGENT_KIND_DATABASE,
            "$database_update_merge",
            Some(new_boxed::<DatabaseUpdateMergeAgent>),
        )
        .with_title("Database Update Merge")
        .with_category(CATEGORY)
        .with_inputs(vec![CH_KV])
        .with_outputs(vec![CH_KV])
        .with_default_config(vec![
            (
                CONFIG_DB.into(),
                AgentConfigEntry::new(AgentValue::new_string(""), "string"),
            ),
            (
                CONFIG_TABLE.into(),
                AgentConfigEntry::new(AgentValue::new_string(""), "string"),
            ),
            (
                CONFIG_RETURN_AFTER.into(),
                AgentConfigEntry::new(AgentValue::new_boolean(false), "boolean"),
            ),
        ]),
    );

    // Database Upsert
    defs.insert(
        "$database_upsert".into(),
        AgentDefinition::new(
            AGENT_KIND_DATABASE,
            "$database_upsert",
            Some(new_boxed::<DatabaseUpsertAgent>),
        )
        .with_title("Database Upsert")
        .with_category(CATEGORY)
        .with_inputs(vec![CH_KV])
        .with_outputs(vec![CH_KV])
        .with_default_config(vec![
            (
                CONFIG_DB.into(),
                AgentConfigEntry::new(AgentValue::new_string(""), "string"),
            ),
            (
                CONFIG_TABLE.into(),
                AgentConfigEntry::new(AgentValue::new_string(""), "string"),
            ),
        ]),
    );

    // Database Upsert Merge
    defs.insert(
        "$database_upsert_merge".into(),
        AgentDefinition::new(
            AGENT_KIND_DATABASE,
            "$database_upsert_merge",
            Some(new_boxed::<DatabaseUpsertMergeAgent>),
        )
        .with_title("Database Upsert Merge")
        .with_category(CATEGORY)
        .with_inputs(vec![CH_KV])
        .with_outputs(vec![CH_KV])
        .with_default_config(vec![
            (
                CONFIG_DB.into(),
                AgentConfigEntry::new(AgentValue::new_string(""), "string"),
            ),
            (
                CONFIG_TABLE.into(),
                AgentConfigEntry::new(AgentValue::new_string(""), "string"),
            ),
            (
                CONFIG_RETURN_AFTER.into(),
                AgentConfigEntry::new(AgentValue::new_boolean(false), "boolean"),
            ),
        ]),
    );

    // Database Query
    defs.insert(
        "$database_query".into(),
        AgentDefinition::new(
            AGENT_KIND_DATABASE,
            "$database_query",
            Some(new_boxed::<DatabaseQueryAgent>),
        )
        .with_title("Database Query")
        .with_category(CATEGORY)
        .with_inputs(vec![CH_QUERY])
        .with_outputs(vec![CH_DATA])
        .with_default_config(vec![(
            CONFIG_DB.into(),
            AgentConfigEntry::new(AgentValue::new_string(""), "string"),
        )]),
    );
}
