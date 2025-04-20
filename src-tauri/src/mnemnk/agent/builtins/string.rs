use anyhow::{bail, Context as _, Result};
use handlebars::Handlebars;
use regex::Regex;
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentConfigEntry, AgentData, AgentDefinition, AgentDefinitions, AgentValue,
    AsAgent, AsAgentData,
};

// Regex Filter
struct RegexFilterAgent {
    data: AsAgentData,
}

impl AsAgent for RegexFilterAgent {
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

    fn process(&mut self, ch: String, data: AgentData) -> Result<()> {
        let config = self.data.config.as_ref().context("Missing config")?;

        let key = config
            .get("key")
            .context("Missing key")?
            .as_str()
            .context("key is not a string")?;
        if key.is_empty() {
            // key is not set
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

        if let AgentValue::Object(value) = &data.value {
            let Some(key_value) = value.get(key) else {
                // value does not have the key
                return Ok(());
            };
            let key_value = key_value
                .as_str()
                .context("value is not a string")?
                .to_string();
            if regex.is_match(&key_value) {
                // value matches the regex
                self.try_output(ch, data)
                    .context("Failed to output regex result")?;
            }
        }

        Ok(())
    }
}

// Template String Agent
struct TemplateStringAgent {
    data: AsAgentData,
}

impl AsAgent for TemplateStringAgent {
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

    fn process(&mut self, _ch: String, data: AgentData) -> Result<()> {
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
        let rendered_string = reg.render_template(template, &data)?;

        let (kind, out_value) = match self.def_name() {
            "$template_string" => ("string", AgentValue::new_string(rendered_string)),
            "$template_text" => ("text", AgentValue::new_text(rendered_string)),
            _ => bail!("Invalid def_name"),
        };

        self.try_output(
            kind.to_string(),
            AgentData {
                kind: kind.to_string(),
                value: out_value,
            },
        )
        .context("Failed to output template")
    }
}

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    // RegexFilterAgent
    defs.insert(
        "$regex_filter".into(),
        AgentDefinition::new(
            "Builtin",
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
                AgentConfigEntry::new(AgentValue::new_string("".to_string()), "string")
                    .with_title("Field"),
            ),
            (
                "regex".into(),
                AgentConfigEntry::new(AgentValue::new_string("".to_string()), "string")
                    .with_title("Regex"),
            ),
        ]),
    );

    // Template String Agent
    defs.insert(
        "$template_string".into(),
        AgentDefinition::new(
            "Builtin",
            "$template_string",
            Some(new_boxed::<TemplateStringAgent>),
        )
        .with_title("Template String")
        .with_category("Core/String")
        .with_inputs(vec!["*"])
        .with_outputs(vec!["string"])
        .with_default_config(vec![(
            "template".into(),
            AgentConfigEntry::new(AgentValue::new_string("".to_string()), "string"),
        )]),
    );

    // Template Text Agent
    defs.insert(
        "$template_text".into(),
        AgentDefinition::new(
            // We can use TemplateStringAgent for `$template_text` too,
            // since the only difference is the config type.
            "Builtin",
            "$template_text",
            Some(new_boxed::<TemplateStringAgent>),
        )
        .with_title("Template Text")
        .with_category("Core/String")
        .with_inputs(vec!["*"])
        .with_outputs(vec!["text"])
        .with_default_config(vec![(
            "template".into(),
            AgentConfigEntry::new(AgentValue::new_text("".to_string()), "text"),
        )]),
    );
}
