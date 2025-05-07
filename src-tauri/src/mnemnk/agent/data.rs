use std::{collections::BTreeMap, sync::Arc};

use anyhow::{bail, Result};
use serde::{
    ser::{SerializeMap, SerializeSeq},
    Deserialize, Deserializer, Serialize, Serializer,
};

#[derive(Debug, Clone, PartialEq, Serialize)]
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
    pub fn new_string(value: impl Into<String>) -> Self {
        AgentData {
            kind: "string".to_string(),
            value: AgentValue::new_string(value.into()),
        }
    }

    #[allow(unused)]
    pub fn new_text(value: impl Into<String>) -> Self {
        AgentData {
            kind: "text".to_string(),
            value: AgentValue::new_string(value.into()),
        }
    }

    #[allow(unused)]
    pub fn new_object(value: AgentValueMap<String, AgentValue>) -> Self {
        AgentData {
            kind: "object".to_string(),
            value: AgentValue::new_object(value),
        }
    }

    #[allow(unused)]
    pub fn new_custom_object(
        kind: impl Into<String>,
        value: AgentValueMap<String, AgentValue>,
    ) -> Self {
        AgentData {
            kind: kind.into(),
            value: AgentValue::new_object(value),
        }
    }

    #[allow(unused)]
    pub fn new_array(kind: impl Into<String>, value: Vec<AgentValue>) -> Self {
        AgentData {
            kind: kind.into(),
            value: AgentValue::new_array(value),
        }
    }

    pub fn from_value(value: AgentValue) -> Self {
        let kind = value.kind();
        AgentData { kind, value }
    }

    pub fn from_json_data(kind: impl Into<String>, value: serde_json::Value) -> Result<Self> {
        let kind: String = kind.into();
        let value = AgentValue::from_kind_value(&kind, value)?;
        Ok(Self { kind, value })
    }

    pub fn from_json_value(json_value: serde_json::Value) -> Result<Self> {
        let value = AgentValue::from_json_value(json_value)?;
        Ok(AgentData {
            kind: value.kind(),
            value,
        })
    }

    #[allow(unused)]
    pub fn is_unit(&self) -> bool {
        self.kind == "unit"
    }

    #[allow(unused)]
    pub fn is_boolean(&self) -> bool {
        self.kind == "boolean"
    }

    #[allow(unused)]
    pub fn is_integer(&self) -> bool {
        self.kind == "integer"
    }

    #[allow(unused)]
    pub fn is_number(&self) -> bool {
        self.kind == "number"
    }

    #[allow(unused)]
    pub fn is_string(&self) -> bool {
        self.kind == "string"
    }

    #[allow(unused)]
    pub fn is_text(&self) -> bool {
        self.kind == "text"
    }

    #[allow(unused)]
    pub fn is_object(&self) -> bool {
        !self.is_unit()
            && !self.is_boolean()
            && !self.is_integer()
            && !self.is_number()
            && !self.is_string()
            && !self.is_text()
    }

    #[allow(unused)]
    pub fn is_array(&self) -> bool {
        if let AgentValue::Array(_) = &self.value {
            true
        } else {
            false
        }
    }

    #[allow(unused)]
    pub fn as_bool(&self) -> Option<bool> {
        self.value.as_bool()
    }

    #[allow(unused)]
    pub fn as_i64(&self) -> Option<i64> {
        self.value.as_i64()
    }

    #[allow(unused)]
    pub fn as_f64(&self) -> Option<f64> {
        self.value.as_f64()
    }

    pub fn as_str(&self) -> Option<&str> {
        self.value.as_str()
    }

    pub fn as_object(&self) -> Option<&AgentValueMap<String, AgentValue>> {
        self.value.as_object()
    }

    #[allow(unused)]
    pub fn as_array(&self) -> Option<&Vec<AgentValue>> {
        self.value.as_array()
    }
}

