use std::sync::Arc;

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::{value::Index, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentData {
    pub kind: String,
    pub value: AgentValue,
}

impl AgentData {
    pub fn new_unit() -> Self {
        Self {
            kind: "unit".to_string(),
            value: AgentValue::new_unit(),
        }
    }

    pub fn new_boolean(value: bool) -> Self {
        AgentData {
            kind: "boolean".to_string(),
            value: AgentValue::new_boolean(value),
        }
    }

    pub fn new_integer(value: i64) -> Self {
        AgentData {
            kind: "integer".to_string(),
            value: AgentValue::new_integer(value),
        }
    }

    #[allow(unused)]
    pub fn new_number(value: f64) -> Self {
        AgentData {
            kind: "number".to_string(),
            value: AgentValue::new_number(value),
        }
    }

    #[allow(unused)]
    pub fn new_string(value: String) -> Self {
        AgentData {
            kind: "string".to_string(),
            value: AgentValue::new_string(value),
        }
    }

    #[allow(unused)]
    pub fn new_text(value: String) -> Self {
        AgentData {
            kind: "text".to_string(),
            value: AgentValue::new_text(value),
        }
    }

    #[allow(unused)]
    pub fn new_object(value: Value) -> Self {
        AgentData {
            kind: "object".to_string(),
            value: AgentValue::new_object(value),
        }
    }

    pub fn from_kind_value(kind: String, value: Value) -> Self {
        let value = AgentValue::from_kind_value(&kind, value);
        Self { kind, value }
    }
}

#[derive(Debug, Clone)]
pub enum AgentValue {
    // Primitive types stored directly
    Null,
    Boolean(bool),
    Integer(i64),
    Number(f64),

    // Larger data structures use reference counting
    String(Arc<String>),
    Text(Arc<String>),
    Object(Arc<Value>),
}

impl AgentValue {
    #[allow(unused)]
    pub fn is_null(&self) -> bool {
        matches!(self, AgentValue::Null)
    }

    #[allow(unused)]
    pub fn is_boolean(&self) -> bool {
        matches!(self, AgentValue::Boolean(_))
    }

    #[allow(unused)]
    pub fn is_integer(&self) -> bool {
        matches!(self, AgentValue::Integer(_))
    }

    #[allow(unused)]
    pub fn is_number(&self) -> bool {
        matches!(self, AgentValue::Number(_))
    }

    #[allow(unused)]
    pub fn is_string(&self) -> bool {
        matches!(self, AgentValue::String(_))
    }

    #[allow(unused)]
    pub fn is_text(&self) -> bool {
        matches!(self, AgentValue::Text(_))
    }

    #[allow(unused)]
    pub fn is_object(&self) -> bool {
        matches!(self, AgentValue::Object(_))
    }

    pub fn new_unit() -> Self {
        AgentValue::Null
    }

    pub fn new_boolean(value: bool) -> Self {
        AgentValue::Boolean(value)
    }

    pub fn new_integer(value: i64) -> Self {
        AgentValue::Integer(value)
    }

    pub fn new_number(value: f64) -> Self {
        AgentValue::Number(value)
    }

    pub fn new_string(value: String) -> Self {
        AgentValue::String(Arc::new(value))
    }

    pub fn new_text(value: String) -> Self {
        AgentValue::Text(Arc::new(value))
    }

    pub fn new_object(value: Value) -> Self {
        AgentValue::Object(Arc::new(value))
    }

    pub fn from_kind_value(kind: &str, value: Value) -> Self {
        match kind {
            "unit" => AgentValue::Null,
            "boolean" => AgentValue::Boolean(value.as_bool().unwrap_or(false)),
            "integer" => AgentValue::Integer(value.as_i64().unwrap_or(0)),
            "number" => AgentValue::Number(value.as_f64().unwrap_or(0.0)),
            "string" => AgentValue::String(Arc::new(value.as_str().unwrap_or("").to_string())),
            "text" => AgentValue::Text(Arc::new(value.as_str().unwrap_or("").to_string())),
            _ => AgentValue::Object(Arc::new(value)),
        }
    }

    // pub fn from_json_str(json_str: &str) -> Self {
    //     match serde_json::from_str::<Value>(json_str) {
    //         Ok(value) => {
    //             // if value has "kind" and "value" fields, use them
    //             if let Some(kind) = value.get("kind").and_then(Value::as_str) {
    //                 if let Some(value) = value.get("value") {
    //                     return AgentValue::from_kind_value(kind, value.clone());
    //                 }
    //             }
    //             return AgentValue::from_json_value(value);
    //         }
    //         Err(_) => AgentValue::new_string(json_str.to_string()),
    //     }
    // }

    pub fn from_json_value(value: Value) -> Self {
        match value {
            Value::Null => AgentValue::Null,
            Value::Bool(b) => AgentValue::Boolean(b),
            Value::Number(_) => {
                if let Some(i) = value.as_i64() {
                    AgentValue::Integer(i)
                } else if let Some(f) = value.as_f64() {
                    AgentValue::Number(f)
                } else {
                    AgentValue::Object(Arc::new(value))
                }
            }
            Value::String(s) => AgentValue::new_string(s),
            Value::Array(_) => AgentValue::Object(Arc::new(value)),
            _ => AgentValue::Object(Arc::new(value)),
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            AgentValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    #[allow(unused)]
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            AgentValue::Integer(i) => Some(*i),
            _ => None,
        }
    }

    #[allow(unused)]
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            AgentValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            AgentValue::String(s) => Some(s),
            AgentValue::Text(t) => Some(t),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&Value> {
        match self {
            AgentValue::Object(o) => Some(o),
            _ => None,
        }
    }

    #[allow(unused)]
    pub fn get<I: Index>(&self, index: I) -> Option<&Value> {
        match self {
            AgentValue::Object(o) => o.get(index),
            _ => None,
        }
    }
}

impl Default for AgentValue {
    fn default() -> Self {
        AgentValue::Null
    }
}

impl Serialize for AgentValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            AgentValue::Null => serializer.serialize_none(),
            AgentValue::Boolean(b) => serializer.serialize_bool(*b),
            AgentValue::Integer(i) => serializer.serialize_i64(*i),
            AgentValue::Number(n) => serializer.serialize_f64(*n),
            AgentValue::String(s) => serializer.serialize_str(s),
            AgentValue::Text(t) => serializer.serialize_str(t),
            AgentValue::Object(o) => o.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for AgentValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        Ok(AgentValue::from_json_value(value))
    }
}
