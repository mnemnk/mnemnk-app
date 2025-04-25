use anyhow::{Context as _, Result};
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::definition::AGENT_KIND_BUILTIN;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentConfigEntry, AgentContext, AgentData, AgentDefinition,
    AgentDefinitions, AgentValue, AsAgent, AsAgentData,
};

// As Kind Agent
struct AsKindAgent {
    data: AsAgentData,
}

impl AsAgent for AsKindAgent {
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

    fn process(&mut self, ch: String, data: AgentData) -> Result<()> {
        let kind = self
            .config()
            .context("no config")?
            .get_string_or_default(CONFIG_KIND);
        if kind.is_empty() {
            // kind is not set
            return Ok(());
        }

        self.try_output(
            ch,
            AgentData {
                kind,
                value: data.value,
            },
        )
        .context("Failed to output")?;

        Ok(())
    }
}

static CONFIG_KIND: &str = "kind";

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    // AsKindAgent
    defs.insert(
        "$as_kind_filter".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$as_kind",
            Some(new_boxed::<AsKindAgent>),
        )
        .with_title("As Kind")
        .with_category("Core")
        .with_inputs(vec!["*"])
        .with_outputs(vec!["*"])
        .with_default_config(vec![(
            CONFIG_KIND.into(),
            AgentConfigEntry::new(AgentValue::new_string(""), "string").with_title("Kind"),
        )]),
    );
}
