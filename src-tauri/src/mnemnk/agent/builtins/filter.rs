use anyhow::{bail, Context as _, Result};
use regex::RegexSet;
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::definition::AGENT_KIND_BUILTIN;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentConfigEntry, AgentContext, AgentData, AgentDefinition,
    AgentDefinitions, AgentValue, AsAgent, AsAgentData,
};

// Truthy Pass agent
struct TruthyPassAgent {
    data: AsAgentData,
}

impl AsAgent for TruthyPassAgent {
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
        if is_truthy(&data) {
            self.try_output(ch, data)?;
        }
        Ok(())
    }
}

fn is_truthy(data: &AgentData) -> bool {
    match &data.value {
        AgentValue::Boolean(b) => *b,
        AgentValue::Integer(n) => *n != 0,
        AgentValue::Number(n) => *n != 0.0,
        AgentValue::String(s) => !s.is_empty(),
        AgentValue::Text(s) => !s.is_empty(),
        AgentValue::Array(a) => !a.is_empty(),
        AgentValue::Object(v) => match &**v {
            serde_json::Value::Bool(b) => *b,
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    i != 0
                } else if let Some(f) = n.as_f64() {
                    f != 0.0
                } else {
                    false
                }
            }
            serde_json::Value::String(s) => !s.is_empty(),
            serde_json::Value::Array(a) => !a.is_empty(),
            serde_json::Value::Object(o) => !o.is_empty(),
            _ => false,
        },
        _ => false,
    }
}

// Falsy Pass agent
struct FalsyPassAgent {
    data: AsAgentData,
}

impl AsAgent for FalsyPassAgent {
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
        if !is_truthy(&data) {
            self.try_output(ch, data)?;
        }
        Ok(())
    }
}

// Allow List
struct PassOrBlockRegexListAgent {
    data: AsAgentData,
    regex_set: Option<RegexSet>,
}

impl PassOrBlockRegexListAgent {
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

impl AsAgent for PassOrBlockRegexListAgent {
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

    fn process(&mut self, ch: String, data: AgentData) -> Result<()> {
        let config = self.config().context("missing config")?;
        let field = config.get_string_or_default(CONFIG_FIELD);
        if field.is_empty() {
            bail!("field is not set");
        }

        if self.def_name() == "$pass_regex_list" {
            if self.is_match(&data, &field) {
                // value matches the regex
                self.try_output(ch, data)
                    .context("Failed to output result")?;
            }
        } else if self.def_name() == "$block_regex_list" {
            if !self.is_match(&data, &field) {
                // value does not match the regex
                self.try_output(ch, data)
                    .context("Failed to output result")?;
            }
        }

        Ok(())
    }
}

static CATEGORY: &str = "Core/Filter";

static CONFIG_FIELD: &str = "field";
static CONFIG_REGEX_LIST: &str = "regex_list";

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    defs.insert(
        "$truthy_pass".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$truthy_pass",
            Some(new_boxed::<TruthyPassAgent>),
        )
        .with_title("Truthy Pass")
        .with_category(CATEGORY)
        .with_inputs(vec!["*"])
        .with_outputs(vec!["*"]),
    );

    defs.insert(
        "$falsy_pass".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$falsy_pass",
            Some(new_boxed::<FalsyPassAgent>),
        )
        .with_title("Falsy Pass")
        .with_category(CATEGORY)
        .with_inputs(vec!["*"])
        .with_outputs(vec!["*"]),
    );

    defs.insert(
        "$pass_regex_list".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$pass_regex_list",
            Some(new_boxed::<PassOrBlockRegexListAgent>),
        )
        .with_title("Pass Regex List")
        .with_category(CATEGORY)
        .with_inputs(vec!["*"])
        .with_outputs(vec!["*"])
        .with_default_config(vec![
            (
                CONFIG_FIELD.into(),
                AgentConfigEntry::new(AgentValue::new_string(""), "string").with_title("Field"),
            ),
            (
                CONFIG_REGEX_LIST.into(),
                AgentConfigEntry::new(AgentValue::new_text(""), "text").with_title("regex list"),
            ),
        ]),
    );

    defs.insert(
        "$block_regex_list".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$block_regex_list",
            Some(new_boxed::<PassOrBlockRegexListAgent>),
        )
        .with_title("Block Regex List")
        .with_category(CATEGORY)
        .with_inputs(vec!["*"])
        .with_outputs(vec!["*"])
        .with_default_config(vec![
            (
                CONFIG_FIELD.into(),
                AgentConfigEntry::new(AgentValue::new_string(""), "string").with_title("Field"),
            ),
            (
                CONFIG_REGEX_LIST.into(),
                AgentConfigEntry::new(AgentValue::new_text(""), "text").with_title("regex list"),
            ),
        ]),
    );
}
