use anyhow::{Context as _, Result};
use handlebars::Handlebars;
use regex::Regex;
use serde::Serialize;
use serde_json::{json, Value};
use tauri::AppHandle;

use super::agent::{new_boxed, Agent, AgentConfig, AgentData, AgentStatus, AsAgent};
use super::definition::{
    AgentConfigEntry, AgentDefinition, AgentDefinitions, AgentDisplayConfigEntry,
};
use super::message;

// Database

pub struct DatabaseAgent {
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
        message::try_send_store(self.app(), kind, value)
    }
}

// Display Value

pub struct DisplayValueAgent {
    data: AgentData,
}

impl AsAgent for DisplayValueAgent {
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
        let display_value = DisplayValue { kind, value };
        self.emit_display("value".to_string(), json!(display_value))
    }
}

#[derive(Debug, Clone, Serialize)]
struct DisplayValue {
    kind: String,
    value: Value,
}

// Display Messages
pub struct DisplayMessagesAgent {
    data: AgentData,
}

impl AsAgent for DisplayMessagesAgent {
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

    fn input(&mut self, _kind: String, value: Value) -> Result<()> {
        let display_value = DisplayValue {
            kind: "messages".to_string(),
            value,
        };
        self.emit_display("messages".to_string(), json!(display_value))
    }
}

// Regex Filter

pub struct RegexFilterAgent {
    data: AgentData,
}

impl AsAgent for RegexFilterAgent {
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
        let config = self.data.config.as_ref().context("Missing config")?;

        let field = config
            .get("field")
            .context("Missing field")?
            .as_str()
            .context("field is not a string")?;
        if field.is_empty() {
            // field is not set
            return Ok(());
        }

        let regex = config
            .get("regex")
            .context("Missing regex")?
            .as_str()
            .context("regex is not a string")?;
        if regex.is_empty() {
            // regex is not set
            return Ok(());
        }
        let regex = Regex::new(regex).context("Failed to compile regex")?;

        let Some(field_value) = value.get(field) else {
            // value does not have the field
            return Ok(());
        };
        let field_value = field_value
            .as_str()
            .context("value is not a string")?
            .to_string();
        if regex.is_match(&field_value) {
            // value matches the regex
            self.try_output(kind.clone(), value.into())
                .context("Failed to output regex result")?;
        }

        Ok(())
    }
}

// Boolean Input
pub struct BooleanInputAgent {
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
pub struct IntegerInputAgent {
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
pub struct NumberInputAgent {
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
pub struct StringInputAgent {
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
pub struct TextInputAgent {
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
pub struct ObjectInputAgent {
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

// As Kind Agent
pub struct AsKindAgent {
    data: AgentData,
}

impl AsAgent for AsKindAgent {
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

    fn input(&mut self, _kind: String, value: Value) -> Result<()> {
        let kind = self
            .data
            .config
            .as_ref()
            .context("Missing config")?
            .get("kind")
            .context("Missing kind")?
            .as_str()
            .context("kind is not a string")?;
        if kind.is_empty() {
            // kind is not set
            return Ok(());
        }
        self.try_output(kind.to_string(), value.clone())
            .context("Failed to output")?;
        Ok(())
    }
}

// Template String Agent
pub struct TemplateStringAgent {
    data: AgentData,
}

impl AsAgent for TemplateStringAgent {
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

