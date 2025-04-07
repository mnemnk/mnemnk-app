use anyhow::{Context as _, Result};
use serde_json::{json, Value};
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentConfigEntry, AgentData, AgentDefinition, AgentDefinitions,
    AgentStatus, AsAgent,
};

// Unit Input
struct UnitInputAgent {
    data: AgentData,
}

impl AsAgent for UnitInputAgent {
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

    fn set_config(&mut self, config: AgentConfig) -> Result<()> {
        self.mut_data().config = Some(config);

        // Since set_config is called even when the agent is not running,
        // we need to check the status before outputting the value.
        if *self.status() == AgentStatus::Run {
            self.try_output("unit".to_string(), json!(()))
                .context("Failed to output value")?;
        }

        Ok(())
    }

    fn input(&mut self, _kind: String, _value: Value) -> Result<()> {
        Ok(())
    }
}

// Boolean Input
struct BooleanInputAgent {
    data: AgentData,
}

impl AsAgent for BooleanInputAgent {
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

    fn set_config(&mut self, config: AgentConfig) -> Result<()> {
        self.mut_data().config = Some(config);

        // Since set_config is called even when the agent is not running,
        // we need to check the status before outputting the value.
        if *self.status() == AgentStatus::Run {
            let value = self
                .data
                .config
                .as_ref()
                .context("Missing config")?
                .get("boolean")
                .context("Missing boolean")?;
            self.try_output("boolean".to_string(), value.clone())
                .context("Failed to output value")?;
        }

        Ok(())
    }

    fn input(&mut self, _kind: String, _value: Value) -> Result<()> {
        Ok(())
    }
}

// Integer Input
struct IntegerInputAgent {
    data: AgentData,
}

impl AsAgent for IntegerInputAgent {
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

    fn set_config(&mut self, config: AgentConfig) -> Result<()> {
        self.mut_data().config = Some(config);

        // Since set_config is called even when the agent is not running,
        // we need to check the status before outputting the value.
        if *self.status() == AgentStatus::Run {
            let value = self
                .data
                .config
                .as_ref()
                .context("Missing config")?
                .get("integer")
                .context("Missing integer")?;
            self.try_output("integer".to_string(), value.clone())
                .context("Failed to output value")?;
        }

        Ok(())
    }

    fn input(&mut self, _kind: String, _value: Value) -> Result<()> {
        Ok(())
    }
}

// Number Input
struct NumberInputAgent {
    data: AgentData,
}

impl AsAgent for NumberInputAgent {
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

    fn set_config(&mut self, config: AgentConfig) -> Result<()> {
        self.mut_data().config = Some(config);

        // Since set_config is called even when the agent is not running,
        // we need to check the status before outputting the value.
        if *self.status() == AgentStatus::Run {
            let value = self
                .data
                .config
                .as_ref()
                .context("Missing config")?
                .get("number")
                .context("Missing number")?;
            self.try_output("number".to_string(), value.clone())
                .context("Failed to output value")?;
        }

        Ok(())
    }

    fn input(&mut self, _kind: String, _value: Value) -> Result<()> {
        Ok(())
    }
}

// String Input
struct StringInputAgent {
    data: AgentData,
}

impl AsAgent for StringInputAgent {
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

    fn set_config(&mut self, config: AgentConfig) -> Result<()> {
        self.mut_data().config = Some(config);

        // Since set_config is called even when the agent is not running,
        // we need to check the status before outputting the value.
        if *self.status() == AgentStatus::Run {
            let value = self
                .data
                .config
                .as_ref()
                .context("Missing config")?
                .get("string")
                .context("Missing string")?;
            self.try_output("string".to_string(), value.clone())
                .context("Failed to output value")?;
        }

        Ok(())
    }

    fn input(&mut self, _kind: String, _value: Value) -> Result<()> {
        Ok(())
    }
}

// Text Input
struct TextInputAgent {
    data: AgentData,
}

impl AsAgent for TextInputAgent {
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

