use anyhow::{anyhow, bail, Context as _, Result};
use rhai::{Dynamic, Scope, AST};
use tauri::{AppHandle, Manager};

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::definition::AGENT_KIND_BUILTIN;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentConfigEntry, AgentContext, AgentData, AgentDefinition,
    AgentDefinitions, AgentEnv, AgentValue, AsAgent, AsAgentData,
};

// Rhai Expr Agent
struct RhaiExprAgent {
    data: AsAgentData,
    ast: Option<AST>,
}

impl RhaiExprAgent {
    fn compile_expr(env: &AgentEnv, config: &AgentConfig) -> Result<Option<AST>> {
        let Some(expr) = config.get_string(CONFIG_EXPR).map(|s| s.trim().to_string()) else {
            return Ok(None);
        };
        if expr.is_empty() {
            return Ok(None);
        }
        let ast = env
            .rhai_engine
            .compile_expression(expr)
            .context("Failed to compile Rhai expression")?;
        Ok(Some(ast))
    }

    fn to_dynamic(data: &AgentData) -> Result<Dynamic> {
        Self::from_kind_value_to_dynamic(&data.kind, &data.value)
    }

    fn from_kind_value_to_dynamic(kind: &str, value: &AgentValue) -> Result<Dynamic> {
        if let Some(arr) = value.as_array() {
            let mut rhai_array: Vec<Dynamic> = Vec::with_capacity(arr.len());
            for v in arr.iter() {
                let d = Self::from_kind_value_to_dynamic(kind, v)?;
                rhai_array.push(d);
            }
            return Ok(rhai_array.into());
        }

        let rhai_value = match kind {
            "unit" => ().into(),
            "boolean" => value.as_bool().context("wrong boolean value")?.into(),
            "integer" => value.as_i64().context("wrong integer value")?.into(),
            "number" => value.as_f64().context("wrong number value")?.into(),
            "string" => value.as_str().context("wrong string value")?.into(),
            "text" => value.as_str().context("wrong text value")?.into(),
            _ => {
                let obj = value.as_object().context("wrong object value")?;
                rhai::serde::to_dynamic(obj)?
            }
        };
        Ok(rhai_value)
    }

    fn from_dynamic(data: &Dynamic) -> Result<AgentData> {
        if data.is_unit() {
            return Ok(AgentData::new_unit());
        }
        if data.is_bool() {
            let value = data
                .as_bool()
                .map_err(|e| anyhow!("Failed as_bool: {}", e))?;
            return Ok(AgentData::new_boolean(value));
        }
        if data.is_int() {
            let value = data.as_int().map_err(|e| anyhow!("Failed as_int: {}", e))?;
            return Ok(AgentData::new_integer(value));
        }
        if data.is_float() {
            let value = data
                .as_float()
                .map_err(|e| anyhow!("Failed as_float: {}", e))?;
            return Ok(AgentData::new_number(value));
        }
        if data.is_string() {
            let value = data
                .clone()
                .into_string()
                .map_err(|e| anyhow!("Failed into_string: {}", e))?;
            return Ok(AgentData::new_string(value));
        }
        if data.is_map() {
            let value: serde_json::Value = rhai::serde::from_dynamic(data)?;
            return Ok(AgentData::new_object(value));
        }
        if data.is_array() {
            let arr = data
                .as_array_ref()
                .map_err(|e| anyhow!("Failed as_array_ref: {}", e))?;
            let mut value_array: Vec<AgentValue> = Vec::with_capacity(arr.len());
            for v in arr.iter() {
                let d = Self::from_dynamic_to_value(v)?;
                value_array.push(d);
            }
            let kind = if value_array.is_empty() {
                "object".to_string() // for now
            } else {
                value_array[0].kind()
            };
            return Ok(AgentData::new_array(kind, value_array));
        }

        bail!("Unsupported Rhai data type: {}", data.type_name());
    }

