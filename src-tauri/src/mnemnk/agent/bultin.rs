use std::collections::HashMap;

use serde_json::Value;

use super::definition::{AgentDefaultConfig, AgentDefinition, AgentDefinitions};

pub fn builtin_agent_defs() -> AgentDefinitions {
    let mut defs: AgentDefinitions = Default::default();

    defs.insert(
        "$board".to_string(),
        AgentDefinition {
            name: "$board".to_string(),
            title: Some("Board".to_string()),
            description: None,
            path: None,
            inputs: Some(vec!["*".to_string()]),
            outputs: Some(vec!["*".to_string()]),
            default_config: Some(HashMap::from([(
                "board_name".to_string(),
                AgentDefaultConfig {
                    value: Value::String("".to_string()),
                    type_: Some("string?".to_string()),
                    title: Some("Board Name".to_string()),
                    description: None,
                    scope: None,
                },
            )])),
        },
    );

    defs.insert(
        "$database".to_string(),
        AgentDefinition {
            name: "$database".to_string(),
            title: Some("Database".to_string()),
            description: None,
            path: None,
            inputs: Some(vec!["*".to_string()]),
            outputs: None,
            default_config: None,
        },
    );

    defs
}
