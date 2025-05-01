use anyhow::{Context as _, Result};
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::definition::AGENT_KIND_BUILTIN;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentConfigEntry, AgentContext, AgentData, AgentDefinition,
    AgentDefinitions, AgentDisplayConfigEntry, AgentValue, AsAgent, AsAgentData,
};

// Unit Pass agent
struct UnitPassAgent {
    data: AsAgentData,
}

impl AsAgent for UnitPassAgent {
    fn new(
        app: AppHandle,
        id: String,
        def_name: String,
        config: Option<AgentConfig>,
    ) -> Result<Self> {
        Ok(Self {
            data: AsAgentData::new(app, id, def_name, config),
        })
    }

    fn data(&self) -> &AsAgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AsAgentData {
        &mut self.data
    }

    fn process(&mut self, _ch: String, data: AgentData) -> Result<()> {
        let is_unit = data.is_unit();
        if is_unit {
            self.try_output(CH_UNIT, data)?;
        }
        Ok(())
    }
}

static CATEGORY: &str = "Core/Filter";

static CH_DATA: &str = "data";
static CH_UNIT: &str = "unit";

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    defs.insert(
        "$unit_pass".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$unit_pass",
            Some(new_boxed::<UnitPassAgent>),
        )
        .with_title("Unit Pass")
        .with_category(CATEGORY)
        .with_inputs(vec![CH_DATA])
        .with_outputs(vec![CH_UNIT]),
    );
}