    fn set_config(&mut self, config: AgentConfig) -> Result<()> {
        self.mut_data().config = Some(config);

        // Since set_config is called even when the agent is not running,
        // we need to check the status before outputting the value.
        if *self.status() == AgentStatus::Run {
            let value = self
                .data
                .config
                .as_ref()
                .context("Missing config")?
                .get("text")
                .context("Missing text")?;
            self.try_output("text".to_string(), value.clone())
                .context("Failed to output text")?;
        }

        Ok(())
    }

    fn input(&mut self, _kind: String, _value: Value) -> Result<()> {
        Ok(())
    }
}

// Object Input
struct ObjectInputAgent {
    data: AgentData,
}

impl AsAgent for ObjectInputAgent {
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

    fn set_config(&mut self, config: AgentConfig) -> Result<()> {
        self.mut_data().config = Some(config);

        // Since set_config is called even when the agent is not running,
        // we need to check the status before outputting the value.
        if *self.status() == AgentStatus::Run {
            let value = self
                .data
                .config
                .as_ref()
                .context("Missing config")?
                .get("object")
                .context("Missing object")?;
            self.try_output("object".to_string(), value.clone())
                .context("Failed to output value")?;
        }

        Ok(())
    }

    fn input(&mut self, _kind: String, _value: Value) -> Result<()> {
        Ok(())
    }
}

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    // Unit Input
    defs.insert(
        "$unit_input".into(),
        AgentDefinition::new(
            "UnitInput",
            "$unit_input",
            Some(new_boxed::<UnitInputAgent>),
        )
        .with_title("Unit Input")
        .with_category("Input")
        .with_outputs(vec!["unit"])
        .with_default_config(vec![(
            "unit".into(),
            AgentConfigEntry::new(json!(()), "unit"),
        )]),
    );

    // Boolean Input
    defs.insert(
        "$boolean_input".into(),
        AgentDefinition::new(
            "BooleanInput",
            "$boolean_input",
            Some(new_boxed::<BooleanInputAgent>),
        )
        .with_title("Boolean Input")
        .with_category("Input")
        .with_outputs(vec!["boolean"])
        .with_default_config(vec![(
            "boolean".into(),
            AgentConfigEntry::new(json!(false), "boolean"),
        )]),
    );

    // Integer Input
    defs.insert(
        "$integer_input".into(),
        AgentDefinition::new(
            "IntegerInput",
            "$integer_input",
            Some(new_boxed::<IntegerInputAgent>),
        )
        .with_title("Integer Input")
        .with_category("Input")
        .with_outputs(vec!["integer"])
        .with_default_config(vec![(
            "integer".into(),
            AgentConfigEntry::new(json!(0), "integer"),
        )]),
    );

    // Number Input
    defs.insert(
        "$number_input".into(),
        AgentDefinition::new(
            "NumberInput",
            "$number_input",
            Some(new_boxed::<NumberInputAgent>),
        )
        .with_title("Number Input")
        .with_category("Input")
        .with_outputs(vec!["number"])
        .with_default_config(vec![(
            "number".into(),
            AgentConfigEntry::new(json!(0.0), "number"),
        )]),
    );

    // String Input
    defs.insert(
        "$string_input".into(),
        AgentDefinition::new(
            "StringInput",
            "$string_input",
            Some(new_boxed::<StringInputAgent>),
        )
        .with_title("String Input")
        .with_category("Input")
        .with_outputs(vec!["string"])
        .with_default_config(vec![(
            "string".into(),
            AgentConfigEntry::new(json!(""), "string"),
        )]),
    );

    // Text Input
    defs.insert(
        "$text_input".into(),
        AgentDefinition::new(
            "TextInput",
            "$text_input",
            Some(new_boxed::<TextInputAgent>),
        )
        .with_title("Text Input")
        .with_category("Input")
        .with_outputs(vec!["text"])
        .with_default_config(vec![(
            "text".into(),
            AgentConfigEntry::new(json!(vec![""]), "text"),
        )]),
    );

    // Object Input
    defs.insert(
        "$object_input".into(),
        AgentDefinition::new(
            "ObjectInput",
            "$object_input",
            Some(new_boxed::<ObjectInputAgent>),
        )
        .with_title("Object Input")
        .with_category("Input")
        .with_outputs(vec!["object"])
        .with_default_config(vec![(
            "object".into(),
            AgentConfigEntry::new(Value::Null, "object"),
        )]),
    );
}