    fn input(&mut self, _kind: String, value: Value) -> Result<()> {
        if !value.is_object() {
            // value is not an object
            return Ok(());
        }

        let config = self.data.config.as_ref().context("Missing config")?;
        let template = config
            .get("template")
            .context("Missing template")?
            .as_str()
            .context("template is not a string")?;
        if template.is_empty() {
            // template is not set
            return Ok(());
        }

        let reg = Handlebars::new();
        let out_value = reg.render_template(template, &value)?;

        self.try_output("string".to_string(), json!(out_value))
            .context("Failed to output template")?;
        Ok(())
    }
}

// Agent Definitions

pub fn builtin_agent_defs() -> AgentDefinitions {
    let mut defs: AgentDefinitions = Default::default();

    // BoardInAgent
    defs.insert(
        "$board_in".into(),
        AgentDefinition::new(
            "BoardIn",
            "$board_in",
            Some(new_boxed::<super::board::BoardInAgent>),
        )
        .with_title("Board In")
        .with_category("Core")
        .with_inputs(vec!["*"])
        .with_default_config(vec![(
            "board_name".into(),
            AgentConfigEntry::new(json!(""), "string")
                .with_title("Board Name")
                .with_description("* = source kind"),
        )]),
    );

    // BoardOutAgent
    defs.insert(
        "$board_out".into(),
        AgentDefinition::new(
            "BoardOut",
            "$board_out",
            Some(new_boxed::<super::board::BoardOutAgent>),
        )
        .with_title("Board Out")
        .with_category("Core")
        .with_outputs(vec!["*"])
        .with_default_config(vec![(
            "board_name".into(),
            AgentConfigEntry::new(json!(""), "string").with_title("Board Name"),
        )]),
    );

    // DatabaseAgent
    defs.insert(
        "$database".into(),
        AgentDefinition::new("Database", "$database", Some(new_boxed::<DatabaseAgent>))
            .with_title("Database")
            .with_description("Store data")
            .with_category("Core")
            .with_inputs(vec!["*"]),
    );

    // Display Value
    defs.insert(
        "$display_value".into(),
        AgentDefinition::new(
            "DisplayValue",
            "$display_value",
            Some(new_boxed::<DisplayValueAgent>),
        )
        .with_title("Display Value")
        // .with_description("Display value on the node")
        .with_category("Display")
        .with_inputs(vec!["*"])
        .with_display_config(vec![(
            "value".into(),
            AgentDisplayConfigEntry::new("object"),
        )]),
    );

    // Display Messages
    defs.insert(
        "$display_messages".into(),
        AgentDefinition::new(
            "DisplayMessages",
            "$display_messages",
            Some(new_boxed::<DisplayMessagesAgent>),
        )
        .with_title("Display Messages")
        .with_description("Display messages on the node")
        .with_category("LLM")
        .with_inputs(vec!["*"])
        .with_display_config(vec![(
            "messages".into(),
            AgentDisplayConfigEntry::new("messages"),
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

    // AsKindAgent
    defs.insert(
        "$as_kind_filter".into(),
        AgentDefinition::new(
            "AsKindAgent",
            "$as_kind_agent",
            Some(new_boxed::<AsKindAgent>),
        )
        .with_title("As Kind")
        .with_category("Core")
        .with_inputs(vec!["*"])
        .with_outputs(vec!["*"])
        .with_default_config(vec![(
            "kind".into(),
            AgentConfigEntry::new(json!(""), "string").with_title("Kind"),
        )]),
    );

    // RegexFilterAgent
    defs.insert(
        "$regex_filter".into(),
        AgentDefinition::new(
            "RegexFilter",
            "$regex_filter",
            Some(new_boxed::<RegexFilterAgent>),
        )
        .with_title("Regex Filter")
        .with_category("Core/String")
        .with_inputs(vec!["*"])
        .with_outputs(vec!["*"])
        .with_default_config(vec![
            (
                "field".into(),
                AgentConfigEntry::new(json!(""), "string").with_title("Field"),
            ),
            (
                "regex".into(),
                AgentConfigEntry::new(json!(""), "string").with_title("Regex"),
            ),
        ]),
    );

    // Template String Agent
    defs.insert(
        "$template_string".into(),
        AgentDefinition::new(
            "TemplateString",
            "$template_string",
            Some(new_boxed::<TemplateStringAgent>),
        )
        .with_title("Template String")
        .with_category("Core/String")
        .with_inputs(vec!["*"])
        .with_outputs(vec!["string"])
        .with_default_config(vec![(
            "template".into(),
            AgentConfigEntry::new(json!(""), "string"),
        )]),
    );

    // Template Text Agent
    // We can use the same agent as TemplateStringAgent,
    // since the only difference is the config type.
    defs.insert(
        "$template_text".into(),
        AgentDefinition::new(
            "TemplateString",
            "$template_text",
            Some(new_boxed::<TemplateStringAgent>),
        )
        .with_title("Template Text")
        .with_category("Core/String")
        .with_inputs(vec!["*"])
        .with_outputs(vec!["string"])
        .with_default_config(vec![(
            "template".into(),
            AgentConfigEntry::new(json!(""), "text"),
        )]),
    );

    defs
}
