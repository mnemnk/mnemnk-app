use anyhow::{bail, Context as _, Result};
use handlebars::Handlebars;
use regex::{Regex, RegexSet};
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::definition::AGENT_KIND_BUILTIN;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentConfigEntry, AgentContext, AgentData, AgentDefinition,
    AgentDefinitions, AgentValue, AsAgent, AsAgentData,
};

// Allow List
struct AllowListAgent {
    data: AsAgentData,
    regex_set: Option<RegexSet>,
}

impl AllowListAgent {
    fn parse_allow_list(allow_list: &str) -> Option<RegexSet> {
        let allow_list: Vec<String> = allow_list
            .split_terminator('\n')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .map(|s| format!("^{}", s))
            .collect();
        if allow_list.is_empty() {
            return None;
        }
        match RegexSet::new(allow_list) {
            Ok(set) => {
                return Some(set);
            }
            Err(e) => {
                log::error!("Failed to parse allow list regex: {}", e);
                return None;
            }
        }
    }
}

impl AsAgent for AllowListAgent {
    fn new(
        app: AppHandle,
        id: String,
        def_name: String,
        config: Option<AgentConfig>,
    ) -> Result<Self> {
        let allow_list = config
            .as_ref()
            .and_then(|c| c.get_string(CONFIG_ALLOW_LIST))
            .unwrap_or_default();

        Ok(Self {
            data: AsAgentData::new(app, id, def_name, config),
            regex_set: AllowListAgent::parse_allow_list(&allow_list),
        })
    }

    fn data(&self) -> &AsAgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AsAgentData {
        &mut self.data
    }

    fn set_config(&mut self, config: AgentConfig) -> Result<()> {
        let allow_list = config.get_string(CONFIG_ALLOW_LIST);
        if allow_list.is_none() {
            return Ok(());
        }
        self.regex_set = AllowListAgent::parse_allow_list(allow_list.unwrap().as_str());
        Ok(())
    }

    fn process(&mut self, ch: String, data: AgentData) -> Result<()> {
        let config = self.config().context("missing config")?;
        let field = config.get_string_or_default(CONFIG_FIELD);
        if field.is_empty() {
            bail!("field is not set");
        }
        if self.regex_set.is_none() {
            // no regex set
            return Ok(());
        }

        if let AgentValue::Object(value) = &data.value {
            let Some(key_value) = value.get(field) else {
                // value does not have the field
                return Ok(());
            };
            let key_value = key_value.as_str().map(|s| s.to_string());
            if key_value.is_none() {
                // value is not a string
                return Ok(());
            }
            if self
                .regex_set
                .as_ref()
                .unwrap()
                .is_match(key_value.unwrap().as_str())
            {
                // value matches the regex
                self.try_output(ch, data)
                    .context("Failed to output a result")?;
            }
        }

        Ok(())
    }
}

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
        let config = self.config().context("missing config")?;
        let field = config.get_string_or_default(CONFIG_FIELD);
        if field.is_empty() {
            bail!("field is not set");
        }

        let regex = config.get_string_or_default(CONFIG_REGEX);
        if regex.is_empty() {
            bail!("regex is not set");
        }
        let regex = Regex::new(&regex).context("Failed to compile regex")?;

        if let AgentValue::Object(value) = &data.value {
            let Some(key_value) = value.get(field) else {
                // value does not have the field
                return Ok(());
            };
            let key_value = key_value.as_str().map(|s| s.to_string());
            if key_value.is_none() {
                // value is not a string
                return Ok(());
            }
            if regex.is_match(key_value.unwrap().as_str()) {
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
        let config = self.config().context("missing config")?;

        let template = config.get_string_or_default(CONFIG_TEMPLATE);
        if template.is_empty() {
            bail!("template is not set");
        }

        let reg = Handlebars::new();
        let rendered_string = reg.render_template(&template, &data)?;

        let (kind, out_value) = match self.def_name() {
            "$template_string" => ("string", AgentValue::new_string(rendered_string)),
            "$template_text" => ("text", AgentValue::new_text(rendered_string)),
            _ => bail!("Invalid def_name"),
        };

        self.try_output(
            ch,
            AgentData {
                kind: kind.to_string(),
                value: out_value,
            },
        )
        .context("Failed to output template")
    }
}

static CATEGORY: &str = "Core/String";

static CH_STRING: &str = "string";
static CH_TEXT: &str = "text";

static CONFIG_ALLOW_LIST: &str = "allow_list";
static CONFIG_FIELD: &str = "field";
static CONFIG_REGEX: &str = "regex";
static CONFIG_TEMPLATE: &str = "template";

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    defs.insert(
        "$allow_list".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$allow_list",
            Some(new_boxed::<AllowListAgent>),
        )
        .with_title("Allow List")
        .with_category(CATEGORY)
        .with_inputs(vec!["*"])
        .with_outputs(vec!["*"])
        .with_default_config(vec![
            (
                CONFIG_FIELD.into(),
                AgentConfigEntry::new(AgentValue::new_string(""), "string").with_title("Field"),
            ),
            (
                CONFIG_ALLOW_LIST.into(),
                AgentConfigEntry::new(AgentValue::new_text(""), "text").with_title("allow list"),
            ),
        ]),
    );

    defs.insert(
        "$regex_filter".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$regex_filter",
            Some(new_boxed::<RegexFilterAgent>),
        )
        .with_title("Regex Filter")
        .with_category(CATEGORY)
        .with_inputs(vec!["*"])
        .with_outputs(vec!["*"])
        .with_default_config(vec![
            (
                CONFIG_FIELD.into(),
                AgentConfigEntry::new(AgentValue::new_string(""), "string").with_title("Field"),
            ),
            (
                CONFIG_REGEX.into(),
                AgentConfigEntry::new(AgentValue::new_text(""), "text").with_title("Regex"),
            ),
        ]),
    );

    // Template String Agent
    defs.insert(
        "$template_string".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$template_string",
            Some(new_boxed::<TemplateStringAgent>),
        )
        .with_title("Template String")
        .with_category(CATEGORY)
        .with_inputs(vec!["*"])
        .with_outputs(vec![CH_STRING])
        .with_default_config(vec![(
            CONFIG_TEMPLATE.into(),
            AgentConfigEntry::new(AgentValue::new_string(""), "string"),
        )]),
    );

    // Template Text Agent
    defs.insert(
        "$template_text".into(),
        AgentDefinition::new(
            // We can use TemplateStringAgent for `$template_text` too,
            // since the only difference is the config type.
            AGENT_KIND_BUILTIN,
            "$template_text",
            Some(new_boxed::<TemplateStringAgent>),
        )
        .with_title("Template Text")
        .with_category(CATEGORY)
        .with_inputs(vec!["*"])
        .with_outputs(vec![CH_TEXT])
        .with_default_config(vec![(
            CONFIG_TEMPLATE.into(),
            AgentConfigEntry::new(AgentValue::new_text(""), "text"),
        )]),
    );
}
