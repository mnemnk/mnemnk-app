use anyhow::{bail, Context as _, Result};
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::definition::AGENT_KIND_BUILTIN;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentConfigEntry, AgentContext, AgentData, AgentDefinition,
    AgentDefinitions, AgentStatus, AgentValue, AsAgent, AsAgentData,
};

// Unit Input
struct UnitInputAgent {
    data: AsAgentData,
}

impl AsAgent for UnitInputAgent {
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

    fn set_config(&mut self, _config: AgentConfig) -> Result<()> {
        // Since set_config is called even when the agent is not running,
        // we need to check the status before outputting the value.
        if *self.status() == AgentStatus::Start {
            self.try_output(CONFIG_UNIT, AgentData::new_unit())
                .context("Failed to output value")?;
        }

        Ok(())
    }
}

// Boolean Input
struct BooleanInputAgent {
    data: AsAgentData,
}

impl AsAgent for BooleanInputAgent {
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

    fn set_config(&mut self, _config: AgentConfig) -> Result<()> {
        // Since set_config is called even when the agent is not running,
        // we need to check the status before outputting the value.
        if *self.status() == AgentStatus::Start {
            let value = self
                .config()
                .context("no context")?
                .get_bool(CONFIG_BOOLEAN)
                .context("not a boolean")?;
            self.try_output(CONFIG_BOOLEAN, AgentData::new_boolean(value))
                .context("Failed to output value")?;
        }

        Ok(())
    }
}

// Integer Input
struct IntegerInputAgent {
    data: AsAgentData,
}

impl AsAgent for IntegerInputAgent {
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

    fn set_config(&mut self, _config: AgentConfig) -> Result<()> {
        // Since set_config is called even when the agent is not running,
        // we need to check the status before outputting the value.
        if *self.status() == AgentStatus::Start {
            let value = self
                .config()
                .context("no context")?
                .get_integer(CONFIG_INTEGER)
                .context("not an integer")?;
            self.try_output(CONFIG_INTEGER, AgentData::new_integer(value))
                .context("Failed to output value")?;
        }

        Ok(())
    }
}

// Number Input
struct NumberInputAgent {
    data: AsAgentData,
}

impl AsAgent for NumberInputAgent {
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

    fn set_config(&mut self, _config: AgentConfig) -> Result<()> {
        // Since set_config is called even when the agent is not running,
        // we need to check the status before outputting the value.
        if *self.status() == AgentStatus::Start {
            let value = self
                .config()
                .context("no context")?
                .get_number(CONFIG_NUMBER)
                .context("not a number")?;
            self.try_output(CONFIG_NUMBER, AgentData::new_number(value))
                .context("Failed to output value")?;
        }

        Ok(())
    }
}

// String Input
struct StringInputAgent {
    data: AsAgentData,
}

impl AsAgent for StringInputAgent {
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

    fn set_config(&mut self, _config: AgentConfig) -> Result<()> {
        // Since set_config is called even when the agent is not running,
        // we need to check the status before outputting the value.
        if *self.status() == AgentStatus::Start {
            let value = self
                .config()
                .context("no context")?
                .get_string(CONFIG_STRING)
                .context("not a string")?;
            self.try_output(CONFIG_STRING, AgentData::new_string(value))
                .context("Failed to output value")?;
        }

        Ok(())
    }
}

// Text Input
struct TextInputAgent {
    data: AsAgentData,
}

impl AsAgent for TextInputAgent {
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

    fn set_config(&mut self, _config: AgentConfig) -> Result<()> {
        // Since set_config is called even when the agent is not running,
        // we need to check the status before outputting the value.
        if *self.status() == AgentStatus::Start {
            let value = self
                .config()
                .context("no context")?
                .get_string(CONFIG_TEXT)
                .context("not a text")?;
            self.try_output(CONFIG_TEXT, AgentData::new_text(value))
                .context("Failed to output text")?;
        }

        Ok(())
    }
}

// Object Input
struct ObjectInputAgent {
    data: AsAgentData,
}

impl AsAgent for ObjectInputAgent {
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

