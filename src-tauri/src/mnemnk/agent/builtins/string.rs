use anyhow::{bail, Context as _, Result};
use handlebars::Handlebars;
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::definition::AGENT_KIND_BUILTIN;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentConfigEntry, AgentContext, AgentData, AgentDefinition,
    AgentDefinitions, AgentValue, AsAgent, AsAgentData,
};

// Template String Agent
struct TemplateStringAgent {
    data: AsAgentData,
}

impl AsAgent for TemplateStringAgent {
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
        let config = self.config().context("missing config")?;

        let template = config.get_string_or_default(CONFIG_TEMPLATE);
        if template.is_empty() {
            bail!("template is not set");
        }

        let reg = Handlebars::new();
        let rendered_string = reg.render_template(&template, &data)?;

        let (kind, out_value) = match self.def_name() {
            "$template_string" => ("string", AgentValue::new_string(rendered_string)),
            "$template_text" => ("text", AgentValue::new_text(rendered_string)),
            _ => bail!("Invalid def_name"),
        };

        self.try_output(
            ch,
            AgentData {
                kind: kind.to_string(),
                value: out_value,
            },
        )
        .context("Failed to output template")
    }
}

static CATEGORY: &str = "Core/String";

static CH_STRING: &str = "string";
static CH_TEXT: &str = "text";

static CONFIG_TEMPLATE: &str = "template";

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    // Template String Agent
    defs.insert(
        "$template_string".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$template_string",
            Some(new_boxed::<TemplateStringAgent>),
        )
        .with_title("Template String")
        .with_category(CATEGORY)
        .with_inputs(vec!["*"])
        .with_outputs(vec![CH_STRING])
        .with_default_config(vec![(
            CONFIG_TEMPLATE.into(),
            AgentConfigEntry::new(AgentValue::new_string(""), "string"),
        )]),
    );

    // Template Text Agent
    defs.insert(
        "$template_text".into(),
        AgentDefinition::new(
            // We can use TemplateStringAgent for `$template_text` too,
            // since the only difference is the config type.
            AGENT_KIND_BUILTIN,
            "$template_text",
            Some(new_boxed::<TemplateStringAgent>),
        )
        .with_title("Template Text")
        .with_category(CATEGORY)
        .with_inputs(vec!["*"])
        .with_outputs(vec![CH_TEXT])
        .with_default_config(vec![(
            CONFIG_TEMPLATE.into(),
            AgentConfigEntry::new(AgentValue::new_text(""), "text"),
        )]),
    );
}