impl<'de> Deserialize<'de> for AgentData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let json_value = serde_json::Value::deserialize(deserializer)?;
        let serde_json::Value::Object(obj) = json_value else {
            return Err(serde::de::Error::custom("not a JSON object"));
        };
        let Some(kind) = obj.get("kind").and_then(|k| k.as_str()) else {
            return Err(serde::de::Error::custom("missing kind"));
        };
        let Some(value) = obj.get("value") else {
            return Err(serde::de::Error::custom("Missing value"));
        };
        AgentData::from_json_data(kind, value.to_owned()).map_err(|e| {
            serde::de::Error::custom(format!("Failed to deserialize AgentData: {}", e))
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum AgentValue {
    // Primitive types stored directly
    Null,
    Boolean(bool),
    Integer(i64),
    Number(f64),

    // Larger data structures use reference counting
    String(Arc<String>),

    // Recursive data structures
    Array(Arc<Vec<AgentValue>>),
    Object(Arc<AgentValueMap<String, AgentValue>>),
}

pub type AgentValueMap<S, T> = BTreeMap<S, T>;

impl AgentValue {
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

    pub fn new_string(value: impl Into<String>) -> Self {
        AgentValue::String(Arc::new(value.into()))
    }

    pub fn new_object(value: AgentValueMap<String, AgentValue>) -> Self {
        AgentValue::Object(Arc::new(value))
    }

    pub fn new_array(value: Vec<AgentValue>) -> Self {
        AgentValue::Array(Arc::new(value))
    }

    pub fn default_boolean() -> Self {
        AgentValue::Boolean(false)
    }

    pub fn default_integer() -> Self {
        AgentValue::Integer(0)
    }

    pub fn default_number() -> Self {
        AgentValue::Number(0.0)
    }

    pub fn default_string() -> Self {
        AgentValue::String(Arc::new(String::new()))
    }

    pub fn default_array() -> Self {
        AgentValue::Array(Arc::new(Vec::new()))
    }

    pub fn default_object() -> Self {
        AgentValue::Object(Arc::new(AgentValueMap::new()))
    }

    pub fn from_json_value(value: serde_json::Value) -> Result<Self> {
        match value {
            serde_json::Value::Null => Ok(AgentValue::Null),
            serde_json::Value::Bool(b) => Ok(AgentValue::Boolean(b)),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Ok(AgentValue::Integer(i))
                } else if let Some(f) = n.as_f64() {
                    Ok(AgentValue::Number(f))
                } else {
                    // This case should not happen, but handle it gracefully
                    Ok(AgentValue::Integer(0))
                }
            }
            serde_json::Value::String(s) => Ok(AgentValue::new_string(s)),
            serde_json::Value::Array(arr) => {
                let mut agent_arr = Vec::new();
                for v in arr {
                    agent_arr.push(AgentValue::from_json_value(v)?);
                }
                Ok(AgentValue::new_array(agent_arr))
            }
            serde_json::Value::Object(obj) => {
                let mut map = AgentValueMap::new();
                for (k, v) in obj {
                    map.insert(k, AgentValue::from_json_value(v)?);
                }
                Ok(AgentValue::new_object(map))
            }
        }
    }

    pub fn from_kind_value(kind: &str, value: serde_json::Value) -> Result<Self> {
        match kind {
            "unit" => {
                if let serde_json::Value::Array(a) = value {
                    Ok(AgentValue::Array(Arc::new(
                        a.into_iter().map(|_| AgentValue::Null).collect(),
                    )))
                } else {
                    Ok(AgentValue::Null)
                }
            }
            "boolean" => match value {
                serde_json::Value::Bool(b) => Ok(AgentValue::Boolean(b)),
                serde_json::Value::Array(a) => {
                    let mut agent_arr = Vec::new();
                    for v in a {
                        if let serde_json::Value::Bool(b) = v {
                            agent_arr.push(AgentValue::Boolean(b));
                        } else {
                            bail!("Invalid boolean value in array");
                        }
                    }
                    Ok(AgentValue::Array(Arc::new(agent_arr)))
                }
                _ => bail!("Invalid boolean value"),
            },
            "integer" => match value {
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        Ok(AgentValue::Integer(i))
                    } else if let Some(f) = n.as_f64() {
                        Ok(AgentValue::Integer(f as i64))
                    } else {
                        bail!("Invalid integer value")
                    }
                }
                serde_json::Value::Array(a) => {
                    let mut agent_arr = Vec::new();
                    for n in a {
                        if let Some(i) = n.as_i64() {
                            agent_arr.push(AgentValue::Integer(i));
                        } else if let Some(f) = n.as_f64() {
                            agent_arr.push(AgentValue::Integer(f as i64));
                        } else {
                            bail!("Invalid integer value in array");
                        }
                    }
                    Ok(AgentValue::Array(Arc::new(agent_arr)))
                }
                _ => bail!("Invalid integer value"),
            },
            "number" => match value {
                serde_json::Value::Number(n) => {
                    if let Some(f) = n.as_f64() {
                        Ok(AgentValue::Number(f))
                    } else if let Some(i) = n.as_i64() {
                        Ok(AgentValue::Number(i as f64))
                    } else {
                        bail!("Invalid number value")
                    }
                }
                serde_json::Value::Array(a) => {
                    let mut agent_arr = Vec::new();
                    for n in a {
                        if let Some(f) = n.as_f64() {
                            agent_arr.push(AgentValue::Number(f));
                        } else if let Some(i) = n.as_i64() {
                            agent_arr.push(AgentValue::Number(i as f64));
                        } else {
                            bail!("Invalid number value in array");
                        }
                    }
                    Ok(AgentValue::Array(Arc::new(agent_arr)))
                }
                _ => bail!("Invalid number value"),
            },
            "string" | "text" => match value {
                serde_json::Value::String(s) => Ok(AgentValue::new_string(s)),
                serde_json::Value::Array(a) => {
                    let mut agent_arr = Vec::new();
                    for v in a {
                        if let serde_json::Value::String(s) = v {
                            agent_arr.push(AgentValue::new_string(s));
                        } else {
                            bail!("Invalid string value in array");
                        }
                    }
                    Ok(AgentValue::Array(Arc::new(agent_arr)))
                }
                _ => bail!("Invalid string value"),
            },
            _ => match value {
                serde_json::Value::Null => Ok(AgentValue::Null),
                serde_json::Value::Bool(b) => Ok(AgentValue::Boolean(b)),
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        Ok(AgentValue::Integer(i))
                    } else if let Some(f) = n.as_f64() {
                        Ok(AgentValue::Number(f))
                    } else {
                        bail!("Invalid number value")
                    }
                }
                serde_json::Value::String(s) => Ok(AgentValue::new_string(s)),
                serde_json::Value::Array(a) => {
                    let mut agent_arr = Vec::new();
                    for v in a {
                        let agent_v = AgentValue::from_kind_value(kind, v)?;
                        agent_arr.push(agent_v);
                    }
                    Ok(AgentValue::Array(Arc::new(agent_arr)))
                }
                serde_json::Value::Object(obj) => {
                    let mut map = AgentValueMap::new();
                    for (k, v) in obj {
                        map.insert(k.clone(), AgentValue::from_json_value(v)?);
                    }
                    Ok(AgentValue::new_object(map))
                }
            },
        }
    }

    pub fn to_json_value(&self) -> serde_json::Value {
        match self {
            AgentValue::Null => serde_json::Value::Null,
            AgentValue::Boolean(b) => (*b).into(),
            AgentValue::Integer(i) => (*i).into(),
            AgentValue::Number(n) => (*n).into(),
            AgentValue::String(s) => s.as_str().into(),
            AgentValue::Object(o) => {
                let mut map = serde_json::Map::new();
                for (k, v) in o.iter() {
                    map.insert(k.clone(), v.to_json_value());
                }
                serde_json::Value::Object(map)
            }
            AgentValue::Array(a) => {
                let arr: Vec<serde_json::Value> = a.iter().map(|v| v.to_json_value()).collect();
                serde_json::Value::Array(arr)
            }
        }
    }

    #[allow(unused)]
    pub fn is_unit(&self) -> bool {
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
    pub fn is_array(&self) -> bool {
        matches!(self, AgentValue::Array(_))
    }

    #[allow(unused)]
    pub fn is_object(&self) -> bool {
        matches!(self, AgentValue::Object(_))
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            AgentValue::Boolean(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            AgentValue::Integer(i) => Some(*i),
            AgentValue::Number(n) => Some(*n as i64),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            AgentValue::Integer(i) => Some(*i as f64),
            AgentValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            AgentValue::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_object(&self) -> Option<&AgentValueMap<String, AgentValue>> {
        match self {
            AgentValue::Object(o) => Some(o),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<AgentValue>> {
        match self {
            AgentValue::Array(a) => Some(a),
            _ => None,
        }
    }

    pub fn kind(&self) -> String {
        match self {
            AgentValue::Null => "unit".to_string(),
            AgentValue::Boolean(_) => "boolean".to_string(),
            AgentValue::Integer(_) => "integer".to_string(),
            AgentValue::Number(_) => "number".to_string(),
            AgentValue::String(_) => "string".to_string(),
            AgentValue::Object(_) => "object".to_string(),
            AgentValue::Array(arr) => {
                if arr.is_empty() {
                    "array".to_string()
                } else {
                    arr[0].kind()
                }
            }
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
            AgentValue::Object(o) => {
                let mut map = serializer.serialize_map(Some(o.len()))?;
                for (k, v) in o.iter() {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
            AgentValue::Array(a) => {
                let mut seq = serializer.serialize_seq(Some(a.len()))?;
                for e in a.iter() {
                    seq.serialize_element(e)?;
                }
                seq.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for AgentValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        AgentValue::from_json_value(value).map_err(|e| {
            serde::de::Error::custom(format!("Failed to deserialize AgentValue: {}", e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_agent_data_new_constructors() {
        // Test all the constructor methods
        let unit_data = AgentData::new_unit();
        assert_eq!(unit_data.kind, "unit");
        assert_eq!(unit_data.value, AgentValue::Null);

        let bool_data = AgentData::new_boolean(true);
        assert_eq!(bool_data.kind, "boolean");
        assert_eq!(bool_data.value, AgentValue::Boolean(true));

        let int_data = AgentData::new_integer(42);
        assert_eq!(int_data.kind, "integer");
        assert_eq!(int_data.value, AgentValue::Integer(42));

        let num_data = AgentData::new_number(3.14);
        assert_eq!(num_data.kind, "number");
        assert!(matches!(num_data.value, AgentValue::Number(_)));
        if let AgentValue::Number(num) = num_data.value {
            assert!((num - 3.14).abs() < f64::EPSILON);
        }

        let str_data = AgentData::new_string("hello".to_string());
        assert_eq!(str_data.kind, "string");
        assert!(matches!(str_data.value, AgentValue::String(_)));
        assert_eq!(str_data.as_str().unwrap(), "hello");

        let text_data = AgentData::new_text("multiline\ntext\n\n".to_string());
        assert_eq!(text_data.kind, "text");
        assert!(matches!(text_data.value, AgentValue::String(_)));
        assert_eq!(text_data.as_str().unwrap(), "multiline\ntext\n\n");

        let obj_val = AgentValueMap::from([
            ("key1".to_string(), AgentValue::new_string("string1")),
            ("key2".to_string(), AgentValue::new_integer(2)),
        ]);
        let obj_data = AgentData::new_object(obj_val.clone());
        assert_eq!(obj_data.kind, "object");
        assert!(matches!(obj_data.value, AgentValue::Object(_)));
        assert_eq!(obj_data.as_object().unwrap(), &obj_val);
    }

    #[test]
    fn test_agent_data_from_kind_value() {
        // Test creating AgentData from kind and JSON value
        let unit_data = AgentData::from_json_data("unit", json!(null)).unwrap();
        assert_eq!(unit_data.kind, "unit");
        assert_eq!(unit_data.value, AgentValue::Null);

        let bool_data = AgentData::from_json_data("boolean", json!(true)).unwrap();
        assert_eq!(bool_data.kind, "boolean");
        assert_eq!(bool_data.value, AgentValue::Boolean(true));

        let int_data = AgentData::from_json_data("integer", json!(42)).unwrap();
        assert_eq!(int_data.kind, "integer");
        assert_eq!(int_data.value, AgentValue::Integer(42));

        let int_data = AgentData::from_json_data("integer", json!(3.14)).unwrap();
        assert_eq!(int_data.kind, "integer");
        assert_eq!(int_data.value, AgentValue::Integer(3));

        let num_data = AgentData::from_json_data("number", json!(3.14)).unwrap();
        assert_eq!(num_data.kind, "number");
        assert_eq!(num_data.value, AgentValue::new_number(3.14));

        let num_data = AgentData::from_json_data("number", json!(3)).unwrap();
        assert_eq!(num_data.kind, "number");
        assert_eq!(num_data.value, AgentValue::new_number(3.0));

        let str_data = AgentData::from_json_data("string", json!("hello")).unwrap();
        assert_eq!(str_data.kind, "string");
        assert_eq!(str_data.value, AgentValue::new_string("hello"));

        let str_data = AgentData::from_json_data("string", json!("hello\nworld\n\n")).unwrap();
        assert_eq!(str_data.kind, "string");
        assert_eq!(str_data.value, AgentValue::new_string("hello\nworld\n\n"));

        let text_data = AgentData::from_json_data("text", json!("hello")).unwrap();
        assert_eq!(text_data.kind, "text");
        assert_eq!(text_data.value, AgentValue::new_string("hello"));

        let text_data = AgentData::from_json_data("text", json!("hello\nworld\n\n")).unwrap();
        assert_eq!(text_data.kind, "text");
        assert_eq!(text_data.value, AgentValue::new_string("hello\nworld\n\n"));

        let obj_data =
            AgentData::from_json_data("object", json!({"key1": "string1", "key2": 2})).unwrap();
        assert_eq!(obj_data.kind, "object");
        assert_eq!(
            obj_data.value,
            AgentValue::new_object(AgentValueMap::from([
                ("key1".to_string(), AgentValue::new_string("string1")),
                ("key2".to_string(), AgentValue::new_integer(2)),
            ]))
        );

        // Test custom object kind
        let obj_data =
            AgentData::from_json_data("custom_type".to_string(), json!({"foo": "hi", "bar": 3}))
                .unwrap();
        assert_eq!(obj_data.kind, "custom_type");
        assert_eq!(
            obj_data.value,
            AgentValue::new_object(AgentValueMap::from([
                ("foo".to_string(), AgentValue::new_string("hi")),
                ("bar".to_string(), AgentValue::new_integer(3)),
            ]))
        );

        // Test array values
        let array_data = AgentData::from_json_data("unit", json!([null, null])).unwrap();
        assert_eq!(array_data.kind, "unit");
        assert_eq!(
            array_data.value,
            AgentValue::new_array(vec![AgentValue::new_unit(), AgentValue::new_unit(),])
        );

        let array_data = AgentData::from_json_data("boolean", json!([true, false])).unwrap();
        assert_eq!(array_data.kind, "boolean");
        assert_eq!(
            array_data.value,
            AgentValue::new_array(vec![
                AgentValue::new_boolean(true),
                AgentValue::new_boolean(false),
            ])
        );

        let array_data = AgentData::from_json_data("integer", json!([1, 2.1, 3.0])).unwrap();
        assert_eq!(array_data.kind, "integer");
        assert_eq!(
            array_data.value,
            AgentValue::new_array(vec![
                AgentValue::new_integer(1),
                AgentValue::new_integer(2),
                AgentValue::new_integer(3),
            ])
        );

        let array_data = AgentData::from_json_data("number", json!([1.0, 2.1, 3])).unwrap();
        assert_eq!(array_data.kind, "number");
        assert_eq!(
            array_data.value,
            AgentValue::new_array(vec![
                AgentValue::new_number(1.0),
                AgentValue::new_number(2.1),
                AgentValue::new_number(3.0),
            ])
        );

        let array_data =
            AgentData::from_json_data("string", json!(["test", "hello\nworld\n", ""])).unwrap();
        assert_eq!(array_data.kind, "string");
        assert_eq!(
            array_data.value,
            AgentValue::new_array(vec![
                AgentValue::new_string("test"),
                AgentValue::new_string("hello\nworld\n"),
                AgentValue::new_string(""),
            ])
        );

        let array_data =
            AgentData::from_json_data("text", json!(["test", "hello\nworld\n", ""])).unwrap();
        assert_eq!(array_data.kind, "text");
        assert_eq!(
            array_data.value,
            AgentValue::new_array(vec![
                AgentValue::new_string("test"),
                AgentValue::new_string("hello\nworld\n"),
                AgentValue::new_string(""),
            ])
        );

        let array_data = AgentData::from_json_data(
            "object",
            json!([{"key1":"test","key2":1}, {"key1":"test2","key2":"hi"}, {}]),
        )
        .unwrap();
        assert_eq!(array_data.kind, "object");
        assert_eq!(
            array_data.value,
            AgentValue::new_array(vec![
                AgentValue::new_object(AgentValueMap::from([
                    ("key1".to_string(), AgentValue::new_string("test")),
                    ("key2".to_string(), AgentValue::new_integer(1)),
                ])),
                AgentValue::new_object(AgentValueMap::from([
                    ("key1".to_string(), AgentValue::new_string("test2")),
                    ("key2".to_string(), AgentValue::new_string("hi")),
                ])),
                AgentValue::new_object(AgentValueMap::default()),
            ])
        );

        let array_data = AgentData::from_json_data(
            "custom",
            json!([{"key1":"test","key2":1}, {"key1":"test2","key2":"hi"}, {}]),
        )
        .unwrap();
        assert_eq!(array_data.kind, "custom");
        assert_eq!(
            array_data.value,
            AgentValue::new_array(vec![
                AgentValue::new_object(AgentValueMap::from([
                    ("key1".to_string(), AgentValue::new_string("test")),
                    ("key2".to_string(), AgentValue::new_integer(1)),
                ])),
                AgentValue::new_object(AgentValueMap::from([
                    ("key1".to_string(), AgentValue::new_string("test2")),
                    ("key2".to_string(), AgentValue::new_string("hi")),
                ])),
                AgentValue::new_object(AgentValueMap::default()),
            ])
        );
    }

    #[test]
    fn test_agent_data_from_json_value() {
        // Test automatic kind inference from JSON values
        let unit_data = AgentData::from_json_value(json!(null)).unwrap();
        assert_eq!(unit_data.kind, "unit");
        assert_eq!(unit_data.value, AgentValue::Null);

        let bool_data = AgentData::from_json_value(json!(true)).unwrap();
        assert_eq!(bool_data.kind, "boolean");
        assert_eq!(bool_data.value, AgentValue::Boolean(true));

        let int_data = AgentData::from_json_value(json!(42)).unwrap();
        assert_eq!(int_data.kind, "integer");
        assert_eq!(int_data.value, AgentValue::Integer(42));

        let num_data = AgentData::from_json_value(json!(3.14)).unwrap();
        assert_eq!(num_data.kind, "number");
        assert_eq!(num_data.value, AgentValue::new_number(3.14));

        let str_data = AgentData::from_json_value(json!("hello")).unwrap();
        assert_eq!(str_data.kind, "string");
        assert_eq!(str_data.value, AgentValue::new_string("hello"));

        let str_data = AgentData::from_json_value(json!("hello\nworld\n\n")).unwrap();
        assert_eq!(str_data.kind, "string");
        assert_eq!(str_data.value, AgentValue::new_string("hello\nworld\n\n"));

        let obj_data = AgentData::from_json_value(json!({"key1": "string1", "key2": 2})).unwrap();
        assert_eq!(obj_data.kind, "object");
        assert_eq!(
            obj_data.value,
            AgentValue::new_object(AgentValueMap::from([
                ("key1".to_string(), AgentValue::new_string("string1")),
                ("key2".to_string(), AgentValue::new_integer(2)),
            ]))
        );

        // Test array values
        let array_data = AgentData::from_json_value(json!([null, null])).unwrap();
        assert_eq!(array_data.kind, "unit");
        assert_eq!(
            array_data.value,
            AgentValue::new_array(vec![AgentValue::new_unit(), AgentValue::new_unit(),])
        );

        let array_data = AgentData::from_json_value(json!([true, false])).unwrap();
        assert_eq!(array_data.kind, "boolean");
        assert_eq!(
            array_data.value,
            AgentValue::new_array(vec![
                AgentValue::new_boolean(true),
                AgentValue::new_boolean(false),
            ])
        );

        let array_data = AgentData::from_json_value(json!([1, 2, 3])).unwrap();
        assert_eq!(array_data.kind, "integer");
        assert_eq!(
            array_data.value,
            AgentValue::new_array(vec![
                AgentValue::new_integer(1),
                AgentValue::new_integer(2),
                AgentValue::new_integer(3),
            ])
        );

        let array_data = AgentData::from_json_value(json!([1.0, 2.1, 3.2])).unwrap();
        assert_eq!(array_data.kind, "number");
        assert_eq!(
            array_data.value,
            AgentValue::new_array(vec![
                AgentValue::new_number(1.0),
                AgentValue::new_number(2.1),
                AgentValue::new_number(3.2),
            ])
        );

        let array_data = AgentData::from_json_value(json!(["test", "hello\nworld\n", ""])).unwrap();
        assert_eq!(array_data.kind, "string");
        assert_eq!(
            array_data.value,
            AgentValue::new_array(vec![
                AgentValue::new_string("test"),
                AgentValue::new_string("hello\nworld\n"),
                AgentValue::new_string(""),
            ])
        );

        let array_data = AgentData::from_json_value(
            json!([{"key1":"test","key2":1}, {"key1":"test2","key2":"hi"}, {}]),
        )
        .unwrap();
        assert_eq!(array_data.kind, "object");
        assert_eq!(
            array_data.value,
            AgentValue::new_array(vec![
                AgentValue::new_object(AgentValueMap::from([
                    ("key1".to_string(), AgentValue::new_string("test")),
                    ("key2".to_string(), AgentValue::new_integer(1)),
                ])),
                AgentValue::new_object(AgentValueMap::from([
                    ("key1".to_string(), AgentValue::new_string("test2")),
                    ("key2".to_string(), AgentValue::new_string("hi")),
                ])),
                AgentValue::new_object(AgentValueMap::default()),
            ])
        );
    }

    #[test]
    fn test_agent_data_accessor_methods() {
        // Test accessor methods
        let str_data = AgentData::new_string("hello".to_string());
        assert_eq!(str_data.as_str().unwrap(), "hello");
        assert!(str_data.as_object().is_none());

        let obj_val = AgentValueMap::from([
            ("key1".to_string(), AgentValue::new_string("string1")),
            ("key2".to_string(), AgentValue::new_integer(2)),
        ]);
        let obj_data = AgentData::new_object(obj_val.clone());
        assert!(obj_data.as_str().is_none());
        assert_eq!(obj_data.as_object().unwrap(), &obj_val);
    }

    #[test]
    fn test_agent_data_serialization() {
        // Test unit serialization
        {
            let data = AgentData::new_unit();
            assert_eq!(
                serde_json::to_string(&data).unwrap(),
                r#"{"kind":"unit","value":null,"metadata":{}}"#
            );
        }

        // Test Boolean serialization
        {
            let data = AgentData::new_boolean(true);
            assert_eq!(
                serde_json::to_string(&data).unwrap(),
                r#"{"kind":"boolean","value":true,"metadata":{}}"#
            );

            let data = AgentData::new_boolean(false);
            assert_eq!(
                serde_json::to_string(&data).unwrap(),
                r#"{"kind":"boolean","value":false,"metadata":{}}"#
            );
        }

        // Test Integer serialization
        {
            let data = AgentData::new_integer(42);
            assert_eq!(
                serde_json::to_string(&data).unwrap(),
                r#"{"kind":"integer","value":42,"metadata":{}}"#
            );
        }

        // Test Number serialization
        {
            let data = AgentData::new_number(3.14);
            assert_eq!(
                serde_json::to_string(&data).unwrap(),
                r#"{"kind":"number","value":3.14,"metadata":{}}"#
            );

            let data = AgentData::new_number(3.0);
            assert_eq!(
                serde_json::to_string(&data).unwrap(),
                r#"{"kind":"number","value":3.0,"metadata":{}}"#
            );
        }

        // Test String serialization
        {
            let data = AgentData::new_string("Hello, world!");
            assert_eq!(
                serde_json::to_string(&data).unwrap(),
                r#"{"kind":"string","value":"Hello, world!","metadata":{}}"#
            );

            let data = AgentData::new_string("hello\nworld\n\n");
            assert_eq!(
                serde_json::to_string(&data).unwrap(),
                r#"{"kind":"string","value":"hello\nworld\n\n","metadata":{}}"#
            );
        }

        // Test Text serialization
        {
            let data = AgentData::new_text("Hello, world!");
            assert_eq!(
                serde_json::to_string(&data).unwrap(),
                r#"{"kind":"text","value":"Hello, world!","metadata":{}}"#
            );

            let data = AgentData::new_text("hello\nworld\n\n");
            assert_eq!(
                serde_json::to_string(&data).unwrap(),
                r#"{"kind":"text","value":"hello\nworld\n\n","metadata":{}}"#
            );
        }

        // Test Object serialization
        {
            let data = AgentData::new_object(AgentValueMap::from([
                ("key1".to_string(), AgentValue::new_string("string1")),
                ("key2".to_string(), AgentValue::new_integer(2)),
            ]));
            assert_eq!(
                serde_json::to_string(&data).unwrap(),
                r#"{"kind":"object","value":{"key1":"string1","key2":2},"metadata":{}}"#
            );
        }

        // Test custom object serialization
        {
            let data = AgentData::new_custom_object(
                "custom",
                AgentValueMap::from([
                    ("key1".to_string(), AgentValue::new_string("test")),
                    ("key2".to_string(), AgentValue::new_integer(3)),
                ]),
            );
            assert_eq!(
                serde_json::to_string(&data).unwrap(),
                r#"{"kind":"custom","value":{"key1":"test","key2":3},"metadata":{}}"#
            );
        }

        // Test Array serialization
        {
            let data =
                AgentData::new_array("unit", vec![AgentValue::new_unit(), AgentValue::new_unit()]);
            assert_eq!(
                serde_json::to_string(&data).unwrap(),
                r#"{"kind":"unit","value":[null,null],"metadata":{}}"#
            );

            let data = AgentData::new_array(
                "boolean",
                vec![
                    AgentValue::new_boolean(false),
                    AgentValue::new_boolean(true),
                ],
            );
            assert_eq!(
                serde_json::to_string(&data).unwrap(),
                r#"{"kind":"boolean","value":[false,true],"metadata":{}}"#
            );

            let data = AgentData::new_array(
                "integer",
                vec![
                    AgentValue::new_integer(1),
                    AgentValue::new_integer(2),
                    AgentValue::new_integer(3),
                ],
            );
            assert_eq!(
                serde_json::to_string(&data).unwrap(),
                r#"{"kind":"integer","value":[1,2,3],"metadata":{}}"#
            );

            let data = AgentData::new_array(
                "number",
                vec![
                    AgentValue::new_number(1.0),
                    AgentValue::new_number(2.1),
                    AgentValue::new_number(3.2),
                ],
            );
            assert_eq!(
                serde_json::to_string(&data).unwrap(),
                r#"{"kind":"number","value":[1.0,2.1,3.2],"metadata":{}}"#
            );

            let data = AgentData::new_array(
                "string",
                vec![
                    AgentValue::new_string("test"),
                    AgentValue::new_string("hello\nworld\n"),
                    AgentValue::new_string(""),
                ],
            );
            assert_eq!(
                serde_json::to_string(&data).unwrap(),
                r#"{"kind":"string","value":["test","hello\nworld\n",""],"metadata":{}}"#
            );

            let data = AgentData::new_array(
                "text",
                vec![
                    AgentValue::new_string("test"),
                    AgentValue::new_string("hello\nworld\n"),
                    AgentValue::new_string(""),
                ],
            );
            assert_eq!(
                serde_json::to_string(&data).unwrap(),
                r#"{"kind":"text","value":["test","hello\nworld\n",""],"metadata":{}}"#
            );

            let data = AgentData::new_array(
                "object",
                vec![
                    AgentValue::new_object(AgentValueMap::from([
                        ("key1".to_string(), AgentValue::new_string("test")),
                        ("key2".to_string(), AgentValue::new_integer(1)),
                    ])),
                    AgentValue::new_object(AgentValueMap::from([
                        ("key1".to_string(), AgentValue::new_string("test2")),
                        ("key2".to_string(), AgentValue::new_string("hi")),
                    ])),
                    AgentValue::new_object(AgentValueMap::default()),
                ],
            );
            assert_eq!(
                serde_json::to_string(&data).unwrap(),
                r#"{"kind":"object","value":[{"key1":"test","key2":1},{"key1":"test2","key2":"hi"},{}],"metadata":{}}"#
            );

            let data = AgentData::new_array(
                "custom",
                vec![
                    AgentValue::new_object(AgentValueMap::from([
                        ("key1".to_string(), AgentValue::new_string("test")),
                        ("key2".to_string(), AgentValue::new_integer(1)),
                    ])),
                    AgentValue::new_object(AgentValueMap::from([
                        ("key1".to_string(), AgentValue::new_string("test2")),
                        ("key2".to_string(), AgentValue::new_string("hi")),
                    ])),
                    AgentValue::new_object(AgentValueMap::default()),
                ],
            );
            assert_eq!(
                serde_json::to_string(&data).unwrap(),
                r#"{"kind":"custom","value":[{"key1":"test","key2":1},{"key1":"test2","key2":"hi"},{}],"metadata":{}}"#
            );
        }
    }

    #[test]
    fn test_agent_data_deserialization() {
        // Test unit deserialization
        {
            let deserialized: AgentData =
                serde_json::from_str(r#"{"kind":"unit","value":null,"metadata":{}}"#).unwrap();
            assert_eq!(deserialized, AgentData::new_unit());
        }

        // Test Boolean deserialization
        {
            let deserialized: AgentData =
                serde_json::from_str(r#"{"kind":"boolean","value":false,"metadata":{}}"#).unwrap();
            assert_eq!(deserialized, AgentData::new_boolean(false));

            let deserialized: AgentData =
                serde_json::from_str(r#"{"kind":"boolean","value":true,"metadata":{}}"#).unwrap();
            assert_eq!(deserialized, AgentData::new_boolean(true));
        }

        // Test Integer deserialization
        {
            let deserialized: AgentData =
                serde_json::from_str(r#"{"kind":"integer","value":123,"metadata":{}}"#).unwrap();
            assert_eq!(deserialized, AgentData::new_integer(123));
        }

        // Test Number deserialization
        {
            let deserialized: AgentData =
                serde_json::from_str(r#"{"kind":"number","value":3.14,"metadata":{}}"#).unwrap();
            assert_eq!(deserialized, AgentData::new_number(3.14));

            let deserialized: AgentData =
                serde_json::from_str(r#"{"kind":"number","value":3.0,"metadata":{}}"#).unwrap();
            assert_eq!(deserialized, AgentData::new_number(3.0));
        }

        // Test String deserialization
        {
            let deserialized: AgentData =
                serde_json::from_str(r#"{"kind":"string","value":"Hello, world!","metadata":{}}"#)
                    .unwrap();
            assert_eq!(deserialized, AgentData::new_string("Hello, world!"));

            let deserialized: AgentData = serde_json::from_str(
                r#"{"kind":"string","value":"hello\nworld\n\n","metadata":{}}"#,
            )
            .unwrap();
            assert_eq!(deserialized, AgentData::new_string("hello\nworld\n\n"));
        }

        // Test Object deserialization
        {
            let deserialized: AgentData = serde_json::from_str(
                r#"{"kind":"object","value":{"key1":"test","key2":3},"metadata":{}}"#,
            )
            .unwrap();
            assert_eq!(
                deserialized,
                AgentData::new_object(AgentValueMap::from([
                    ("key1".to_string(), AgentValue::new_string("test")),
                    ("key2".to_string(), AgentValue::new_integer(3))
                ]))
            );
        }

        // Test custom object deserialization
        {
            let deserialized: AgentData = serde_json::from_str(
                r#"{"kind":"custom","value":{"name":"test","value":3},"metadata":{}}"#,
            )
            .unwrap();
            assert_eq!(
                deserialized,
                AgentData::new_custom_object(
                    "custom",
                    AgentValueMap::from([
                        ("name".to_string(), AgentValue::new_string("test")),
                        ("value".to_string(), AgentValue::new_integer(3))
                    ])
                )
            );
        }

        // Test Array deserialization
        {
            let deserialized: AgentData =
                serde_json::from_str(r#"{"kind":"unit","value":[null,null],"metadata":{}}"#)
                    .unwrap();
            assert_eq!(
                deserialized,
                AgentData::new_array(
                    "unit",
                    vec![AgentValue::new_unit(), AgentValue::new_unit(),]
                )
            );

            let deserialized: AgentData =
                serde_json::from_str(r#"{"kind":"boolean","value":[true,false],"metadata":{}}"#)
                    .unwrap();
            assert_eq!(
                deserialized,
                AgentData::new_array(
                    "boolean",
                    vec![
                        AgentValue::new_boolean(true),
                        AgentValue::new_boolean(false),
                    ]
                )
            );

            let deserialized: AgentData =
                serde_json::from_str(r#"{"kind":"integer","value":[1,2,3],"metadata":{}}"#)
                    .unwrap();
            assert_eq!(
                deserialized,
                AgentData::new_array(
                    "integer",
                    vec![
                        AgentValue::new_integer(1),
                        AgentValue::new_integer(2),
                        AgentValue::new_integer(3),
                    ]
                )
            );

            let deserialized: AgentData =
                serde_json::from_str(r#"{"kind":"number","value":[1.0,2.1,3],"metadata":{}}"#)
                    .unwrap();
            assert_eq!(
                deserialized,
                AgentData::new_array(
                    "number",
                    vec![
                        AgentValue::new_number(1.0),
                        AgentValue::new_number(2.1),
                        AgentValue::new_number(3.0),
                    ]
                )
            );

            let deserialized: AgentData = serde_json::from_str(
                r#"{"kind":"string","value":["test","hello\nworld\n",""],"metadata":{}}"#,
            )
            .unwrap();
            assert_eq!(
                deserialized,
                AgentData::new_array(
                    "string",
                    vec![
                        AgentValue::new_string("test"),
                        AgentValue::new_string("hello\nworld\n"),
                        AgentValue::new_string(""),
                    ]
                )
            );

            let deserialized: AgentData = serde_json::from_str(
                r#"{"kind":"text","value":["test","hello\nworld\n",""],"metadata":{}}"#,
            )
            .unwrap();
            assert_eq!(
                deserialized,
                AgentData::new_array(
                    "text",
                    vec![
                        AgentValue::new_string("test"),
                        AgentValue::new_string("hello\nworld\n"),
                        AgentValue::new_string(""),
                    ]
                )
            );

            let deserialized: AgentData =
                    serde_json::from_str(r#"{"kind":"object","value":[{"key1":"test","key2":1},{"key1":"test2","key2":"hi"},{}],"metadata":{}}"#)
                        .unwrap();
            assert_eq!(
                deserialized,
                AgentData::new_array(
                    "object",
                    vec![
                        AgentValue::new_object(AgentValueMap::from([
                            ("key1".to_string(), AgentValue::new_string("test")),
                            ("key2".to_string(), AgentValue::new_integer(1)),
                        ])),
                        AgentValue::new_object(AgentValueMap::from([
                            ("key1".to_string(), AgentValue::new_string("test2")),
                            ("key2".to_string(), AgentValue::new_string("hi")),
                        ])),
                        AgentValue::new_object(AgentValueMap::default()),
                    ]
                )
            );

            let deserialized: AgentData =
                    serde_json::from_str(r#"{"kind":"custom","value":[{"key1":"test","key2":1},{"key1":"test2","key2":"hi"},{}],"metadata":{}}"#)
                        .unwrap();
            assert_eq!(
                deserialized,
                AgentData::new_array(
                    "custom",
                    vec![
                        AgentValue::new_object(AgentValueMap::from([
                            ("key1".to_string(), AgentValue::new_string("test")),
                            ("key2".to_string(), AgentValue::new_integer(1)),
                        ])),
                        AgentValue::new_object(AgentValueMap::from([
                            ("key1".to_string(), AgentValue::new_string("test2")),
                            ("key2".to_string(), AgentValue::new_string("hi")),
                        ])),
                        AgentValue::new_object(AgentValueMap::default()),
                    ]
                )
            );
        }
    }

    #[test]
    fn test_agent_value_constructors() {
        // Test AgentValue constructors
        let unit = AgentValue::new_unit();
        assert_eq!(unit, AgentValue::Null);

        let boolean = AgentValue::new_boolean(true);
        assert_eq!(boolean, AgentValue::Boolean(true));

        let integer = AgentValue::new_integer(42);
        assert_eq!(integer, AgentValue::Integer(42));

        let number = AgentValue::new_number(3.14);
        assert!(matches!(number, AgentValue::Number(_)));
        if let AgentValue::Number(num) = number {
            assert!((num - 3.14).abs() < f64::EPSILON);
        }

        let string = AgentValue::new_string("hello");
        assert!(matches!(string, AgentValue::String(_)));
        assert_eq!(string.as_str().unwrap(), "hello");

        let text = AgentValue::new_string("multiline\ntext");
        assert!(matches!(text, AgentValue::String(_)));
        assert_eq!(text.as_str().unwrap(), "multiline\ntext");

        let array =
            AgentValue::new_array(vec![AgentValue::new_integer(1), AgentValue::new_integer(2)]);
        assert!(matches!(array, AgentValue::Array(_)));
        if let AgentValue::Array(arr) = array {
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0].as_i64().unwrap(), 1);
            assert_eq!(arr[1].as_i64().unwrap(), 2);
        }

        let obj = AgentValue::new_object(AgentValueMap::from([
            ("key1".to_string(), AgentValue::new_string("string1")),
            ("key2".to_string(), AgentValue::new_integer(2)),
        ]));
        assert!(matches!(obj, AgentValue::Object(_)));
        if let AgentValue::Object(obj) = obj {
            assert_eq!(obj.get("key1").and_then(|v| v.as_str()), Some("string1"));
            assert_eq!(obj.get("key2").and_then(|v| v.as_i64()), Some(2));
        } else {
            panic!("Object was not deserialized correctly");
        }
    }

    #[test]
    fn test_agent_value_from_json_value() {
        // Test converting from JSON value to AgentValue
        let null = AgentValue::from_json_value(json!(null)).unwrap();
        assert_eq!(null, AgentValue::Null);

        let boolean = AgentValue::from_json_value(json!(true)).unwrap();
        assert_eq!(boolean, AgentValue::Boolean(true));

        let integer = AgentValue::from_json_value(json!(42)).unwrap();
        assert_eq!(integer, AgentValue::Integer(42));

        let number = AgentValue::from_json_value(json!(3.14)).unwrap();
        assert!(matches!(number, AgentValue::Number(_)));
        if let AgentValue::Number(num) = number {
            assert!((num - 3.14).abs() < f64::EPSILON);
        }

        let string = AgentValue::from_json_value(json!("hello")).unwrap();
        assert!(matches!(string, AgentValue::String(_)));
        if let AgentValue::String(s) = string {
            assert_eq!(*s, "hello");
        } else {
            panic!("Expected string value");
        }

        let array = AgentValue::from_json_value(json!([1, "test", true])).unwrap();
        assert!(matches!(array, AgentValue::Array(_)));
        if let AgentValue::Array(arr) = array {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], AgentValue::Integer(1));
            assert!(matches!(&arr[1], AgentValue::String(_)));
            if let AgentValue::String(s) = &arr[1] {
                assert_eq!(**s, "test");
            } else {
                panic!("Expected string value");
            }
            assert_eq!(arr[2], AgentValue::Boolean(true));
        }

        let object = AgentValue::from_json_value(json!({"key1": "string1", "key2": 2})).unwrap();
        assert!(matches!(object, AgentValue::Object(_)));
        if let AgentValue::Object(obj) = object {
            assert_eq!(obj.get("key1").and_then(|v| v.as_str()), Some("string1"));
            assert_eq!(obj.get("key2").and_then(|v| v.as_i64()), Some(2));
        } else {
            panic!("Object was not deserialized correctly");
        }
    }

    #[test]
    fn test_agent_value_from_kind_value() {
        // Test AgentValue::from_kind_value with different kinds and values
        let unit = AgentValue::from_kind_value("unit", json!(null)).unwrap();
        assert_eq!(unit, AgentValue::Null);

        let boolean = AgentValue::from_kind_value("boolean", json!(true)).unwrap();
        assert_eq!(boolean, AgentValue::Boolean(true));

        let integer = AgentValue::from_kind_value("integer", json!(42)).unwrap();
        assert_eq!(integer, AgentValue::Integer(42));

        let integer = AgentValue::from_kind_value("integer", json!(42.0)).unwrap();
        assert_eq!(integer, AgentValue::Integer(42));

        let number = AgentValue::from_kind_value("number", json!(3.14)).unwrap();
        assert!(matches!(number, AgentValue::Number(_)));
        if let AgentValue::Number(num) = number {
            assert!((num - 3.14).abs() < f64::EPSILON);
        }

        let number = AgentValue::from_kind_value("number", json!(3)).unwrap();
        assert!(matches!(number, AgentValue::Number(_)));
        if let AgentValue::Number(num) = number {
            assert!((num - 3.0).abs() < f64::EPSILON);
        }

        let string = AgentValue::from_kind_value("string", json!("hello")).unwrap();
        assert!(matches!(string, AgentValue::String(_)));
        if let AgentValue::String(s) = string {
            assert_eq!(*s, "hello");
        } else {
            panic!("Expected string value");
        }

        let text = AgentValue::from_kind_value("text", json!("multiline\ntext")).unwrap();
        assert!(matches!(text, AgentValue::String(_)));
        if let AgentValue::String(t) = text {
            assert_eq!(*t, "multiline\ntext");
        } else {
            panic!("Expected text value");
        }

        let array = AgentValue::from_kind_value("array", json!([1, "test", true])).unwrap();
        assert!(matches!(array, AgentValue::Array(_)));
        if let AgentValue::Array(arr) = array {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], AgentValue::Integer(1));
            assert!(matches!(&arr[1], AgentValue::String(_)));
            if let AgentValue::String(s) = &arr[1] {
                assert_eq!(**s, "test");
            } else {
                panic!("Expected string value");
            }
            assert_eq!(arr[2], AgentValue::Boolean(true));
        }

        let obj =
            AgentValue::from_kind_value("object", json!({"key1": "test", "key2": 2})).unwrap();
        assert!(matches!(obj, AgentValue::Object(_)));
        if let AgentValue::Object(obj) = obj {
            assert_eq!(obj.get("key1").and_then(|v| v.as_str()), Some("test"));
            assert_eq!(obj.get("key2").and_then(|v| v.as_i64()), Some(2));
        } else {
            panic!("Object was not deserialized correctly");
        }

        // Test arrays
        let unit_array = AgentValue::from_kind_value("unit", json!([null, null])).unwrap();
        assert!(matches!(unit_array, AgentValue::Array(_)));
        if let AgentValue::Array(arr) = unit_array {
            assert_eq!(arr.len(), 2);
            for val in arr.iter() {
                assert_eq!(*val, AgentValue::Null);
            }
        }

        let bool_array = AgentValue::from_kind_value("boolean", json!([true, false])).unwrap();
        assert!(matches!(bool_array, AgentValue::Array(_)));
        if let AgentValue::Array(arr) = bool_array {
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0], AgentValue::Boolean(true));
            assert_eq!(arr[1], AgentValue::Boolean(false));
        }

        let int_array = AgentValue::from_kind_value("integer", json!([1, 2, 3])).unwrap();
        assert!(matches!(int_array, AgentValue::Array(_)));
        if let AgentValue::Array(arr) = int_array {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], AgentValue::Integer(1));
            assert_eq!(arr[1], AgentValue::Integer(2));
            assert_eq!(arr[2], AgentValue::Integer(3));
        }

        let num_array = AgentValue::from_kind_value("number", json!([1.1, 2.2, 3.3])).unwrap();
        assert!(matches!(num_array, AgentValue::Array(_)));
        if let AgentValue::Array(arr) = num_array {
            assert_eq!(arr.len(), 3);
            assert_eq!(arr[0], AgentValue::Number(1.1));
            assert_eq!(arr[1], AgentValue::Number(2.2));
            assert_eq!(arr[2], AgentValue::Number(3.3));
        }

        let string_array =
            AgentValue::from_kind_value("string", json!(["hello", "world"])).unwrap();
        assert!(matches!(string_array, AgentValue::Array(_)));
        if let AgentValue::Array(arr) = string_array {
            assert_eq!(arr.len(), 2);
            assert!(matches!(&arr[0], AgentValue::String(_)));
            if let AgentValue::String(s) = &arr[0] {
                assert_eq!(**s, "hello".to_string());
            }
            assert!(matches!(&arr[1], AgentValue::String(_)));
            if let AgentValue::String(s) = &arr[1] {
                assert_eq!(**s, "world".to_string());
            }
        }

        let text_array = AgentValue::from_kind_value("text", json!(["hello", "world!\n"])).unwrap();
        assert!(matches!(text_array, AgentValue::Array(_)));
        if let AgentValue::Array(arr) = text_array {
            assert_eq!(arr.len(), 2);
            assert!(matches!(&arr[0], AgentValue::String(_)));
            if let AgentValue::String(s) = &arr[0] {
                assert_eq!(**s, "hello".to_string());
            }
            assert!(matches!(&arr[1], AgentValue::String(_)));
            if let AgentValue::String(s) = &arr[1] {
                assert_eq!(**s, "world!\n".to_string());
            }
        }

        // array_array

        // object_array
    }

    #[test]
    fn test_agent_value_test_methods() {
        // Test test methods on AgentValue
        let unit = AgentValue::new_unit();
        assert_eq!(unit.is_unit(), true);
        assert_eq!(unit.is_boolean(), false);
        assert_eq!(unit.is_integer(), false);
        assert_eq!(unit.is_number(), false);
        assert_eq!(unit.is_string(), false);
        assert_eq!(unit.is_array(), false);
        assert_eq!(unit.is_object(), false);

        let boolean = AgentValue::new_boolean(true);
        assert_eq!(boolean.is_unit(), false);
        assert_eq!(boolean.is_boolean(), true);
        assert_eq!(boolean.is_integer(), false);
        assert_eq!(boolean.is_number(), false);
        assert_eq!(boolean.is_string(), false);
        assert_eq!(boolean.is_array(), false);
        assert_eq!(boolean.is_object(), false);

        let integer = AgentValue::new_integer(42);
        assert_eq!(integer.is_unit(), false);
        assert_eq!(integer.is_boolean(), false);
        assert_eq!(integer.is_integer(), true);
        assert_eq!(integer.is_number(), false);
        assert_eq!(integer.is_string(), false);
        assert_eq!(integer.is_array(), false);
        assert_eq!(integer.is_object(), false);

        let number = AgentValue::new_number(3.14);
        assert_eq!(number.is_unit(), false);
        assert_eq!(number.is_boolean(), false);
        assert_eq!(number.is_integer(), false);
        assert_eq!(number.is_number(), true);
        assert_eq!(number.is_string(), false);
        assert_eq!(number.is_array(), false);
        assert_eq!(number.is_object(), false);

        let string = AgentValue::new_string("hello");
        assert_eq!(string.is_unit(), false);
        assert_eq!(string.is_boolean(), false);
        assert_eq!(string.is_integer(), false);
        assert_eq!(string.is_number(), false);
        assert_eq!(string.is_string(), true);
        assert_eq!(string.is_array(), false);
        assert_eq!(string.is_object(), false);

        let array =
            AgentValue::new_array(vec![AgentValue::new_integer(1), AgentValue::new_integer(2)]);
        assert_eq!(array.is_unit(), false);
        assert_eq!(array.is_boolean(), false);
        assert_eq!(array.is_integer(), false);
        assert_eq!(array.is_number(), false);
        assert_eq!(array.is_string(), false);
        assert_eq!(array.is_array(), true);
        assert_eq!(array.is_object(), false);

        let obj = AgentValue::new_object(AgentValueMap::from([
            ("key1".to_string(), AgentValue::new_string("string1")),
            ("key2".to_string(), AgentValue::new_integer(2)),
        ]));
        assert_eq!(obj.is_unit(), false);
        assert_eq!(obj.is_boolean(), false);
        assert_eq!(obj.is_integer(), false);
        assert_eq!(obj.is_number(), false);
        assert_eq!(obj.is_string(), false);
        assert_eq!(obj.is_array(), false);
        assert_eq!(obj.is_object(), true);
    }

    #[test]
    fn test_agent_value_accessor_methods() {
        // Test accessor methods on AgentValue
        let boolean = AgentValue::new_boolean(true);
        assert_eq!(boolean.as_bool(), Some(true));
        assert_eq!(boolean.as_i64(), None);
        assert_eq!(boolean.as_f64(), None);
        assert_eq!(boolean.as_str(), None);
        assert!(boolean.as_array().is_none());
        assert_eq!(boolean.as_object(), None);

        let integer = AgentValue::new_integer(42);
        assert_eq!(integer.as_bool(), None);
        assert_eq!(integer.as_i64(), Some(42));
        assert_eq!(integer.as_f64(), Some(42.0));
        assert_eq!(integer.as_str(), None);
        assert!(integer.as_array().is_none());
        assert_eq!(integer.as_object(), None);

        let number = AgentValue::new_number(3.14);
        assert_eq!(number.as_bool(), None);
        assert_eq!(number.as_i64(), Some(3)); // truncated
        assert_eq!(number.as_f64().unwrap(), 3.14);
        assert_eq!(number.as_str(), None);
        assert!(number.as_array().is_none());
        assert_eq!(number.as_object(), None);

        let string = AgentValue::new_string("hello");
        assert_eq!(string.as_bool(), None);
        assert_eq!(string.as_i64(), None);
        assert_eq!(string.as_f64(), None);
        assert_eq!(string.as_str(), Some("hello"));
        assert!(string.as_array().is_none());
        assert_eq!(string.as_object(), None);

        let array =
            AgentValue::new_array(vec![AgentValue::new_integer(1), AgentValue::new_integer(2)]);
        assert_eq!(array.as_bool(), None);
        assert_eq!(array.as_i64(), None);
        assert_eq!(array.as_f64(), None);
        assert_eq!(array.as_str(), None);
        assert!(array.as_array().is_some());
        if let Some(arr) = array.as_array() {
            assert_eq!(arr.len(), 2);
            assert_eq!(arr[0].as_i64().unwrap(), 1);
            assert_eq!(arr[1].as_i64().unwrap(), 2);
        }
        assert_eq!(array.as_object(), None);

        let obj = AgentValue::new_object(AgentValueMap::from([
            ("key1".to_string(), AgentValue::new_string("string1")),
            ("key2".to_string(), AgentValue::new_integer(2)),
        ]));
        assert_eq!(obj.as_bool(), None);
        assert_eq!(obj.as_i64(), None);
        assert_eq!(obj.as_f64(), None);
        assert_eq!(obj.as_str(), None);
        assert!(obj.as_array().is_none());
        assert!(obj.as_object().is_some());
        if let Some(value) = obj.as_object() {
            assert_eq!(value.get("key1").and_then(|v| v.as_str()), Some("string1"));
            assert_eq!(value.get("key2").and_then(|v| v.as_i64()), Some(2));
        }
    }

    #[test]
    fn test_agent_value_default() {
        assert_eq!(AgentValue::default(), AgentValue::Null);
    }

    #[test]
    fn test_agent_value_serialization() {
        // Test Null serialization
        {
            let null = AgentValue::Null;
            assert_eq!(serde_json::to_string(&null).unwrap(), "null");
        }

        // Test Boolean serialization
        {
            let boolean_t = AgentValue::new_boolean(true);
            assert_eq!(serde_json::to_string(&boolean_t).unwrap(), "true");

            let boolean_f = AgentValue::new_boolean(false);
            assert_eq!(serde_json::to_string(&boolean_f).unwrap(), "false");
        }

        // Test Integer serialization
        {
            let integer = AgentValue::new_integer(42);
            assert_eq!(serde_json::to_string(&integer).unwrap(), "42");
        }

        // Test Number serialization
        {
            let num = AgentValue::new_number(3.14);
            assert_eq!(serde_json::to_string(&num).unwrap(), "3.14");

            let num = AgentValue::new_number(3.0);
            assert_eq!(serde_json::to_string(&num).unwrap(), "3.0");
        }

        // Test String serialization
        {
            let s = AgentValue::new_string("Hello, world!");
            assert_eq!(serde_json::to_string(&s).unwrap(), "\"Hello, world!\"");

            let s = AgentValue::new_string("hello\nworld\n\n");
            assert_eq!(serde_json::to_string(&s).unwrap(), r#""hello\nworld\n\n""#);
        }

        // Test Array serialization
        {
            let array = AgentValue::new_array(vec![
                AgentValue::new_integer(1),
                AgentValue::new_string("test"),
                AgentValue::new_object(AgentValueMap::from([
                    ("key1".to_string(), AgentValue::new_string("test")),
                    ("key2".to_string(), AgentValue::new_integer(2)),
                ])),
            ]);
            assert_eq!(
                serde_json::to_string(&array).unwrap(),
                r#"[1,"test",{"key1":"test","key2":2}]"#
            );
        }

        // Test Object serialization
        {
            let obj = AgentValue::new_object(AgentValueMap::from([
                ("key1".to_string(), AgentValue::new_string("test")),
                ("key2".to_string(), AgentValue::new_integer(3)),
            ]));
            assert_eq!(
                serde_json::to_string(&obj).unwrap(),
                r#"{"key1":"test","key2":3}"#
            );
        }
    }

    #[test]
    fn test_agent_value_deserialization() {
        // Test Null deserialization
        {
            let deserialized: AgentValue = serde_json::from_str("null").unwrap();
            assert_eq!(deserialized, AgentValue::Null);
        }

        // Test Boolean deserialization
        {
            let deserialized: AgentValue = serde_json::from_str("false").unwrap();
            assert_eq!(deserialized, AgentValue::new_boolean(false));

            let deserialized: AgentValue = serde_json::from_str("true").unwrap();
            assert_eq!(deserialized, AgentValue::new_boolean(true));
        }

        // Test Integer deserialization
        {
            let deserialized: AgentValue = serde_json::from_str("123").unwrap();
            assert_eq!(deserialized, AgentValue::new_integer(123));
        }

        // Test Number deserialization
        {
            let deserialized: AgentValue = serde_json::from_str("3.14").unwrap();
            assert_eq!(deserialized, AgentValue::new_number(3.14));

            let deserialized: AgentValue = serde_json::from_str("3.0").unwrap();
            assert_eq!(deserialized, AgentValue::new_number(3.0));
        }

        // Test String deserialization
        {
            let deserialized: AgentValue = serde_json::from_str("\"Hello, world!\"").unwrap();
            assert_eq!(deserialized, AgentValue::new_string("Hello, world!"));

            let deserialized: AgentValue = serde_json::from_str(r#""hello\nworld\n\n""#).unwrap();
            assert_eq!(deserialized, AgentValue::new_string("hello\nworld\n\n"));
        }

        // Test Array deserialization
        {
            let deserialized: AgentValue =
                serde_json::from_str(r#"[1,"test",{"key1":"test","key2":2}]"#).unwrap();
            assert!(matches!(deserialized, AgentValue::Array(_)));
            if let AgentValue::Array(arr) = deserialized {
                assert_eq!(arr.len(), 3, "Array length mismatch after serialization");
                assert_eq!(arr[0], AgentValue::new_integer(1));
                assert_eq!(arr[1], AgentValue::new_string("test"));
                assert_eq!(
                    arr[2],
                    AgentValue::new_object(AgentValueMap::from([
                        ("key1".to_string(), AgentValue::new_string("test")),
                        ("key2".to_string(), AgentValue::new_integer(2)),
                    ]))
                );
            }
        }

        // Test Object deserialization
        {
            let deserialized: AgentValue =
                serde_json::from_str(r#"{"key1":"test","key2":3}"#).unwrap();
            assert_eq!(
                deserialized,
                AgentValue::new_object(AgentValueMap::from([
                    ("key1".to_string(), AgentValue::new_string("test")),
                    ("key2".to_string(), AgentValue::new_integer(3)),
                ]))
            );
        }
    }
}