    fn from_dynamic_to_value(data: &Dynamic) -> Result<AgentValue> {
        if data.is_unit() {
            return Ok(AgentValue::new_unit());
        }
        if data.is_bool() {
            let value = data
                .as_bool()
                .map_err(|e| anyhow!("Failed as_bool: {}", e))?;
            return Ok(AgentValue::new_boolean(value));
        }
        if data.is_int() {
            let value = data.as_int().map_err(|e| anyhow!("Failed as_int: {}", e))?;
            return Ok(AgentValue::new_integer(value));
        }
        if data.is_float() {
            let value = data
                .as_float()
                .map_err(|e| anyhow!("Failed as_float: {}", e))?;
            return Ok(AgentValue::new_number(value));
        }
        if data.is_string() {
            let value = data
                .clone()
                .into_string()
                .map_err(|e| anyhow!("Failed into_string: {}", e))?;
            return Ok(AgentValue::new_string(value));
        }
        if data.is_map() {
            let value: serde_json::Value = rhai::serde::from_dynamic(data)?;
            return Ok(AgentValue::new_object(value));
        }
        if data.is_array() {
            let arr = data
                .as_array_ref()
                .map_err(|e| anyhow!("Failed as_array_ref: {}", e))?;
            let mut value_array: Vec<AgentValue> = Vec::with_capacity(arr.len());
            for v in arr.iter() {
                let d = Self::from_dynamic_to_value(v)?;
                value_array.push(d);
            }
            return Ok(AgentValue::new_array(value_array));
        }

        bail!("Unsupported Rhai data type: {}", data.type_name());
    }
}

impl AsAgent for RhaiExprAgent {
    fn new(
        app: AppHandle,
        id: String,
        def_name: String,
        config: Option<AgentConfig>,
    ) -> Result<Self> {
        let env = app.state::<AgentEnv>();
        let ast = match &config {
            Some(c) => Self::compile_expr(&env, c)?,
            None => None,
        };
        Ok(Self {
            data: AsAgentData::new(app, id, def_name, config),
            ast,
        })
    }

    fn data(&self) -> &AsAgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AsAgentData {
        &mut self.data
    }

    fn set_config(&mut self, config: AgentConfig) -> Result<()> {
        self.ast = Self::compile_expr(&self.env(), &config)?;
        Ok(())
    }

    fn process(&mut self, ch: String, data: AgentData) -> Result<()> {
        let Some(ast) = &self.ast else {
            return Ok(());
        };

        let mut scope = Scope::new();
        scope.push("ch", ch);

        let rhai_value: Dynamic = Self::to_dynamic(&data)?;
        scope.push("kind", data.kind);
        scope.push("value", rhai_value);

        let env = self.env();
        let result: Dynamic = env
            .rhai_engine
            .eval_ast_with_scope(&mut scope, ast)
            .context("Failed to evaluate Rhai expression")?;

        let out_data: AgentData = Self::from_dynamic(&result)?;

        self.try_output(CH_DATA, out_data)
            .context("Failed to output template")
    }
}

