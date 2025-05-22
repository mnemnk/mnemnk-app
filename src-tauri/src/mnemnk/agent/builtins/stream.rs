use anyhow::{bail, Context as _, Result};
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::definition::AGENT_KIND_BUILTIN;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentConfigEntry, AgentContext, AgentData, AgentDefinition,
    AgentDefinitions, AgentOutput, AgentValue, AgentValueMap, AsAgent, AsAgentData,
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

    fn process(&mut self, ctx: AgentContext, data: AgentData) -> Result<()> {
        let stream_name = self
            .config()
            .context("missing config")?
            .get(CONFIG_STREAM)
            .context("missing stream")?
            .as_str()
            .context("failed as_str")?
            .to_string();
        if stream_name.is_empty() {
            self.try_output(ctx, CH_DATA, data)
                .context("Failed to output")?;
            return Ok(());
        }

        self.last_id += 1;
        let key = format!("{}:$stream:{}", self.flow_name(), stream_name);
        let new_ctx = ctx.with_var(key, AgentValue::new_integer(self.last_id));
        self.try_output(new_ctx, CH_DATA, data)
            .context("Failed to output")?;

        Ok(())
    }
}

// Stream Zip agent
struct StreamZipAgent {
    data: AsAgentData,
    n: usize,
    in_channels: Vec<String>,
    keys: Vec<String>,
    input_value: Vec<Option<AgentValue>>,
    current_id: i64,
}

impl AsAgent for StreamZipAgent {
    fn new(
        app: AppHandle,
        id: String,
        def_name: String,
        config: Option<AgentConfig>,
    ) -> Result<Self> {
        let mut this = Self {
            data: AsAgentData::new(app, id, def_name, config.clone()),
            n: 0,
            in_channels: Vec::new(),
            keys: Vec::new(),
            input_value: Vec::new(),
            current_id: -1,
        };
        if let Some(c) = config {
            AsAgent::set_config(&mut this, c)?;
        } else {
            bail!("missing config");
        }
        Ok(this)
    }

    fn data(&self) -> &AsAgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AsAgentData {
        &mut self.data
    }

    fn set_config(&mut self, config: AgentConfig) -> Result<()> {
        let n = config
            .get(CONFIG_N)
            .context("missing n")?
            .as_i64()
            .context("failed as_i64")?;
        if n <= 1 {
            bail!("n must be greater than 1");
        }
        let n = n as usize;
        if self.n == n {
            self.keys = (0..self.n)
                .map(|i| config.get_string_or_default(&format!("key{}", i + 1)))
                .collect();
        } else {
            self.n = n;
            self.in_channels = (0..self.n).map(|i| format!("in{}", i + 1)).collect();
            self.keys = (0..self.n)
                .map(|i| config.get_string_or_default(&format!("key{}", i + 1)))
                .collect();
            self.input_value = vec![None; self.n];
            self.current_id = -1;
        }
        Ok(())
    }

    fn process(&mut self, ctx: AgentContext, data: AgentData) -> Result<()> {
        for i in 0..self.n {
            if self.keys[i].is_empty() {
                bail!("key{} is not set", i + 1);
            }
        }

        let stream_name = self
            .config()
            .context("missing config")?
            .get(CONFIG_STREAM)
            .context("missing stream")?
            .as_str()
            .context("failed as_str")?
            .to_string();

        if !stream_name.is_empty() {
            let key = format!("{}:$stream:{}", self.flow_name(), stream_name);
            let Some(value) = ctx.get_var(key.as_str()) else {
                // value does not have the stream key
                return Ok(());
            };
            let Some(stream_id) = value.as_i64() else {
                // value is not a number
                return Ok(());
            };
            if stream_id != self.current_id {
                self.current_id = stream_id;
                for i in 0..self.n {
                    self.input_value[i] = None;
                }
            }
        }

        for i in 0..self.n {
            if ctx.ch() == self.in_channels[i] {
                self.input_value[i] = Some(data.value.clone());
            }
        }

        // Check if all inputs are present
        for i in 0..self.n {
            if self.input_value[i].is_none() {
                return Ok(());
            }
        }

        // All inputs are present, create the output
        let mut map = AgentValueMap::new();
        for i in 0..self.n {
            let key = self.keys[i].clone();
            let value = self.input_value[i].take().unwrap();
            map.insert(key, value);
        }
        let out_data = AgentData::new_object(map);

        self.try_output(ctx, CH_DATA, out_data)
            .context("Failed to output")?;

        Ok(())
    }
}

static CATEGORY: &str = "Core/Stream";

static CH_DATA: &str = "data";
static CH_IN1: &str = "in1";
static CH_IN2: &str = "in2";
static CH_IN3: &str = "in3";
static CH_IN4: &str = "in4";

static CONFIG_STREAM: &str = "stream";
static CONFIG_KEY1: &str = "key1";
static CONFIG_KEY2: &str = "key2";
static CONFIG_KEY3: &str = "key3";
static CONFIG_KEY4: &str = "key4";
static CONFIG_N: &str = "n";

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
                CONFIG_N.into(),
                AgentConfigEntry::new(AgentValue::new_integer(2), "integer").with_hidden(true),
            ),
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

    defs.insert(
        "$stream_zip3".to_string(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$stream_zip3",
            Some(new_boxed::<StreamZipAgent>),
        )
        .with_title("Zip3")
        .with_category(CATEGORY)
        .with_inputs(vec![CH_IN1, CH_IN2, CH_IN3])
        .with_outputs(vec![CH_DATA])
        .with_default_config(vec![
            (
                CONFIG_N.into(),
                AgentConfigEntry::new(AgentValue::new_integer(3), "integer").with_hidden(true),
            ),
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
            (
                CONFIG_KEY3.into(),
                AgentConfigEntry::new(AgentValue::new_string(""), "string"),
            ),
        ]),
    );

    defs.insert(
        "$stream_zip4".to_string(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$stream_zip4",
            Some(new_boxed::<StreamZipAgent>),
        )
        .with_title("Zip4")
        .with_category(CATEGORY)
        .with_inputs(vec![CH_IN1, CH_IN2, CH_IN3, CH_IN4])
        .with_outputs(vec![CH_DATA])
        .with_default_config(vec![
            (
                CONFIG_N.into(),
                AgentConfigEntry::new(AgentValue::new_integer(4), "integer").with_hidden(true),
            ),
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
            (
                CONFIG_KEY3.into(),
                AgentConfigEntry::new(AgentValue::new_string(""), "string"),
            ),
            (
                CONFIG_KEY4.into(),
                AgentConfigEntry::new(AgentValue::new_string(""), "string"),
            ),
        ]),
    );
}
