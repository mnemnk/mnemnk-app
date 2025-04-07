use anyhow::{Context as _, Result};
use serde_json::{json, Value};
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentConfigEntry, AgentDefinition, AgentDefinitions, AsAgent, AsAgentData,
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
            data: AsAgentData {
                app,
                id,
                status: Default::default(),
                def_name,
                config,
            },
        })
    }

    fn data(&self) -> &AsAgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AsAgentData {
        &mut self.data
    }

    fn input(&mut self, _kind: String, value: Value) -> Result<()> {
        let kind = self
            .data
            .config
            .as_ref()
            .context("Missing config")?
            .get("kind")
            .context("Missing kind")?
            .as_str()
            .context("kind is not a string")?;
        if kind.is_empty() {
            // kind is not set
            return Ok(());
        }
        self.try_output(kind.to_string(), value.clone())
            .context("Failed to output")?;
        Ok(())
    }
}

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    // AsKindAgent
    defs.insert(
        "$as_kind_filter".into(),
        AgentDefinition::new("AsKind", "$as_kind", Some(new_boxed::<AsKindAgent>))
            .with_title("As Kind")
            .with_category("Core")
            .with_inputs(vec!["*"])
            .with_outputs(vec!["*"])
            .with_default_config(vec![(
                "kind".into(),
                AgentConfigEntry::new(json!(""), "string").with_title("Kind"),
            )]),
    );
}
