use anyhow::{bail, Context as _, Result};
use handlebars::Handlebars;
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::definition::AGENT_KIND_BUILTIN;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentConfigEntry, AgentContext, AgentData, AgentDefinition,
    AgentDefinitions, AgentValue, AsAgent, AsAgentData,
};

/// The `StringJoinAgent` is responsible for joining an array of strings into a single string
/// using a specified separator. It processes input data, applies transformations to handle
/// escape sequences (e.g., `\n`, `\t`), and outputs the resulting string.
///
/// # Configuration
/// - `CONFIG_SEP`: Specifies the separator to use when joining strings. Defaults to an empty string.
///
/// # Input
/// - Expects an array of strings as input data.
///
/// # Output
/// - Produces a single joined string as output.
///
/// # Example
/// Given the input `["Hello", "World"]` and `CONFIG_SEP` set to `" "`, the output will be `"Hello World"`.
struct StringJoinAgent {
    data: AsAgentData,
}

impl AsAgent for StringJoinAgent {
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

        let sep = config.get_string_or_default(CONFIG_SEP);

        if data.is_array() {
            let mut out = Vec::new();
            for v in data.as_array().context("failed as_array")? {
                out.push(v.as_str().unwrap_or_default());
            }
            let mut out = out.join(&sep);
            out = out.replace("\\n", "\n");
            out = out.replace("\\t", "\t");
            out = out.replace("\\r", "\r");
            out = out.replace("\\\\", "\\");
            let out_data = AgentData::new_string(out);
            self.try_output(CH_STRING, out_data)
                .context("Failed to output")
        } else {
            self.try_output(CH_STRING, data)
                .context("Failed to output template")
        }
    }
}

/// The `TextJoinAgent` is responsible for joining an array of texts into a single text
/// using a specified separator. It processes input data, applies transformations to handle
/// escape sequences (e.g., `\n`, `\t`), and outputs the resulting text.
///
/// # Configuration
/// - `CONFIG_SEP`: Specifies the separator to use when joining texts. Defaults to an empty string.
///
/// # Input
/// - Expects an array of texts as input data.
///
/// # Output
/// - Produces a single joined text as output.
///
/// # Example
/// Given the input `["Hello", "World"]` and `CONFIG_SEP` set to `"\\n"`, the output will be `"Hello\nWorld"`.
struct TextJoinAgent {
    data: AsAgentData,
}

impl AsAgent for TextJoinAgent {
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

        let sep = config.get_string_or_default(CONFIG_SEP);

        if data.is_array() {
            let mut out = Vec::new();
            for v in data.as_array().context("failed as_array")? {
                out.push(v.as_str().unwrap_or_default());
            }
            let mut out = out.join(&sep);
            out = out.replace("\\n", "\n");
            out = out.replace("\\t", "\t");
            out = out.replace("\\r", "\r");
            out = out.replace("\\\\", "\\");
            let out_data = AgentData::new_text(out);
            self.try_output(CH_TEXT, out_data)
                .context("Failed to output")
        } else {
            self.try_output(CH_TEXT, data)
                .context("Failed to output template")
        }
    }
}

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

    fn process(&mut self, _ch: String, data: AgentData) -> Result<()> {
        let config = self.config().context("missing config")?;

        let template = config.get_string_or_default(CONFIG_TEMPLATE);
        if template.is_empty() {
            bail!("template is not set");
        }

        let reg = Handlebars::new();

        if data.is_array() {
            let kind = &data.kind;
            let mut out_arr = Vec::new();
            for v in data.as_array().context("failed as_array")? {
                let d = AgentData {
                    kind: kind.clone(),
                    value: v.clone(),
                };
                let rendered_string = reg.render_template(&template, &d)?;
                out_arr.push(AgentValue::new_string(rendered_string));
            }
            self.try_output(CH_STRING, AgentData::new_array("string", out_arr))
                .context("Failed to output template")
        } else {
            let rendered_string = reg.render_template(&template, &data)?;
            let out_data = AgentData::new_string(rendered_string);
            self.try_output(CH_STRING, out_data)
                .context("Failed to output template")
        }
    }
}

// Template Text Agent
struct TemplateTextAgent {
    data: AsAgentData,
}

impl AsAgent for TemplateTextAgent {
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

        let template = config.get_string_or_default(CONFIG_TEMPLATE);
        if template.is_empty() {
            bail!("template is not set");
        }

        let reg = Handlebars::new();

        if data.is_array() {
            let kind = &data.kind;
            let mut out_arr = Vec::new();
            for v in data.as_array().context("failed as_array")? {
                let d = AgentData {
                    kind: kind.clone(),
                    value: v.clone(),
                };
                let rendered_string = reg.render_template(&template, &d)?;
                out_arr.push(AgentValue::new_text(rendered_string));
            }
            self.try_output(CH_TEXT, AgentData::new_array("text", out_arr))
                .context("Failed to output template")
        } else {
            let rendered_string = reg.render_template(&template, &data)?;
            let out_data = AgentData::new_text(rendered_string);
            self.try_output(CH_TEXT, out_data)
                .context("Failed to output template")
        }
    }
}

static CATEGORY: &str = "Core/String";

static CH_DATA: &str = "data";
static CH_STRING: &str = "string";
static CH_STRINGS: &str = "strings";
static CH_TEXT: &str = "text";
static CH_TEXTS: &str = "texts";

static CONFIG_SEP: &str = "sep";
static CONFIG_TEMPLATE: &str = "template";

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    defs.insert(
        "$text_join".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$text_join",
            Some(new_boxed::<TextJoinAgent>),
        )
        .with_title("Text Join")
        .with_category(CATEGORY)
        .with_inputs(vec![CH_TEXTS])
        .with_outputs(vec![CH_TEXT])
        .with_default_config(vec![(
            CONFIG_SEP.into(),
            AgentConfigEntry::new(AgentValue::new_string("\\n"), "string"),
        )]),
    );

    defs.insert(
        "$string_join".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$string_join",
            Some(new_boxed::<StringJoinAgent>),
        )
        .with_title("String Join")
        .with_category(CATEGORY)
        .with_inputs(vec![CH_STRINGS])
        .with_outputs(vec![CH_STRING])
        .with_default_config(vec![(
            CONFIG_SEP.into(),
            AgentConfigEntry::new(AgentValue::new_string("\\n"), "string"),
        )]),
    );

    defs.insert(
        "$template_string".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$template_string",
            Some(new_boxed::<TemplateStringAgent>),
        )
        .with_title("Template String")
        .with_category(CATEGORY)
        .with_inputs(vec![CH_DATA])
        .with_outputs(vec![CH_STRING])
        .with_default_config(vec![(
            CONFIG_TEMPLATE.into(),
            AgentConfigEntry::new(AgentValue::new_string("{{value}}"), "string"),
        )]),
    );

    defs.insert(
        "$template_text".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$template_text",
            Some(new_boxed::<TemplateTextAgent>),
        )
        .with_title("Template Text")
        .with_category(CATEGORY)
        .with_inputs(vec![CH_DATA])
        .with_outputs(vec![CH_TEXT])
        .with_default_config(vec![(
            CONFIG_TEMPLATE.into(),
            AgentConfigEntry::new(AgentValue::new_text("{{value}}"), "text"),
        )]),
    );
}
