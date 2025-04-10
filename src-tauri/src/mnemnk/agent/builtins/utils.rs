use std::vec;

use anyhow::Result;
use serde_json::json;
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentData, AgentDefinition, AgentDefinitions, AgentDisplayConfigEntry,
    AsAgent, AsAgentData,
};

// Counter
struct CounterAgent {
    data: AsAgentData,
    count: i64,
}

impl AsAgent for CounterAgent {
    fn new(
        app: AppHandle,
        id: String,
        def_name: String,
        config: Option<AgentConfig>,
    ) -> Result<Self> {
        Ok(Self {
            data: AsAgentData {
                app,
                id,
                status: Default::default(),
                def_name,
                config,
            },
            count: 0,
        })
    }

    fn data(&self) -> &AsAgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AsAgentData {
        &mut self.data
    }

    fn start(&mut self) -> Result<()> {
        self.count = 0;
        self.emit_display("count".to_string(), json!(0))?;
        Ok(())
    }

    fn input(&mut self, ch: String, _data: AgentData) -> Result<()> {
        if ch == "reset" {
            self.count = 0;
        } else if ch == "in" {
            self.count += 1;
        }
        let value = json!(self.count);
        let data = AgentData {
            kind: "integer".to_string(),
            value: value.clone(),
        };
        self.try_output("count".to_string(), data)?;
        self.emit_display("count".to_string(), value)?;
        Ok(())
    }
}

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    // Display Data
    defs.insert(
        "$counter".into(),
        AgentDefinition::new("Counter", "$counter", Some(new_boxed::<CounterAgent>))
            .with_title("Counter")
            // .with_description("Display value on the node")
            .with_category("Core/Utils")
            .with_inputs(vec!["in", "reset"])
            .with_outputs(vec!["count"])
            .with_display_config(vec![(
                "count".into(),
                AgentDisplayConfigEntry::new("integer"),
            )]),
    );
}
