use anyhow::{Context as _, Result};
use serde_json::Value;
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
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

    fn set_config(&mut self, _config: AgentConfig) -> Result<()> {
        // Since set_config is called even when the agent is not running,
        // we need to check the status before outputting the value.
        if *self.status() == AgentStatus::Start {
            self.try_output("unit".to_string(), AgentData::new_unit())
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

    fn set_config(&mut self, _config: AgentConfig) -> Result<()> {
        // Since set_config is called even when the agent is not running,
        // we need to check the status before outputting the value.
        if *self.status() == AgentStatus::Start {
            let value = self
                .data
                .config
                .as_ref()
                .context("Missing config")?
                .get("boolean")
                .context("Missing boolean")?
                .as_bool()
                .context("boolean in config is not a boolean")?;
            self.try_output("boolean".to_string(), AgentData::new_boolean(value))
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

    fn set_config(&mut self, _config: AgentConfig) -> Result<()> {
        // Since set_config is called even when the agent is not running,
        // we need to check the status before outputting the value.
        if *self.status() == AgentStatus::Start {
            let value = self
                .data
                .config
                .as_ref()
                .context("Missing config")?
                .get("integer")
                .context("Missing integer")?;
            self.try_output(
                "integer".to_string(),
                AgentData {
                    kind: "integer".to_string(),
                    value: value.clone(),
                },
            )
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

    fn set_config(&mut self, _config: AgentConfig) -> Result<()> {
        // Since set_config is called even when the agent is not running,
        // we need to check the status before outputting the value.
        if *self.status() == AgentStatus::Start {
            let value = self
                .data
                .config
                .as_ref()
                .context("Missing config")?
                .get("number")
                .context("Missing number")?;
            self.try_output(
                "number".to_string(),
                AgentData {
                    kind: "number".to_string(),
                    value: value.clone(),
                },
            )
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

    fn set_config(&mut self, _config: AgentConfig) -> Result<()> {
        // Since set_config is called even when the agent is not running,
        // we need to check the status before outputting the value.
        if *self.status() == AgentStatus::Start {
            let value = self
                .data
                .config
                .as_ref()
                .context("Missing config")?
                .get("string")
                .context("Missing string")?
                .clone();
            self.try_output(
                "string".to_string(),
                AgentData {
                    kind: "string".to_string(),
                    value,
                },
            )
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

    fn set_config(&mut self, _config: AgentConfig) -> Result<()> {
        // Since set_config is called even when the agent is not running,
        // we need to check the status before outputting the value.
        if *self.status() == AgentStatus::Start {
            let value = self
                .data
                .config
                .as_ref()
                .context("Missing config")?
                .get("text")
                .context("Missing text")?
                .clone();
            self.try_output(
                "text".to_string(),
                AgentData {
                    kind: "text".to_string(),
                    value,
                },
            )
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

    fn set_config(&mut self, _config: AgentConfig) -> Result<()> {
        // Since set_config is called even when the agent is not running,
        // we need to check the status before outputting the value.
        if *self.status() == AgentStatus::Start {
            let value = self
                .data
                .config
                .as_ref()
                .context("Missing config")?
                .get("object")
                .context("Missing object")?
                .clone();
            self.try_output(
                "object".to_string(),
                AgentData {
                    kind: "object".to_string(),
                    value,
                },
            )
            .context("Failed to output value")?;
        }

        Ok(())
    }
}

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    // Unit Input
    defs.insert(
        "$unit_input".into(),
        AgentDefinition::new("Builtin", "$unit_input", Some(new_boxed::<UnitInputAgent>))
            .with_title("Unit Input")
            .with_category("Core/Input")
            .with_outputs(vec!["unit"])
            .with_default_config(vec![(
                "unit".into(),
                AgentConfigEntry::new(AgentValue::new_unit(), "unit"),
            )]),
    );

    // Boolean Input
    defs.insert(
        "$boolean_input".into(),
        AgentDefinition::new(
            "Builtin",
            "$boolean_input",
            Some(new_boxed::<BooleanInputAgent>),
        )
        .with_title("Boolean Input")
        .with_category("Core/Input")
        .with_outputs(vec!["boolean"])
        .with_default_config(vec![(
            "boolean".into(),
            AgentConfigEntry::new(AgentValue::new_boolean(false), "boolean"),
        )]),
    );

    // Integer Input
    defs.insert(
        "$integer_input".into(),
        AgentDefinition::new(
            "Builtin",
            "$integer_input",
            Some(new_boxed::<IntegerInputAgent>),
        )
        .with_title("Integer Input")
        .with_category("Core/Input")
        .with_outputs(vec!["integer"])
        .with_default_config(vec![(
            "integer".into(),
            AgentConfigEntry::new(AgentValue::new_integer(0), "integer"),
        )]),
    );

    // Number Input
    defs.insert(
        "$number_input".into(),
        AgentDefinition::new(
            "Builtin",
            "$number_input",
            Some(new_boxed::<NumberInputAgent>),
        )
        .with_title("Number Input")
        .with_category("Core/Input")
        .with_outputs(vec!["number"])
        .with_default_config(vec![(
            "number".into(),
            AgentConfigEntry::new(AgentValue::new_number(0.0), "number"),
        )]),
    );

    // String Input
    defs.insert(
        "$string_input".into(),
        AgentDefinition::new(
            "Builtin",
            "$string_input",
            Some(new_boxed::<StringInputAgent>),
        )
        .with_title("String Input")
        .with_category("Core/Input")
        .with_outputs(vec!["string"])
        .with_default_config(vec![(
            "string".into(),
            AgentConfigEntry::new(AgentValue::new_string(""), "string"),
        )]),
    );

    // Text Input
    defs.insert(
        "$text_input".into(),
        AgentDefinition::new("Builtin", "$text_input", Some(new_boxed::<TextInputAgent>))
            .with_title("Text Input")
            .with_category("Core/Input")
            .with_outputs(vec!["text"])
            .with_default_config(vec![(
                "text".into(),
                AgentConfigEntry::new(AgentValue::new_text(""), "text"),
            )]),
    );

    // Object Input
    defs.insert(
        "$object_input".into(),
        AgentDefinition::new(
            "Builtin",
            "$object_input",
            Some(new_boxed::<ObjectInputAgent>),
        )
        .with_title("Object Input")
        .with_category("Core/Input")
        .with_outputs(vec!["object"])
        .with_default_config(vec![(
            "object".into(),
            AgentConfigEntry::new(AgentValue::new_object(Value::Null), "object"),
        )]),
    );
}
