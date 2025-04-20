use anyhow::{Context as _, Result};
use rhai::serde::{from_dynamic, to_dynamic};
use rhai::{Dynamic, Scope};
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentConfigEntry, AgentData, AgentDefinition, AgentDefinitions, AgentValue,
    AsAgent, AsAgentData,
};

// Rhai Expression Agent
struct RhaiExpressionAgent {
    data: AsAgentData,
}

impl AsAgent for RhaiExpressionAgent {
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

    fn process(&mut self, _ch: String, data: AgentData) -> Result<()> {
        let config = self.data.config.as_ref().context("Missing config")?;

        let template = config
            .get("template")
            .context("Missing template")?
            .as_str()
            .context("template is not a string")?;
        if template.is_empty() {
            // template is not set
            return Ok(());
        }

        let mut scope = Scope::new();
        let rhai_data: Dynamic = to_dynamic(data)?;
        scope.push("data", rhai_data);

        let env = self.env();
        let result: Dynamic = env
            .rhai_engine
            .eval_expression_with_scope(&mut scope, template)?;

        let out_data: AgentData = from_dynamic(&result)?;

        self.try_output("data".to_string(), out_data)
            .context("Failed to output template")
    }
}

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    // Template String Agent
    defs.insert(
        "$rhai_expression".into(),
        AgentDefinition::new(
            "Builtin",
            "$rhai_expression",
            Some(new_boxed::<RhaiExpressionAgent>),
        )
        .with_title("Rhai Expression")
        .with_category("Core/Script")
        .with_inputs(vec!["data"])
        .with_outputs(vec!["data"])
        .with_default_config(vec![(
            "template".into(),
            AgentConfigEntry::new(AgentValue::new_string("".to_string()), "text"),
        )]),
    );
}
