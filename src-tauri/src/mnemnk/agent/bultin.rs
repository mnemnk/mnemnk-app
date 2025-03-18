use std::collections::HashMap;

use serde_json::Value;

use super::config::{AgentConfig, AgentConfigs, AgentDefaultConfigEntry};

pub fn builtin_agent_configs() -> AgentConfigs {
    let mut configs = HashMap::new();

    configs.insert(
        "$board".to_string(),
        AgentConfig {
            name: "$board".to_string(),
            title: Some("Board".to_string()),
            description: None,
            path: None,
            inputs: Some(vec!["*".to_string()]),
            outputs: Some(vec!["*".to_string()]),
            default_config: Some(HashMap::from([(
                "board_name".to_string(),
                AgentDefaultConfigEntry {
                    value: Value::String("".to_string()),
                    type_: Some("string?".to_string()),
                    title: Some("Board Name".to_string()),
                    description: None,
                    scope: None,
                },
            )])),
        },
    );

    configs.insert(
        "$database".to_string(),
        AgentConfig {
            name: "$database".to_string(),
            title: Some("Database".to_string()),
            description: None,
            path: None,
            inputs: Some(vec!["*".to_string()]),
            outputs: None,
            default_config: None,
        },
    );

    configs
}
