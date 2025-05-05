use std::sync::Arc;

use anyhow::{Context as _, Result};
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::definition::AGENT_KIND_BUILTIN;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentConfigEntry, AgentContext, AgentData, AgentDefinition,
    AgentDefinitions, AgentValue, AgentValueMap, AsAgent, AsAgentData,
};

// Stream agent
struct StreamAgent {
    data: AsAgentData,
    last_id: i64,
}

impl AsAgent for StreamAgent {
    fn new(
        app: AppHandle,
        id: String,
        def_name: String,
        config: Option<AgentConfig>,
    ) -> Result<Self> {
        Ok(Self {
            data: AsAgentData::new(app, id, def_name, config),
            last_id: 0,
        })
    }

    fn data(&self) -> &AsAgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AsAgentData {
        &mut self.data
    }

    fn process(&mut self, _ch: String, data: AgentData) -> Result<()> {
        let stream = self
            .config()
            .context("missing config")?
            .get(CONFIG_STREAM)
            .context("missing name")?
            .as_str()
            .context("failed as_str")?
            .to_string();
        if stream.is_empty() {
            self.try_output(CH_DATA, data).context("Failed to output")?;
            return Ok(());
        }

        // TODO: add workflow name

        self.last_id += 1;
        let key = format!("$stream:{}", stream);
        let mut new_meta = data.metadata.as_ref().clone();
        new_meta.insert(key, AgentValue::new_integer(self.last_id));

        let out_data = data.clone().with_meta(Arc::new(new_meta));

        self.try_output(CH_DATA, out_data)
            .context("Failed to output")?;

        Ok(())
    }
}

// Stream Zip agent
struct StreamZipAgent {
    data: AsAgentData,
    current_id: i64,
    in1: Option<AgentData>,
    in2: Option<AgentData>,
}

impl AsAgent for StreamZipAgent {
    fn new(
        app: AppHandle,
        id: String,
        def_name: String,
        config: Option<AgentConfig>,
    ) -> Result<Self> {
        Ok(Self {
            data: AsAgentData::new(app, id, def_name, config),
            current_id: -1,
            in1: None,
            in2: None,
        })
    }

    fn data(&self) -> &AsAgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AsAgentData {
        &mut self.data
    }

    fn process(&mut self, ch: String, data: AgentData) -> Result<()> {
        let stream = self
            .config()
            .context("missing config")?
            .get(CONFIG_STREAM)
            .context("missing name")?
            .as_str()
            .context("failed as_str")?
            .to_string();

        if !stream.is_empty() {
            let key = format!("$stream:{}", stream);
            let Some(value) = data.metadata.get(key.as_str()) else {
                // value does not have the stream key
                return Ok(());
            };
            let Some(stream_id) = value.as_i64() else {
                // value is not a number
                return Ok(());
            };
            if stream_id != self.current_id {
                self.current_id = stream_id;
                self.in1 = None;
                self.in2 = None;
            }
        }

        if ch == CH_IN1 {
            self.in1 = Some(data.clone());
        } else if ch == CH_IN2 {
            self.in2 = Some(data.clone());
        } else {
            return Ok(());
        }

        if self.in1.is_none() || self.in2.is_none() {
            return Ok(());
        }

        let in1 = self.in1.take().unwrap();
        let in2 = self.in2.take().unwrap();

        let key1 = self
            .config()
            .context("missing config")?
            .get(CONFIG_KEY1)
            .context("missing key1")?
            .as_str()
            .context("failed as_str")?
            .to_string();

        let key2 = self
            .config()
            .context("missing config")?
            .get(CONFIG_KEY2)
            .context("missing key2")?
            .as_str()
            .context("failed as_str")?
            .to_string();

        if key1.is_empty() || key2.is_empty() {
            return Ok(());
        }

        let out_data =
            AgentData::new_object(AgentValueMap::from([(key1, in1.value), (key2, in2.value)]))
                .from_meta(&data.metadata);

        self.try_output(CH_DATA, out_data)
            .context("Failed to output")?;

        Ok(())
    }
}

static CATEGORY: &str = "Core/Stream";

static CH_DATA: &str = "data";
static CH_IN1: &str = "in1";
static CH_IN2: &str = "in2";

static CONFIG_STREAM: &str = "stream";
static CONFIG_KEY1: &str = "key1";
static CONFIG_KEY2: &str = "key2";

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    defs.insert(
        "$stream".to_string(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$stream",
            Some(new_boxed::<StreamAgent>),
        )
        .with_title("Stream")
        .with_category(CATEGORY)
        .with_inputs(vec![CH_DATA])
        .with_outputs(vec![CH_DATA])
        .with_default_config(vec![(
            CONFIG_STREAM.into(),
            AgentConfigEntry::new(AgentValue::new_string(""), "string"),
        )]),
    );

    defs.insert(
        "$stream_zip2".to_string(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$stream_zip2",
            Some(new_boxed::<StreamZipAgent>),
        )
        .with_title("Zip2")
        .with_category(CATEGORY)
        .with_inputs(vec![CH_IN1, CH_IN2])
        .with_outputs(vec![CH_DATA])
        .with_default_config(vec![
            (
                CONFIG_STREAM.into(),
                AgentConfigEntry::new(AgentValue::new_string(""), "string"),
            ),
            (
                CONFIG_KEY1.into(),
                AgentConfigEntry::new(AgentValue::new_string(""), "string"),
            ),
            (
                CONFIG_KEY2.into(),
                AgentConfigEntry::new(AgentValue::new_string(""), "string"),
            ),
        ]),
    );
}