    fn set_config(&mut self, _config: AgentConfig) -> Result<()> {
        // Since set_config is called even when the agent is not running,
        // we need to check the status before outputting the value.
        if *self.status() == AgentStatus::Start {
            let config = self.config().context("no context")?;
            let value = config.get(CONFIG_OBJECT).context("no object")?;
            if let Some(obj) = value.as_object() {
                self.try_output(CONFIG_OBJECT, AgentData::new_object(obj.clone()))
                    .context("Failed to output value")?;
            } else if let Some(arr) = value.as_array() {
                self.try_output(CONFIG_OBJECT, AgentData::new_array("object", arr.clone()))
                    .context("Failed to output value")?;
            } else {
                bail!("not an object");
            }
        }

        Ok(())
    }
}

static CATEGORY: &str = "Core/Input";

static CONFIG_UNIT: &str = "unit";
static CONFIG_BOOLEAN: &str = "boolean";
static CONFIG_INTEGER: &str = "integer";
static CONFIG_NUMBER: &str = "number";
static CONFIG_STRING: &str = "string";
static CONFIG_TEXT: &str = "text";
static CONFIG_OBJECT: &str = "object";

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    // Unit Input
    defs.insert(
        "$unit_input".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$unit_input",
            Some(new_boxed::<UnitInputAgent>),
        )
        .with_title("Unit Input")
        .with_category(CATEGORY)
        .with_outputs(vec![CONFIG_UNIT])
        .with_default_config(vec![(
            CONFIG_UNIT.into(),
            AgentConfigEntry::new(AgentValue::new_unit(), "unit"),
        )]),
    );

    // Boolean Input
    defs.insert(
        "$boolean_input".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$boolean_input",
            Some(new_boxed::<BooleanInputAgent>),
        )
        .with_title("Boolean Input")
        .with_category(CATEGORY)
        .with_outputs(vec![CONFIG_BOOLEAN])
        .with_default_config(vec![(
            CONFIG_BOOLEAN.into(),
            AgentConfigEntry::new(AgentValue::new_boolean(false), "boolean"),
        )]),
    );

    // Integer Input
    defs.insert(
        "$integer_input".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$integer_input",
            Some(new_boxed::<IntegerInputAgent>),
        )
        .with_title("Integer Input")
        .with_category(CATEGORY)
        .with_outputs(vec![CONFIG_INTEGER])
        .with_default_config(vec![(
            CONFIG_INTEGER.into(),
            AgentConfigEntry::new(AgentValue::new_integer(0), "integer"),
        )]),
    );

    // Number Input
    defs.insert(
        "$number_input".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$number_input",
            Some(new_boxed::<NumberInputAgent>),
        )
        .with_title("Number Input")
        .with_category(CATEGORY)
        .with_outputs(vec![CONFIG_NUMBER])
        .with_default_config(vec![(
            CONFIG_NUMBER.into(),
            AgentConfigEntry::new(AgentValue::new_number(0.0), "number"),
        )]),
    );

    // String Input
    defs.insert(
        "$string_input".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$string_input",
            Some(new_boxed::<StringInputAgent>),
        )
        .with_title("String Input")
        .with_category(CATEGORY)
        .with_outputs(vec![CONFIG_STRING])
        .with_default_config(vec![(
            CONFIG_STRING.into(),
            AgentConfigEntry::new(AgentValue::new_string(""), "string"),
        )]),
    );

    // Text Input
    defs.insert(
        "$text_input".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$text_input",
            Some(new_boxed::<TextInputAgent>),
        )
        .with_title("Text Input")
        .with_category(CATEGORY)
        .with_outputs(vec![CONFIG_TEXT])
        .with_default_config(vec![(
            CONFIG_TEXT.into(),
            AgentConfigEntry::new(AgentValue::new_string(""), "text"),
        )]),
    );

    // Object Input
    defs.insert(
        "$object_input".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$object_input",
            Some(new_boxed::<ObjectInputAgent>),
        )
        .with_title("Object Input")
        .with_category(CATEGORY)
        .with_outputs(vec![CONFIG_OBJECT])
        .with_default_config(vec![(
            CONFIG_OBJECT.into(),
            AgentConfigEntry::new(AgentValue::default_object(), "object"),
        )]),
    );
}
