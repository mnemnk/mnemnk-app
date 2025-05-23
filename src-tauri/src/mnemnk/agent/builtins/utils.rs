use std::vec;

use anyhow::Result;
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::definition::AGENT_KIND_BUILTIN;
use crate::mnemnk::agent::{
    AgentConfig, AgentContext, AgentData, AgentDefinition, AgentDefinitions,
    AgentDisplayConfigEntry, AgentOutput, AsAgent, AsAgentData,
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

    fn process(&mut self, ctx: AgentContext, _data: AgentData) -> Result<()> {
        let ch = ctx.ch();
        if ch == CH_RESET {
            self.count = 0;
        } else if ch == CH_IN {
            self.count += 1;
        }
        self.try_output(ctx, CH_COUNT, AgentData::new_integer(self.count))?;
        self.emit_display(DISPLAY_COUNT, AgentData::new_integer(self.count))
    }
}

static CATEGORY: &str = "Core/Utils";

static CH_IN: &str = "in";
static CH_RESET: &str = "reset";
static CH_COUNT: &str = "count";

static DISPLAY_COUNT: &str = "count";

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
}
