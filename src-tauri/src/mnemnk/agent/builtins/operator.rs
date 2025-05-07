use anyhow::Result;
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::definition::AGENT_KIND_BUILTIN;
use crate::mnemnk::agent::{
    AgentConfig, AgentContext, AgentData, AgentDefinition, AgentDefinitions, AgentOutput, AsAgent,
    AsAgentData,
};

// Latest agent
struct LatestAgent {
    data: AsAgentData,
    latest: Option<(AgentContext, AgentData)>,
}

impl AsAgent for LatestAgent {
    fn new(
        app: AppHandle,
        id: String,
        def_name: String,
        config: Option<AgentConfig>,
    ) -> Result<Self> {
        Ok(Self {
            data: AsAgentData::new(app, id, def_name, config),
            latest: None,
        })
    }

    fn data(&self) -> &AsAgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AsAgentData {
        &mut self.data
    }

    fn stop(&mut self) -> Result<()> {
        self.latest = None;
        Ok(())
    }

    fn process(&mut self, ctx: AgentContext, data: AgentData) -> Result<()> {
        if ctx.ch() == CH_UNIT {
            if let Some(latest) = self.latest.as_ref() {
                let latest_ctx = latest.0.clone();
                let latest_data = latest.1.clone();
                let ch = latest_ctx.ch().to_string();
                self.try_output(latest_ctx, ch, latest_data)?;
            }
            return Ok(());
        }
        self.latest = Some((ctx, data));
        Ok(())
    }
}

// Sample agent
struct SampleAgent {
    data: AsAgentData,
    latest: Option<(AgentContext, AgentData)>,
}

impl AsAgent for SampleAgent {
    fn new(
        app: AppHandle,
        id: String,
        def_name: String,
        config: Option<AgentConfig>,
    ) -> Result<Self> {
        Ok(Self {
            data: AsAgentData::new(app, id, def_name, config),
            latest: None,
        })
    }

    fn data(&self) -> &AsAgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AsAgentData {
        &mut self.data
    }

    fn stop(&mut self) -> Result<()> {
        self.latest = None;
        Ok(())
    }

    fn process(&mut self, ctx: AgentContext, data: AgentData) -> Result<()> {
        if ctx.ch() == CH_UNIT {
            if let Some(latest) = self.latest.take() {
                let ch = latest.0.ch().to_string();
                self.try_output(latest.0, ch, latest.1)?;
            }
            return Ok(());
        }
        self.latest = Some((ctx, data));
        Ok(())
    }
}

static CATEGORY: &str = "Core/Operator";

static CH_UNIT: &str = "unit";

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    defs.insert(
        "$latest".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$latest",
            Some(new_boxed::<LatestAgent>),
        )
        .with_title("Latest")
        .with_category(CATEGORY)
        .with_inputs(vec!["*", CH_UNIT])
        .with_outputs(vec!["*"]),
    );

    defs.insert(
        "$sample".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$sample",
            Some(new_boxed::<SampleAgent>),
        )
        .with_title("Sample")
        .with_category(CATEGORY)
        .with_inputs(vec!["*", CH_UNIT])
        .with_outputs(vec!["*"]),
    );
}
