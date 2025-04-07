use anyhow::{Context as _, Result};
use handlebars::Handlebars;
use regex::Regex;
use serde_json::{json, Value};
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentConfigEntry, AgentData, AgentDefinition, AgentDefinitions, AsAgent,
};

// Regex Filter
struct RegexFilterAgent {
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

// Template String Agent
struct TemplateStringAgent {
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

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
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
    defs.insert(
        "$template_text".into(),
        AgentDefinition::new(
            // We can use the kind as TemplateStringAgent,
            // since the only difference is the config type,
            // and we can use the same agent for both.
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
}
