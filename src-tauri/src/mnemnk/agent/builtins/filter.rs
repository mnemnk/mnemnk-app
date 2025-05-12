use anyhow::{bail, Context as _, Result};
use regex::RegexSet;
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::definition::AGENT_KIND_BUILTIN;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentConfigEntry, AgentContext, AgentData, AgentDefinition,
    AgentDefinitions, AgentOutput, AgentValue, AsAgent, AsAgentData,
};

/// `BooleanFilterAgent` filters data based on a boolean condition.
/// It checks if the data is truthy or falsy.
struct BooleanFilterAgent {
    data: AsAgentData,
}

impl AsAgent for BooleanFilterAgent {
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

    fn process(&mut self, ctx: AgentContext, data: AgentData) -> Result<()> {
        if is_truthy(&data) {
            self.try_output(ctx, CH_TRUE, data)?;
        } else {
            self.try_output(ctx, CH_FALSE, data)?;
        }
        Ok(())
    }
}

pub fn is_truthy(data: &AgentData) -> bool {
    match &data.value {
        AgentValue::Boolean(b) => *b,
        AgentValue::Integer(n) => *n != 0,
        AgentValue::Number(n) => *n != 0.0,
        AgentValue::String(s) => !s.is_empty(),
        AgentValue::Array(a) => !a.is_empty(),
        AgentValue::Object(v) => !v.is_empty(),
        _ => false,
    }
}

/// `RegexListFilterAgent` filters data based on a list of regular expressions.
/// It checks if a specified field in the data matches any of the regexes in the list.
struct RegexListFilterAgent {
    data: AsAgentData,
    regex_set: Option<RegexSet>,
}

impl RegexListFilterAgent {
    fn parse_regex_list(regex_list: &str) -> Option<RegexSet> {
        let regex_list: Vec<String> = regex_list
            .split_terminator('\n')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .map(|s| format!("^{}", s))
            .collect();
        if regex_list.is_empty() {
            return None;
        }
        match RegexSet::new(regex_list) {
            Ok(set) => {
                return Some(set);
            }
            Err(e) => {
                log::error!("Failed to parse regex list: {}", e);
                return None;
            }
        }
    }

    fn is_match(&self, data: &AgentData, field: &str) -> bool {
        if self.regex_set.is_none() {
            return false;
        }
        if let AgentValue::Object(value) = &data.value {
            let Some(key_value) = value.get(field) else {
                // value does not have the field
                return false;
            };
            let Some(key_value) = key_value.as_str().map(|s| s.to_string()) else {
                // value is not a string
                return false;
            };
            return self.regex_set.as_ref().unwrap().is_match(&key_value);
        }
        false
    }
}

impl AsAgent for RegexListFilterAgent {
    fn new(
        app: AppHandle,
        id: String,
        def_name: String,
        config: Option<AgentConfig>,
    ) -> Result<Self> {
        let allow_list = config
            .as_ref()
            .and_then(|c| c.get_string(CONFIG_REGEX_LIST))
            .unwrap_or_default();

        Ok(Self {
            data: AsAgentData::new(app, id, def_name, config),
            regex_set: Self::parse_regex_list(&allow_list),
        })
    }

    fn data(&self) -> &AsAgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AsAgentData {
        &mut self.data
    }

    fn set_config(&mut self, config: AgentConfig) -> Result<()> {
        let allow_list = config.get_string(CONFIG_REGEX_LIST);
        if allow_list.is_none() {
            return Ok(());
        }
        self.regex_set = Self::parse_regex_list(allow_list.unwrap().as_str());
        Ok(())
    }

    fn process(&mut self, ctx: AgentContext, data: AgentData) -> Result<()> {
        let config = self.config().context("missing config")?;
        let field = config.get_string_or_default(CONFIG_FIELD);
        if field.is_empty() {
            bail!("field is not set");
        }

        if self.is_match(&data, &field) {
            self.try_output(ctx, CH_TRUE, data)
                .context("Failed to output result")?;
        } else {
            self.try_output(ctx, CH_FALSE, data)
                .context("Failed to output result")?;
        }

        Ok(())
    }
}

static CATEGORY: &str = "Core/Filter";

static CH_DATA: &str = "data";
static CH_FALSE: &str = "false";
static CH_TRUE: &str = "true";

static CONFIG_FIELD: &str = "field";
static CONFIG_REGEX_LIST: &str = "regex_list";

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    defs.insert(
        "$boolean_filter".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$boolean_filter",
            Some(new_boxed::<BooleanFilterAgent>),
        )
        .with_title("Boolean Filter")
        .with_category(CATEGORY)
        .with_inputs(vec!["*"])
        .with_outputs(vec![CH_TRUE, CH_FALSE]),
    );

    defs.insert(
        "$regex_list_filter".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$regex_list_filter",
            Some(new_boxed::<RegexListFilterAgent>),
        )
        .with_title("Regex List Filter")
        .with_category(CATEGORY)
        .with_inputs(vec![CH_DATA])
        .with_outputs(vec![CH_TRUE, CH_FALSE])
        .with_default_config(vec![
            (
                CONFIG_FIELD.into(),
                AgentConfigEntry::new(AgentValue::new_string(""), "string").with_title("Field"),
            ),
            (
                CONFIG_REGEX_LIST.into(),
                AgentConfigEntry::new(AgentValue::new_string(""), "text").with_title("regex list"),
            ),
        ]),
    );
}