static CH_STAR: &str = "*";
static CH_DATA: &str = "data";
static CONFIG_EXPR: &str = "expr";

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    defs.insert(
        "$rhai_expr".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$rhai_expr",
            Some(new_boxed::<RhaiExprAgent>),
        )
        .with_title("Rhai Expr")
        .with_category("Core/Script")
        .with_inputs(vec![CH_STAR])
        .with_outputs(vec![CH_DATA])
        .with_default_config(vec![(
            CONFIG_EXPR.into(),
            AgentConfigEntry::new(AgentValue::new_text(""), "text"),
        )]),
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_to_dynamic() {
        let data = AgentData::new_unit();
        let dynamic = RhaiExprAgent::to_dynamic(&data).unwrap();
        assert!(dynamic.is_unit());

        let data = AgentData::new_boolean(true);
        let dynamic = RhaiExprAgent::to_dynamic(&data).unwrap();
        assert!(dynamic.is_bool());
        assert_eq!(dynamic.as_bool().unwrap(), true);

        let data = AgentData::new_integer(42);
        let dynamic = RhaiExprAgent::to_dynamic(&data).unwrap();
        assert!(dynamic.is_int());
        assert_eq!(dynamic.as_int().unwrap(), 42);

        let data = AgentData::new_number(3.14);
        let dynamic = RhaiExprAgent::to_dynamic(&data).unwrap();
        assert!(dynamic.is_float());
        assert_eq!(dynamic.as_float().unwrap(), 3.14);

        let data = AgentData::new_string("Hello");
        let dynamic = RhaiExprAgent::to_dynamic(&data).unwrap();
        assert!(dynamic.is_string());
        assert_eq!(dynamic.into_string().unwrap(), "Hello");

        let data = AgentData::new_text("Hello\nWorld!\n\n");
        let dynamic = RhaiExprAgent::to_dynamic(&data).unwrap();
        assert!(dynamic.is_string());
        assert_eq!(dynamic.into_string().unwrap(), "Hello\nWorld!\n\n");

        let data = AgentData::new_object(json!({
            "key1": "value1",
            "key2": 42,
        }));
        let dynamic = RhaiExprAgent::to_dynamic(&data).unwrap();
        assert!(dynamic.is_map());
        let map = dynamic.as_map_ref().unwrap();
        assert_eq!(map.len(), 2);
        assert_eq!(
            map.get("key1").unwrap().clone().into_string().unwrap(),
            "value1"
        );
        assert_eq!(map.get("key2").unwrap().as_int().unwrap(), 42);

        let data = AgentData::new_custom_object(
            "custom",
            json!({
                "key1": "value1",
                "key2": 42,
            }),
        );
        let dynamic = RhaiExprAgent::to_dynamic(&data).unwrap();
        assert!(dynamic.is_map());
        let map = dynamic.as_map_ref().unwrap();
        assert_eq!(map.len(), 2);
        assert_eq!(
            map.get("key1").unwrap().clone().into_string().unwrap(),
            "value1"
        );
        assert_eq!(map.get("key2").unwrap().as_int().unwrap(), 42);

        // test array
        let data =
            AgentData::new_array("unit", vec![AgentValue::new_unit(), AgentValue::new_unit()]);
        let dynamic = RhaiExprAgent::to_dynamic(&data).unwrap();
        assert!(dynamic.is_array());
        let arr = dynamic.as_array_ref().unwrap();
        assert_eq!(arr.len(), 2);
        assert!(arr[0].is_unit());
        assert!(arr[1].is_unit());

        let data = AgentData::new_array(
            "boolean",
            vec![
                AgentValue::new_boolean(true),
                AgentValue::new_boolean(false),
            ],
        );
        let dynamic = RhaiExprAgent::to_dynamic(&data).unwrap();
        assert!(dynamic.is_array());
        let arr = dynamic.as_array_ref().unwrap();
        assert_eq!(arr.len(), 2);
        assert!(arr[0].is_bool());
        assert_eq!(arr[0].as_bool().unwrap(), true);
        assert!(arr[1].is_bool());
        assert_eq!(arr[1].as_bool().unwrap(), false);

        let data = AgentData::new_array(
            "integer",
            vec![
                AgentValue::new_integer(1),
                AgentValue::new_integer(2),
                AgentValue::new_integer(3),
            ],
        );
        let dynamic = RhaiExprAgent::to_dynamic(&data).unwrap();
        assert!(dynamic.is_array());
        let arr = dynamic.as_array_ref().unwrap();
        assert_eq!(arr.len(), 3);
        assert!(arr[0].is_int());
        assert_eq!(arr[0].as_int().unwrap(), 1);
        assert!(arr[1].is_int());
        assert_eq!(arr[1].as_int().unwrap(), 2);
        assert!(arr[2].is_int());
        assert_eq!(arr[2].as_int().unwrap(), 3);

        let data = AgentData::new_array(
            "number",
            vec![
                AgentValue::new_number(1.0),
                AgentValue::new_number(2.1),
                AgentValue::new_number(3.2),
            ],
        );
        let dynamic = RhaiExprAgent::to_dynamic(&data).unwrap();
        assert!(dynamic.is_array());
        let arr = dynamic.as_array_ref().unwrap();
        assert_eq!(arr.len(), 3);
        assert!(arr[0].is_float());
        assert_eq!(arr[0].as_float().unwrap(), 1.0);
        assert!(arr[1].is_float());
        assert_eq!(arr[1].as_float().unwrap(), 2.1);
        assert!(arr[2].is_float());
        assert_eq!(arr[2].as_float().unwrap(), 3.2);

        let data = AgentData::new_array(
            "string",
            vec![
                AgentValue::new_string("s1"),
                AgentValue::new_string("s2\ns3\n\n"),
                AgentValue::new_string(""),
            ],
        );
        let dynamic = RhaiExprAgent::to_dynamic(&data).unwrap();
        assert!(dynamic.is_array());
        let arr = dynamic.as_array_ref().unwrap();
        assert_eq!(arr.len(), 3);
        assert!(arr[0].is_string());
        assert_eq!(arr[0].clone().into_string().unwrap(), "s1");
        assert!(arr[1].is_string());
        assert_eq!(arr[1].clone().into_string().unwrap(), "s2\ns3\n\n");
        assert!(arr[2].is_string());
        assert_eq!(arr[2].clone().into_string().unwrap(), "");

        let data = AgentData::new_array(
            "text",
            vec![
                AgentValue::new_text("s1"),
                AgentValue::new_text("s2\ns3\n\n"),
                AgentValue::new_text(""),
            ],
        );
        let dynamic = RhaiExprAgent::to_dynamic(&data).unwrap();
        assert!(dynamic.is_array());
        let arr = dynamic.as_array_ref().unwrap();
        assert_eq!(arr.len(), 3);
        assert!(arr[0].is_string());
        assert_eq!(arr[0].clone().into_string().unwrap(), "s1");
        assert!(arr[1].is_string());
        assert_eq!(arr[1].clone().into_string().unwrap(), "s2\ns3\n\n");
        assert!(arr[2].is_string());
        assert_eq!(arr[2].clone().into_string().unwrap(), "");

        // test object array
        let data = AgentData::new_array(
            "object",
            vec![
                AgentValue::new_object(json!({
                    "key1": "value1",
                    "key2": 2,
                })),
                AgentValue::new_object(json!({
                    "key3": "value3",
                    "key4": {
                        "key4_1": "value4_1",
                        "key4_2": 42,
                    },
                })),
                AgentValue::new_object(json!({})),
            ],
        );
        let dynamic = RhaiExprAgent::to_dynamic(&data).unwrap();
        assert!(dynamic.is_array());
        let arr = dynamic.as_array_ref().unwrap();
        assert_eq!(arr.len(), 3);

        assert!(arr[0].is_map());
        let map0 = arr[0].as_map_ref().unwrap();
        assert_eq!(map0.len(), 2);
        assert_eq!(
            map0.get("key1").unwrap().clone().into_string().unwrap(),
            "value1"
        );
        assert_eq!(map0.get("key2").unwrap().as_int().unwrap(), 2);

        assert!(arr[1].is_map());
        let map1 = arr[1].as_map_ref().unwrap();
        assert_eq!(map1.len(), 2);
        assert_eq!(
            map1.get("key3").unwrap().clone().into_string().unwrap(),
            "value3"
        );
        let map1_1 = map1.get("key4").unwrap().as_map_ref().unwrap();
        assert_eq!(map1_1.len(), 2);
        assert_eq!(
            map1_1.get("key4_1").unwrap().clone().into_string().unwrap(),
            "value4_1"
        );
        assert_eq!(map1_1.get("key4_2").unwrap().as_int().unwrap(), 42);

        assert!(arr[2].is_map());
        let map2 = arr[2].as_map_ref().unwrap();
        assert_eq!(map2.len(), 0);

        // test custom object array
        let data = AgentData::new_array(
            "custom",
            vec![
                AgentValue::new_object(json!({
                    "key1": "value1",
                    "key2": 2,
                })),
                AgentValue::new_object(json!({
                    "key3": "value3",
                    "key4": {
                        "key4_1": "value4_1",
                        "key4_2": 42,
                    },
                })),
                AgentValue::new_object(json!({})),
            ],
        );
        let dynamic = RhaiExprAgent::to_dynamic(&data).unwrap();
        assert!(dynamic.is_array());
        let arr = dynamic.as_array_ref().unwrap();
        assert_eq!(arr.len(), 3);

        assert!(arr[0].is_map());
        let map0 = arr[0].as_map_ref().unwrap();
        assert_eq!(map0.len(), 2);
        assert_eq!(
            map0.get("key1").unwrap().clone().into_string().unwrap(),
            "value1"
        );
        assert_eq!(map0.get("key2").unwrap().as_int().unwrap(), 2);

        assert!(arr[1].is_map());
        let map1 = arr[1].as_map_ref().unwrap();
        assert_eq!(map1.len(), 2);
        assert_eq!(
            map1.get("key3").unwrap().clone().into_string().unwrap(),
            "value3"
        );
        let map1_1 = map1.get("key4").unwrap().as_map_ref().unwrap();
        assert_eq!(map1_1.len(), 2);
        assert_eq!(
            map1_1.get("key4_1").unwrap().clone().into_string().unwrap(),
            "value4_1"
        );
        assert_eq!(map1_1.get("key4_2").unwrap().as_int().unwrap(), 42);

        assert!(arr[2].is_map());
        let map2 = arr[2].as_map_ref().unwrap();
        assert_eq!(map2.len(), 0);
    }

    #[test]
    fn test_from_dynamic() {
        let dynamic = Dynamic::from(());
        let data = RhaiExprAgent::from_dynamic(&dynamic).unwrap();
        assert_eq!(data.kind, "unit");

        let dynamic = Dynamic::from(true);
        let data = RhaiExprAgent::from_dynamic(&dynamic).unwrap();
        assert_eq!(data.kind, "boolean");
        assert_eq!(data.value.as_bool().unwrap(), true);

        let dynamic = Dynamic::from(1_i64);
        let data = RhaiExprAgent::from_dynamic(&dynamic).unwrap();
        assert_eq!(data.kind, "integer");
        assert_eq!(data.value.as_i64().unwrap(), 1);

        let dynamic = Dynamic::from(3.14_f64);
        let data = RhaiExprAgent::from_dynamic(&dynamic).unwrap();
        assert_eq!(data.kind, "number");
        assert_eq!(data.value.as_f64().unwrap(), 3.14);

        let dynamic = Dynamic::from("hello");
        let data = RhaiExprAgent::from_dynamic(&dynamic).unwrap();
        assert_eq!(data.kind, "string");
        assert_eq!(data.value.as_str().unwrap(), "hello");

        let dynamic = rhai::serde::to_dynamic(&json!({
            "key1": "value1",
            "key2": 42,
        }))
        .unwrap();
        let data = RhaiExprAgent::from_dynamic(&dynamic).unwrap();
        assert_eq!(data.kind, "object");
        let obj = data.value.as_object().unwrap();
        assert_eq!(obj["key1"].as_str().unwrap(), "value1");
        assert_eq!(obj["key2"].as_i64().unwrap(), 42);

        // test array
        let dynamic = Dynamic::from(vec![Dynamic::from(()), Dynamic::from(())]);
        let data = RhaiExprAgent::from_dynamic(&dynamic).unwrap();
        assert_eq!(data.kind, "unit");
        let arr = data.value.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert!(arr[0].is_unit());
        assert!(arr[1].is_unit());

        let dynamic = Dynamic::from(vec![Dynamic::from(true), Dynamic::from(false)]);
        let data = RhaiExprAgent::from_dynamic(&dynamic).unwrap();
        assert_eq!(data.kind, "boolean");
        let arr = data.value.as_array().unwrap();
        assert_eq!(arr.len(), 2);
        assert_eq!(arr[0].as_bool().unwrap(), true);
        assert_eq!(arr[1].as_bool().unwrap(), false);

        let dynamic = Dynamic::from(vec![
            Dynamic::from(1_i64),
            Dynamic::from(2_i64),
            Dynamic::from(3_i64),
        ]);
        let data = RhaiExprAgent::from_dynamic(&dynamic).unwrap();
        assert_eq!(data.kind, "integer");
        let arr = data.value.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_i64().unwrap(), 1);
        assert_eq!(arr[1].as_i64().unwrap(), 2);
        assert_eq!(arr[2].as_i64().unwrap(), 3);

        let dynamic = Dynamic::from(vec![
            Dynamic::from(1.0_f64),
            Dynamic::from(2.1_f64),
            Dynamic::from(3.2_f64),
        ]);
        let data = RhaiExprAgent::from_dynamic(&dynamic).unwrap();
        assert_eq!(data.kind, "number");
        let arr = data.value.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_f64().unwrap(), 1.0);
        assert_eq!(arr[1].as_f64().unwrap(), 2.1);
        assert_eq!(arr[2].as_f64().unwrap(), 3.2);

        let dynamic = Dynamic::from(vec![
            Dynamic::from("s1"),
            Dynamic::from("s2\ns3\n\n"),
            Dynamic::from(""),
        ]);
        let data = RhaiExprAgent::from_dynamic(&dynamic).unwrap();
        assert_eq!(data.kind, "string");
        let arr = data.value.as_array().unwrap();
        assert_eq!(arr.len(), 3);
        assert_eq!(arr[0].as_str().unwrap(), "s1");
        assert_eq!(arr[1].as_str().unwrap(), "s2\ns3\n\n");
        assert_eq!(arr[2].as_str().unwrap(), "");

        let dynamic = rhai::serde::to_dynamic(&json!(
            [
                {
                    "key1": "value1",
                    "key2": 2,
                },
                {
                    "key3": "value3",
                    "key4": {
                        "key4_1": "value4_1",
                        "key4_2": 42,
                    },
                },
                {}
            ]
        ))
        .unwrap();
        let data = RhaiExprAgent::from_dynamic(&dynamic).unwrap();
        assert_eq!(data.kind, "object");
        let arr = data.value.as_array().unwrap();
        assert_eq!(arr.len(), 3);

        let obj = arr[0].as_object().unwrap();
        assert_eq!(obj["key1"].as_str().unwrap(), "value1");
        assert_eq!(obj["key2"].as_i64().unwrap(), 2);

        let obj = arr[1].as_object().unwrap();
        assert_eq!(obj["key3"].as_str().unwrap(), "value3");
        let obj4 = obj["key4"].as_object().unwrap();
        assert_eq!(obj4["key4_1"].as_str().unwrap(), "value4_1");
        assert_eq!(obj4["key4_2"].as_i64().unwrap(), 42);

        let obj = arr[2].as_object().unwrap();
        assert_eq!(obj.to_string(), "{}");
    }
}
