use anyhow::{Context as _, Result};
use rhai::serde::{from_dynamic, to_dynamic};
use rhai::{Dynamic, Scope};
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::definition::AGENT_KIND_BUILTIN;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentConfigEntry, AgentContext, AgentData, AgentDefinition,
    AgentDefinitions, AgentValue, AsAgent, AsAgentData,
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
        let config = self.config().context("missing config")?;

        let template = config
            .get_string(CONFIG_TEMPLATE)
            .context("missing template")?;
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
            .eval_expression_with_scope(&mut scope, &template)?;

        let out_data: AgentData = from_dynamic(&result)?;

        self.try_output(CH_DATA, out_data)
            .context("Failed to output template")
    }
}

static CH_DATA: &str = "data";
static CONFIG_TEMPLATE: &str = "template";

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    // Template String Agent
    defs.insert(
        "$rhai_expression".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$rhai_expression",
            Some(new_boxed::<RhaiExpressionAgent>),
        )
        .with_title("Rhai Expression")
        .with_category("Core/Script")
        .with_inputs(vec![CH_DATA])
        .with_outputs(vec![CH_DATA])
        .with_default_config(vec![(
            CONFIG_TEMPLATE.into(),
            AgentConfigEntry::new(AgentValue::new_string(""), "text"),
        )]),
    );
}
