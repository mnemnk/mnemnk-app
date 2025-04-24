use std::sync::Arc;

use anyhow::Result;
use serde::{ser::SerializeSeq, Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
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

    pub fn from_json_value(value: Value) -> Self {
        match value {
            Value::Null => AgentData {
                kind: "unit".to_string(),
                value: AgentValue::Null,
            },
            Value::Bool(b) => AgentData::new_boolean(b),
            Value::Number(_) => {
                if let Some(i) = value.as_i64() {
                    AgentData::new_integer(i)
                } else if let Some(f) = value.as_f64() {
                    AgentData::new_number(f)
                } else {
                    AgentData::new_object(value)
                }
            }
            Value::String(s) => AgentData::new_string(s),
            Value::Array(arr) => AgentData {
                kind: "object".to_string(),
                value: AgentValue::new_array(
                    arr.into_iter()
                        .map(|v| AgentValue::from_json_value(v))
                        .collect(),
                ),
            },
            _ => AgentData::new_object(value),
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        self.value.as_str()
    }

    pub fn as_object(&self) -> Option<&Value> {
        self.value.as_object()
    }
}

impl<'de> Deserialize<'de> for AgentData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        Ok(AgentData::from_json_value(value))
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
    Array(Arc<Vec<AgentValue>>),
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
    pub fn is_array(&self) -> bool {
        matches!(self, AgentValue::Array(_))
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

    pub fn new_array(value: Vec<AgentValue>) -> Self {
        AgentValue::Array(Arc::new(value))
    }

    pub fn new_object(value: Value) -> Self {
        AgentValue::Object(Arc::new(value))
    }

    pub fn from_kind_value(kind: &str, value: Value) -> Self {
        match kind {
            "unit" => {
                if let Value::Array(a) = value {
                    AgentValue::Array(Arc::new(a.into_iter().map(|_| AgentValue::Null).collect()))
                } else {
                    AgentValue::Null
                }
            }
            "boolean" => match value {
                Value::Bool(b) => AgentValue::Boolean(b),
                Value::Array(a) => AgentValue::Array(Arc::new(
                    a.into_iter()
                        .map(|v| {
                            if let Some(b) = v.as_bool() {
                                AgentValue::Boolean(b)
                            } else {
                                AgentValue::Null
                            }
                        })
                        .collect(),
                )),
                _ => AgentValue::Null,
            },
            "integer" => match value {
                Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        AgentValue::Integer(i)
                    } else if let Some(f) = n.as_f64() {
                        AgentValue::Number(f)
                    } else {
                        AgentValue::Null
                    }
                }
                Value::Array(a) => AgentValue::Array(Arc::new(
                    a.into_iter()
                        .map(|n| {
                            if let Some(i) = n.as_i64() {
                                AgentValue::Integer(i)
                            } else if let Some(f) = n.as_f64() {
                                AgentValue::Number(f)
                            } else {
                                AgentValue::Null
                            }
                        })
                        .collect(),
                )),
                _ => AgentValue::Null,
            },
            "number" => match value {
                Value::Number(n) => {
                    if let Some(f) = n.as_f64() {
                        AgentValue::Number(f)
                    } else if let Some(i) = n.as_i64() {
                        AgentValue::Number(i as f64)
                    } else {
                        AgentValue::Null
                    }
                }
                Value::Array(a) => AgentValue::Array(Arc::new(
                    a.into_iter()
                        .map(|n| {
                            if let Some(f) = n.as_f64() {
                                AgentValue::Number(f)
                            } else if let Some(i) = n.as_i64() {
                                AgentValue::Integer(i)
                            } else {
                                AgentValue::Null
                            }
                        })
                        .collect(),
                )),
                _ => AgentValue::Null,
            },
            "string" => match value {
                Value::String(s) => AgentValue::String(Arc::new(s)),
                Value::Array(a) => AgentValue::Array(Arc::new(
                    a.into_iter()
                        .map(|v| {
                            if let Value::String(s) = v {
                                AgentValue::String(Arc::new(s))
                            } else {
                                AgentValue::Null
                            }
                        })
                        .collect(),
                )),
                _ => AgentValue::Null,
            },
            "text" => match value {
                Value::String(s) => AgentValue::Text(Arc::new(s)),
                Value::Array(a) => AgentValue::Array(Arc::new(
                    a.into_iter()
                        .map(|v| {
                            if let Value::String(s) = v {
                                AgentValue::Text(Arc::new(s))
                            } else {
                                AgentValue::Null
                            }
                        })
                        .collect(),
                )),
                _ => AgentValue::Null,
            },
            "array" => AgentValue::from_json_value(value),
            _ => AgentValue::from_json_value(value),
        }
    }

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
            Value::Array(arr) => AgentValue::new_array(
                arr.into_iter()
                    .map(|v| AgentValue::from_json_value(v))
                    .collect(),
            ),
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

    #[allow(unused)]
    pub fn as_array(&self) -> Option<&Vec<AgentValue>> {
        match self {
            AgentValue::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&Value> {
        match self {
            AgentValue::Object(o) => Some(o),
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
            AgentValue::Array(a) => {
                let mut seq = serializer.serialize_seq(Some(a.len()))?;
                for e in a.iter() {
                    seq.serialize_element(e)?;
                }
                seq.end()
            }
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
