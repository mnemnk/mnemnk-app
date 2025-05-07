use std::{collections::BTreeMap, sync::Arc};

use serde::{Deserialize, Serialize};

use super::data::AgentValue;

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct AgentContext {
    ch: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    vars: Option<Arc<BTreeMap<String, AgentValue>>>,
}

impl AgentContext {
    pub fn new() -> Self {
        Self::default()
    }

    // ch

    pub fn new_with_ch(ch: impl Into<String>) -> Self {
        Self {
            ch: ch.into(),
            vars: None,
        }
    }

    pub fn with_ch(&self, ch: impl Into<String>) -> Self {
        Self {
            ch: ch.into(),
            vars: self.vars.clone(),
        }
    }

    pub fn ch(&self) -> &str {
        &self.ch
    }

    // vars

    pub fn get_var(&self, key: &str) -> Option<&AgentValue> {
        self.vars.as_ref().and_then(|vars| vars.get(key))
    }

    pub fn with_var(&self, key: String, value: AgentValue) -> Self {
        let mut vars = if let Some(vars) = &self.vars {
            vars.as_ref().clone()
        } else {
            BTreeMap::new()
        };
        vars.insert(key, value);
        Self {
            ch: self.ch.clone(),
            vars: Some(Arc::new(vars)),
        }
    }
}
