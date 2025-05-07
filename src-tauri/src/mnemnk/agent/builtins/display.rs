use anyhow::Result;
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::definition::AGENT_KIND_BUILTIN;
use crate::mnemnk::agent::{
    AgentConfig, AgentContext, AgentData, AgentDefinition, AgentDefinitions,
    AgentDisplayConfigEntry, AgentOutput, AgentValue, AgentValueMap, AsAgent, AsAgentData,
};

// Display Data
struct DisplayDataAgent {
    data: AsAgentData,
}

impl AsAgent for DisplayDataAgent {
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

    fn process(&mut self, _ctx: AgentContext, data: AgentData) -> Result<()> {
        self.emit_display(DISPLAY_DATA, data)
    }
}

// Debug Data
struct DebugDataAgent {
    data: AsAgentData,
}

impl AsAgent for DebugDataAgent {
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

    fn process(&mut self, ctx: AgentContext, data: AgentData) -> Result<()> {
        let value = AgentValue::new_object(AgentValueMap::from([
            ("kind".to_string(), AgentValue::new_string(data.kind)),
            ("value".to_string(), data.value),
        ]));
        let ctx = AgentValue::from_json_value(serde_json::to_value(&ctx)?)?;
        let debug_data = AgentData::new_object(AgentValueMap::from([
            ("ctx".to_string(), ctx),
            ("data".to_string(), value),
        ]));
        self.emit_display(DISPLAY_DATA, debug_data)
    }
}

static CATEGORY: &str = "Core/Display";

static DISPLAY_DATA: &str = "data";

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    // Display Data
    defs.insert(
        "$display_data".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$display_data",
            Some(new_boxed::<DisplayDataAgent>),
        )
        .with_title("Display Data")
        .with_category(CATEGORY)
        .with_inputs(vec!["data"])
        .with_display_config(vec![(
            DISPLAY_DATA.into(),
            AgentDisplayConfigEntry::new("*").with_hide_title(),
        )]),
    );

    // Debug Data
    defs.insert(
        "$debug_data".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$debug_data",
            Some(new_boxed::<DebugDataAgent>),
        )
        .with_title("Debug Data")
        .with_category(CATEGORY)
        .with_inputs(vec!["*"])
        .with_display_config(vec![(
            DISPLAY_DATA.into(),
            AgentDisplayConfigEntry::new("object").with_hide_title(),
        )]),
    );
}
