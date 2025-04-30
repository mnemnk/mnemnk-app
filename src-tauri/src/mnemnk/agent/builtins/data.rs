use anyhow::{Context as _, Result};
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::definition::AGENT_KIND_BUILTIN;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentConfigEntry, AgentContext, AgentData, AgentDefinition,
    AgentDefinitions, AgentValue, AsAgent, AsAgentData,
};

// To String
struct ToStringAgent {
    data: AsAgentData,
}

impl AsAgent for ToStringAgent {
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
        let s: String = match data.kind.as_str() {
            "unit" => "()".to_string(),
            "bool" => data.value.as_bool().context("wrong bool")?.to_string(),
            "integer" => data.value.as_i64().context("wrong integer")?.to_string(),
            "float" => data.value.as_f64().context("wrong float")?.to_string(),
            "string" => data.value.as_str().context("wrong string")?.to_string(),
            "text" => data.value.as_str().context("wrong text")?.to_string(),
            _ => serde_json::to_string(&data.value).context("failed to serialize")?,
        };

        self.try_output(CH_STRING, AgentData::new_string(s))
            .context("Failed to output")?;

        Ok(())
    }
}

// To Text
struct ToTextAgent {
    data: AsAgentData,
}

impl AsAgent for ToTextAgent {
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
        let s: String = match data.kind.as_str() {
            "unit" => "()".to_string(),
            "bool" => data.value.as_bool().context("wrong bool")?.to_string(),
            "integer" => data.value.as_i64().context("wrong integer")?.to_string(),
            "float" => data.value.as_f64().context("wrong float")?.to_string(),
            "string" => data.value.as_str().context("wrong string")?.to_string(),
            "text" => data.value.as_str().context("wrong text")?.to_string(),
            _ => serde_json::to_string_pretty(&data.value).context("failed to serialize")?,
        };

        self.try_output(CH_TEXT, AgentData::new_text(s))
            .context("Failed to output")?;

        Ok(())
    }
}

// To JSON
struct ToJsonAgent {
    data: AsAgentData,
}

impl AsAgent for ToJsonAgent {
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
        let json = serde_json::to_string_pretty(&data.value).context("failed to serialize")?;
        self.try_output(CH_JSON, AgentData::new_text(json))
            .context("Failed to output")?;
        Ok(())
    }
}

// From JSON
struct FromJsonAgent {
    data: AsAgentData,
}

impl AsAgent for FromJsonAgent {
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
        let json_value: serde_json::Value =
            serde_json::from_str(data.value.as_str().context("wrong json")?)
                .context("failed to parse")?;
        let data = AgentData::from_json_value(json_value);
        self.try_output(CH_DATA, data).context("Failed to output")?;
        Ok(())
    }
}

// Get Property
struct GetPropertyAgent {
    data: AsAgentData,
}

impl AsAgent for GetPropertyAgent {
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
        let property = self
            .config()
            .context("missing config")?
            .get(CONFIG_PROPERTY)
            .context("missing property")?
            .as_str()
            .context("failed as_str")?;

        if property.is_empty() {
            return Ok(());
        }

        let props = property.split('.').collect::<Vec<_>>();

        if data.value.is_object() {
            let mut value = data.value.as_object().context("failed as_object")?;
            for prop in props {
                if let Some(v) = value.get(prop) {
                    value = v;
                } else {
                    return Ok(());
                }
            }

            self.try_output(CH_DATA, AgentData::from_json_value(value.clone()))
                .context("Failed to output")?;
        }

        // TODO: support array

        Ok(())
    }
}

static CATEGORY: &str = "Core/Data";

static CH_DATA: &str = "data";
static CH_STRING: &str = "string";
static CH_TEXT: &str = "text";
static CH_JSON: &str = "json";

static CONFIG_PROPERTY: &str = "property";

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    defs.insert(
        "$to_string".to_string(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$to_string",
            Some(new_boxed::<ToStringAgent>),
        )
        .with_title("To String")
        .with_category(CATEGORY)
        .with_inputs(vec![CH_DATA])
        .with_outputs(vec![CH_STRING]),
    );

    defs.insert(
        "$to_text".to_string(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$to_text",
            Some(new_boxed::<ToTextAgent>),
        )
        .with_title("To Text")
        .with_category(CATEGORY)
        .with_inputs(vec![CH_DATA])
        .with_outputs(vec![CH_TEXT]),
    );

    defs.insert(
        "$to_json".to_string(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$to_json",
            Some(new_boxed::<ToJsonAgent>),
        )
        .with_title("To JSON")
        .with_category(CATEGORY)
        .with_inputs(vec![CH_DATA])
        .with_outputs(vec![CH_JSON]),
    );

    defs.insert(
        "$from_json".to_string(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$from_json",
            Some(new_boxed::<FromJsonAgent>),
        )
        .with_title("From JSON")
        .with_category(CATEGORY)
        .with_inputs(vec![CH_JSON])
        .with_outputs(vec![CH_DATA]),
    );

    defs.insert(
        "$get_property".to_string(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$geet_property",
            Some(new_boxed::<GetPropertyAgent>),
        )
        .with_title("Get Property")
        .with_category(CATEGORY)
        .with_inputs(vec![CH_DATA])
        .with_outputs(vec![CH_DATA])
        .with_default_config(vec![(
            CONFIG_PROPERTY.into(),
            AgentConfigEntry::new(AgentValue::new_string(""), "string"),
        )]),
    );
}
