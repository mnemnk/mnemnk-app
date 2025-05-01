use std::collections::{BTreeMap, HashMap};

use serde::{Deserialize, Serialize};

use super::data::AgentValue;

pub type AgentConfigs = HashMap<String, AgentConfig>;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AgentConfig(BTreeMap<String, AgentValue>);

impl AgentConfig {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn set(&mut self, key: String, value: AgentValue) {
        self.0.insert(key, value);
    }

    pub fn contains_key(&self, key: &str) -> bool {
        self.0.contains_key(key)
    }

    pub fn get(&self, key: &str) -> Option<&AgentValue> {
        self.0.get(key)
    }

    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.0.get(key).and_then(|v| v.as_bool())
    }

    pub fn get_bool_or(&self, key: &str, default: bool) -> bool {
        self.get_bool(key).unwrap_or(default)
    }

    pub fn get_bool_or_default(&self, key: &str) -> bool {
        self.get_bool(key).unwrap_or_default()
    }

    pub fn get_integer(&self, key: &str) -> Option<i64> {
        self.0.get(key).and_then(|v| v.as_i64())
    }

    pub fn get_integer_or(&self, key: &str, default: i64) -> i64 {
        self.get_integer(key).unwrap_or(default)
    }

    pub fn get_integer_or_default(&self, key: &str) -> i64 {
        self.get_integer(key).unwrap_or_default()
    }

    pub fn get_number(&self, key: &str) -> Option<f64> {
        self.0.get(key).and_then(|v| v.as_f64())
    }

    pub fn get_number_or(&self, key: &str, default: f64) -> f64 {
        self.get_number(key).unwrap_or(default)
    }

    pub fn get_number_or_default(&self, key: &str) -> f64 {
        self.get_number(key).unwrap_or_default()
    }

    pub fn get_string(&self, key: &str) -> Option<String> {
        self.0
            .get(key)
            .and_then(|v| v.as_str())
            .map(|v| v.to_string())
    }

    pub fn get_string_or(&self, key: &str, default: impl Into<String>) -> String {
        self.get_string(key).unwrap_or_else(|| default.into())
    }

    pub fn get_string_or_default(&self, key: &str) -> String {
        self.get_string(key).unwrap_or_default()
    }

    pub fn get_array(&self, key: &str) -> Option<&Vec<AgentValue>> {
        self.0.get(key).and_then(|v| v.as_array())
    }

    pub fn get_array_or<'a>(
        &'a self,
        key: &str,
        default: &'a Vec<AgentValue>,
    ) -> &'a Vec<AgentValue> {
        self.get_array(key).unwrap_or(default)
    }

    pub fn get_object(&self, key: &str) -> Option<&BTreeMap<String, AgentValue>> {
        self.0.get(key).and_then(|v| v.as_object())
    }

    // pub fn get_object_or<'a>(
    //     &'a self,
    //     key: &str,
    //     default: &'a BTreeMap<String, AgentValue>,
    // ) -> &'a BTreeMap<String, AgentValue> {
    //     self.get_object(key).unwrap_or(default)
    // }
}

impl IntoIterator for AgentConfig {
    type Item = (String, AgentValue);
    type IntoIter = std::collections::btree_map::IntoIter<String, AgentValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a AgentConfig {
    type Item = (&'a String, &'a AgentValue);
    type IntoIter = std::collections::btree_map::Iter<'a, String, AgentValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
