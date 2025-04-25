use std::vec;

use anyhow::{Context as _, Result};
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::definition::AGENT_KIND_BUILTIN;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentConfigEntry, AgentContext, AgentData, AgentDefinition,
    AgentDefinitions, AgentDisplayConfigEntry, AgentValue, AsAgent, AsAgentData,
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
        self.emit_display(DISPLAY_COUNT, AgentData::new_integer(0))?;
        Ok(())
    }

    fn process(&mut self, ch: String, _data: AgentData) -> Result<()> {
        if ch == CH_RESET {
            self.count = 0;
        } else if ch == CH_IN {
            self.count += 1;
        }
        self.try_output(CH_COUNT, AgentData::new_integer(self.count))?;
        self.emit_display(DISPLAY_COUNT, AgentData::new_integer(self.count))
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

impl AsAgent for MemoryAgent {
    fn new(
        app: AppHandle,
        id: String,
        def_name: String,
        config: Option<AgentConfig>,
    ) -> Result<Self> {
        Ok(Self {
            data: AsAgentData::new(app, id, def_name, config),
            memory: vec![],
        })
    }

    fn data(&self) -> &AsAgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AsAgentData {
        &mut self.data
    }

    fn process(&mut self, ch: String, data: AgentData) -> Result<()> {
        if ch == CH_RESET {
            // Reset command empties the memory
            self.memory.clear();

            self.try_output(CH_RESET, AgentData::new_unit())?;
        } else if ch == CH_IN {
            // Add new data to memory
            self.memory.push(data.clone());

            // Trim to max size if needed
            if let Some(n) = self.config().context("no config")?.get_integer(CONFIG_N) {
                if n > 0 {
                    let n = n as usize;

                    // If the n is smaller than the current number of data,
                    // trim the oldest data to fit the n
                    if n < self.memory.len() {
                        let data_to_remove = self.memory.len() - n;
                        self.memory.drain(0..data_to_remove);
                    }
                }
            }

            // Output the memory array
            let memory_array =
                AgentValue::new_array(self.memory.iter().map(|data| data.value.clone()).collect());
            self.try_output(
                CH_MEMORY,
                AgentData {
                    kind: self.memory[0].kind.clone(),
                    value: memory_array,
                },
            )?;
        }

        Ok(())
    }
}

static CATEGORY: &str = "Core/Utils";

static CH_IN: &str = "in";
static CH_RESET: &str = "reset";
static CH_COUNT: &str = "count";
static CH_MEMORY: &str = "memory";

static DISPLAY_COUNT: &str = "count";

static CONFIG_N: &str = "n";
const CONFIG_N_DEFAULT: i64 = 10;

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    // Counter Agent
    defs.insert(
        "$counter".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$counter",
            Some(new_boxed::<CounterAgent>),
        )
        .with_title("Counter")
        // .with_description("Display value on the node")
        .with_category(CATEGORY)
        .with_inputs(vec![CH_IN, CH_RESET])
        .with_outputs(vec![CH_COUNT])
        .with_display_config(vec![(
            DISPLAY_COUNT.into(),
            AgentDisplayConfigEntry::new("integer").with_hide_title(),
        )]),
    );

    // Memory Agent
    defs.insert(
        "$memory".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$memory",
            Some(new_boxed::<MemoryAgent>),
        )
        .with_title("Memory")
        .with_description("Stores recent input data")
        .with_category(CATEGORY)
        .with_inputs(vec![CH_IN, CH_RESET])
        .with_outputs(vec![CH_MEMORY, CH_RESET])
        .with_default_config(vec![(
            CONFIG_N.into(),
            AgentConfigEntry::new(AgentValue::new_integer(CONFIG_N_DEFAULT), "integer"),
        )]),
    );
}
