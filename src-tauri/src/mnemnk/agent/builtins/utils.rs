use std::vec;

use anyhow::Result;
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentConfigEntry, AgentData, AgentDefinition, AgentDefinitions,
    AgentDisplayConfigEntry, AgentValue, AsAgent, AsAgentData,
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
            data: AsAgentData::new(app, id, def_name, config),
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
        self.emit_display("count".to_string(), AgentData::new_integer(0))?;
        Ok(())
    }

    fn process(&mut self, ch: String, _data: AgentData) -> Result<()> {
        if ch == "reset" {
            self.count = 0;
        } else if ch == "in" {
            self.count += 1;
        }
        self.try_output("count".to_string(), AgentData::new_integer(self.count))?;
        self.emit_display("count".to_string(), AgentData::new_integer(self.count))
    }
}

// Memory Agent
//
// Retains the last `n` of the input data and outputs them.
// The output data `kind` matches that of the first data.
struct MemoryAgent {
    data: AsAgentData,
    memory: Vec<AgentData>,
}

const DEFAULT_N: i64 = 10;

fn get_n(config: &Option<AgentConfig>) -> i64 {
    if let Some(config) = config {
        if let Some(n) = config.get("n") {
            return n.as_i64().unwrap_or(DEFAULT_N);
        }
    }
    DEFAULT_N
}

impl AsAgent for MemoryAgent {
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
            memory: vec![],
        })
    }

    fn data(&self) -> &AsAgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AsAgentData {
        &mut self.data
    }

    fn set_config(&mut self, config: AgentConfig) -> Result<()> {
        // Update n if it has changed
        if let Some(n) = config.get("n") {
            let new_n = n.as_i64().unwrap_or(DEFAULT_N);
            if new_n > 0 {
                let new_n = new_n as usize;

                // If the new n is smaller than the current number of data,
                // trim the oldest data to fit the new n
                if new_n < self.memory.len() {
                    let data_to_remove = self.memory.len() - new_n;
                    self.memory.drain(0..data_to_remove);
                }
            }
        }

        Ok(())
    }

    fn process(&mut self, ch: String, data: AgentData) -> Result<()> {
        if ch == "reset" {
            // Reset command empties the memory
            self.memory.clear();

            self.try_output("reset".to_string(), AgentData::new_unit())?;
        } else if ch == "in" {
            // Add new data to memory
            self.memory.push(data.clone());

            // Trim to max size if needed
            let n = get_n(&self.data().config);
            if n > 0 {
                let n = n as usize;
                // If the memory is larger than n, remove the oldest item
                if self.memory.len() > n {
                    self.memory.remove(0); // Remove oldest item
                }
            }

            // Output the memory array
            let memory_array =
                AgentValue::new_array(self.memory.iter().map(|data| data.value.clone()).collect());
            self.try_output(
                "memory".to_string(),
                AgentData {
                    kind: self.memory[0].kind.clone(),
                    value: memory_array,
                },
            )?;
        }

        Ok(())
    }
}

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    // Counter Agent
    defs.insert(
        "$counter".into(),
        AgentDefinition::new("Builtin", "$counter", Some(new_boxed::<CounterAgent>))
            .with_title("Counter")
            // .with_description("Display value on the node")
            .with_category("Core/Utils")
            .with_inputs(vec!["in", "reset"])
            .with_outputs(vec!["count"])
            .with_display_config(vec![(
                "count".into(),
                AgentDisplayConfigEntry::new("integer").with_hide_title(),
            )]),
    );

    // Memory Agent
    defs.insert(
        "$memory".into(),
        AgentDefinition::new("Memory", "$memory", Some(new_boxed::<MemoryAgent>))
            .with_title("Memory")
            .with_description("Stores recent input data")
            .with_category("Core/Utils")
            .with_inputs(vec!["in", "reset"])
            .with_outputs(vec!["memory", "reset"])
            .with_default_config(vec![(
                "n".into(),
                AgentConfigEntry::new(AgentValue::new_integer(10), "integer"),
            )]),
    );
}
