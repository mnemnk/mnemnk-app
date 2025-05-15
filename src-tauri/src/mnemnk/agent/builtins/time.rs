use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::vec;

use anyhow::{bail, Context as _, Result};
use regex::Regex;
use tauri::async_runtime::JoinHandle;
use tauri::{AppHandle, Manager};

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::definition::AGENT_KIND_BUILTIN;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentConfigEntry, AgentContext, AgentData, AgentDefinition,
    AgentDefinitions, AgentEnv, AgentOutput, AgentStatus, AgentValue, AsAgent, AsAgentData,
};

// Delay Agent
struct DelayAgent {
    data: AsAgentData,
    num_waiting_data: Arc<Mutex<i64>>,
}

impl AsAgent for DelayAgent {
    fn new(
        app: AppHandle,
        id: String,
        def_name: String,
        config: Option<AgentConfig>,
    ) -> Result<Self> {
        Ok(Self {
            data: AsAgentData::new(app, id, def_name, config),
            num_waiting_data: Arc::new(Mutex::new(0)),
        })
    }

    fn data(&self) -> &AsAgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AsAgentData {
        &mut self.data
    }

    fn process(&mut self, ctx: AgentContext, data: AgentData) -> Result<()> {
        let config = self.config().context("Missing config")?;
        let delay_ms = config.get_integer_or(CONFIG_DELAY, DELAY_MS_DEFAULT);
        let max_num_data = config.get_integer_or(CONFIG_MAX_NUM_DATA, MAX_NUM_DATA_DEFAULT);

        let agent_id = self.id().to_string();
        let app_handle = self.app().clone();

        // To avoid generating too many timers
        {
            let num_waiting_data = self.num_waiting_data.clone();
            let mut num_waiting_data = num_waiting_data.lock().unwrap();
            if *num_waiting_data >= max_num_data {
                return Ok(());
            }
            *num_waiting_data += 1;
        }

        let num_waiting_data = self.num_waiting_data.clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(Duration::from_millis(delay_ms as u64)).await;

            let env = app_handle.state::<crate::mnemnk::agent::AgentEnv>();
            if let Err(e) = env.send_agent_out(agent_id, ctx, data).await {
                log::error!("Failed to send delayed output: {}", e);
            }

            let mut num_waiting_data = num_waiting_data.lock().unwrap();
            *num_waiting_data -= 1;
        });

        Ok(())
    }
}

// Interval Timer Agent
struct IntervalTimerAgent {
    data: AsAgentData,
    timer_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    interval_ms: u64,
}

impl IntervalTimerAgent {
    fn start_timer(&mut self) -> Result<()> {
        let app_handle = self.app().clone();
        let agent_id = self.id().to_string();
        let timer_handle = self.timer_handle.clone();
        let interval_ms = self.interval_ms;

        let handle = tauri::async_runtime::spawn(async move {
            loop {
                // Sleep for the configured interval
                tokio::time::sleep(tokio::time::Duration::from_millis(interval_ms)).await;

                // Check if we've been stopped
                if let Ok(handle) = timer_handle.lock() {
                    if handle.is_none() {
                        break;
                    }
                }

                // Create a unit output
                let env = app_handle.state::<AgentEnv>();
                if let Err(e) = env.try_send_agent_out(
                    agent_id.clone(),
                    AgentContext::new_with_ch(CH_UNIT),
                    AgentData::new_unit(),
                ) {
                    log::error!("Failed to send interval timer output: {}", e);
                }
            }
        });

        // Store the timer handle
        if let Ok(mut timer_handle) = self.timer_handle.lock() {
            *timer_handle = Some(handle);
        }

        Ok(())
    }

    fn stop_timer(&mut self) -> Result<()> {
        // Cancel the timer
        if let Ok(mut timer_handle) = self.timer_handle.lock() {
            if let Some(handle) = timer_handle.take() {
                handle.abort();
            }
        }
        Ok(())
    }
}

impl AsAgent for IntervalTimerAgent {
    fn new(
        app: AppHandle,
        id: String,
        def_name: String,
        config: Option<AgentConfig>,
    ) -> Result<Self> {
        let interval = config
            .as_ref()
            .and_then(|c| c.get_string(CONFIG_INTERVAL))
            .unwrap_or_else(|| INTERVAL_DEFAULT.to_string());
        let interval_ms = parse_duration_to_ms(&interval)?;

        Ok(Self {
            data: AsAgentData::new(app, id, def_name, config),
            timer_handle: Default::default(),
            interval_ms,
        })
    }

    fn data(&self) -> &AsAgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AsAgentData {
        &mut self.data
    }

    fn start(&mut self) -> Result<()> {
        self.start_timer()
    }

    fn stop(&mut self) -> Result<()> {
        self.stop_timer()
    }

    fn set_config(&mut self, config: AgentConfig) -> Result<()> {
        // Check if interval has changed
        if let Some(interval) = config.get_string(CONFIG_INTERVAL) {
            let new_interval = parse_duration_to_ms(&interval)?;
            if new_interval != self.interval_ms {
                self.interval_ms = new_interval;
                if *self.status() == AgentStatus::Start {
                    // Restart the timer with the new interval
                    self.stop_timer()?;
                    self.start_timer()?;
                }
            }
        }
        Ok(())
    }
}

// OnStart
struct OnStartAgent {
    data: AsAgentData,
}

impl AsAgent for OnStartAgent {
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

    fn start(&mut self) -> Result<()> {
        let config = self.config().context("Missing config")?;
        let delay_ms = config.get_integer_or(CONFIG_DELAY, DELAY_MS_DEFAULT);

        let agent_id = self.id().to_string();
        let app_handle = self.app().clone();

        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(Duration::from_millis(delay_ms as u64)).await;

            let env = app_handle.state::<crate::mnemnk::agent::AgentEnv>();
            if let Err(e) = env
                .send_agent_out(
                    agent_id,
                    AgentContext::new_with_ch(CH_UNIT),
                    AgentData::new_unit(),
                )
                .await
            {
                log::error!("Failed to send delayed output: {}", e);
            }
        });

        Ok(())
    }
}

// Throttle agent
struct ThrottleTimeAgent {
    data: AsAgentData,
    timer_handle: Arc<Mutex<Option<JoinHandle<()>>>>,
    time_ms: u64,
    max_num_data: i64,
    waiting_data: Arc<Mutex<Vec<(AgentContext, AgentData)>>>,
}

impl ThrottleTimeAgent {
    fn start_timer(&mut self) -> Result<()> {
        let timer_handle = self.timer_handle.clone();
        let time_ms = self.time_ms;

        let waiting_data = self.waiting_data.clone();
        let app = self.app().clone();
        let agent_id = self.id().to_string();

        let handle = tauri::async_runtime::spawn(async move {
            loop {
                // Sleep for the configured interval
                tokio::time::sleep(tokio::time::Duration::from_millis(time_ms)).await;

                // Check if we've been stopped
                let mut handle = timer_handle.lock().unwrap();
                if handle.is_none() {
                    break;
                }

                // process the waiting data
                let mut wd = waiting_data.lock().unwrap();
                if wd.len() > 0 {
                    // If there are data waiting, output the first one
                    let env = app.state::<AgentEnv>();
                    let (ctx, data) = wd.remove(0);
                    env.try_send_agent_out(agent_id.clone(), ctx, data)
                        .unwrap_or_else(|e| {
                            log::error!("Failed to send delayed output: {}", e);
                        });
                }

                // If there are no data waiting, we stop the timer
                if wd.len() == 0 {
                    handle.take();
                    break;
                }
            }
        });

        // Store the timer handle
        if let Ok(mut timer_handle) = self.timer_handle.lock() {
            *timer_handle = Some(handle);
        }

        Ok(())
    }

    fn stop_timer(&mut self) -> Result<()> {
        // Cancel the timer
        if let Ok(mut timer_handle) = self.timer_handle.lock() {
            if let Some(handle) = timer_handle.take() {
                handle.abort();
            }
        }
        Ok(())
    }
}

impl AsAgent for ThrottleTimeAgent {
    fn new(
        app: AppHandle,
        id: String,
        def_name: String,
        config: Option<AgentConfig>,
    ) -> Result<Self> {
        let time = config
            .as_ref()
            .and_then(|c| c.get_string(CONFIG_TIME))
            .unwrap_or_else(|| TIME_DEFAULT.to_string());
        let time_ms = parse_duration_to_ms(&time)?;

        let max_num_data = config
            .as_ref()
            .and_then(|c| c.get_integer(CONFIG_MAX_NUM_DATA))
            .unwrap_or(0);

        Ok(Self {
            data: AsAgentData::new(app, id, def_name, config),
            timer_handle: Default::default(),
            time_ms,
            max_num_data,
            waiting_data: Arc::new(Mutex::new(vec![])),
        })
    }

    fn data(&self) -> &AsAgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AsAgentData {
        &mut self.data
    }

    fn stop(&mut self) -> Result<()> {
        self.stop_timer()
    }

    fn set_config(&mut self, config: AgentConfig) -> Result<()> {
        // Check if interval has changed
        if let Some(time) = config.get_string(CONFIG_TIME) {
            let new_time = parse_duration_to_ms(&time)?;
            if new_time != self.time_ms {
                self.time_ms = new_time;
            }
        }
        // Check if max_num_data has changed
        if let Some(max_num_data) = config.get_integer(CONFIG_MAX_NUM_DATA) {
            if self.max_num_data != max_num_data {
                let mut wd = self.waiting_data.lock().unwrap();
                let wd_len = wd.len();
                if max_num_data >= 0 && wd_len > (max_num_data as usize) {
                    // If we have reached the max data to keep, we drop the oldest one
                    wd.drain(0..(wd_len - (max_num_data as usize)));
                }
                self.max_num_data = max_num_data;
            }
        }
        Ok(())
    }

    fn process(&mut self, ctx: AgentContext, data: AgentData) -> Result<()> {
        if self.timer_handle.lock().unwrap().is_some() {
            // If the timer is running, we just add the data to the waiting list
            let mut wd = self.waiting_data.lock().unwrap();

            // If max_num_data is 0, we don't need to keep any data
            if self.max_num_data == 0 {
                return Ok(());
            }

            wd.push((ctx, data));
            if self.max_num_data > 0 && wd.len() > self.max_num_data as usize {
                // If we have reached the max data to keep, we drop the oldest one
                wd.remove(0);
            }

            return Ok(());
        }

        // Start the timer
        self.start_timer()?;

        // Output the data
        let ch = ctx.ch().to_string();
        self.try_output(ctx, ch, data)
    }
}

// Parse time duration strings like "2s", "10m", "200ms"
fn parse_duration_to_ms(duration_str: &str) -> Result<u64> {
    const MIN_DURATION: u64 = 10;

    // Regular expression to match number followed by optional unit
    let re = Regex::new(r"^(\d+)(?:([a-zA-Z]+))?$").context("Failed to compile regex")?;

    if let Some(captures) = re.captures(duration_str.trim()) {
        let value: u64 = captures.get(1).unwrap().as_str().parse()?;

        // Get the unit if present, default to "s" (seconds)
        let unit = captures
            .get(2)
            .map_or("s".to_string(), |m| m.as_str().to_lowercase());

        // Convert to milliseconds based on unit
        let milliseconds = match unit.as_str() {
            "ms" => value,               // already in milliseconds
            "s" => value * 1000,         // seconds to milliseconds
            "m" => value * 60 * 1000,    // minutes to milliseconds
            "h" => value * 3600 * 1000,  // hours to milliseconds
            "d" => value * 86400 * 1000, // days to milliseconds
            _ => bail!("Unknown time unit: {}", unit),
        };

        // Ensure we don't return less than the minimum duration
        Ok(std::cmp::max(milliseconds, MIN_DURATION))
    } else {
        // If the string doesn't match the pattern, try to parse it as a plain number
        // and assume it's in seconds
        let value: u64 = duration_str.parse()?;
        Ok(std::cmp::max(value * 1000, MIN_DURATION)) // Convert to ms
    }
}

static CATEGORY: &str = "Core/Time";

static CH_UNIT: &str = "unit";

static CONFIG_DELAY: &str = "delay";
static CONFIG_MAX_NUM_DATA: &str = "max_num_data";
static CONFIG_INTERVAL: &str = "interval";
static CONFIG_TIME: &str = "time";

const DELAY_MS_DEFAULT: i64 = 1000; // 1 second in milliseconds
const MAX_NUM_DATA_DEFAULT: i64 = 10;
static INTERVAL_DEFAULT: &str = "10s";
static TIME_DEFAULT: &str = "1s";

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    // Delay Agent
    defs.insert(
        "$delay".into(),
        AgentDefinition::new(AGENT_KIND_BUILTIN, "$delay", Some(new_boxed::<DelayAgent>))
            .with_title("Delay")
            .with_description("Delays output by a specified time")
            .with_category(CATEGORY)
            .with_inputs(vec!["*"])
            .with_outputs(vec!["*"])
            .with_default_config(vec![
                (
                    CONFIG_DELAY.into(),
                    AgentConfigEntry::new(AgentValue::new_integer(DELAY_MS_DEFAULT), "integer")
                        .with_title("delay (ms)"),
                ),
                (
                    CONFIG_MAX_NUM_DATA.into(),
                    AgentConfigEntry::new(AgentValue::new_integer(MAX_NUM_DATA_DEFAULT), "integer")
                        .with_title("max num data"),
                ),
            ]),
    );

    // Interval Timer Agent
    defs.insert(
        "$interval_timer".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$interval_timer",
            Some(new_boxed::<IntervalTimerAgent>),
        )
        .with_title("Interval Timer")
        .with_description("Outputs a unit signal at specified intervals")
        .with_category(CATEGORY)
        .with_outputs(vec![CH_UNIT])
        .with_default_config(vec![(
            CONFIG_INTERVAL.into(),
            AgentConfigEntry::new(AgentValue::new_string(INTERVAL_DEFAULT), "string")
                .with_description("(ex. 10s, 5m, 100ms, 1h, 1d)"),
        )]),
    );

    // OnStart
    defs.insert(
        "$on_start".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$on_start",
            Some(new_boxed::<OnStartAgent>),
        )
        .with_title("On Start")
        .with_category(CATEGORY)
        .with_outputs(vec![CH_UNIT])
        .with_default_config(vec![(
            CONFIG_DELAY.into(),
            AgentConfigEntry::new(AgentValue::new_integer(DELAY_MS_DEFAULT), "integer")
                .with_title("delay (ms)"),
        )]),
    );

    // Throttle Time Agent
    defs.insert(
        "$throttle_time".into(),
        AgentDefinition::new(
            AGENT_KIND_BUILTIN,
            "$throttle_time",
            Some(new_boxed::<ThrottleTimeAgent>),
        )
        .with_title("Throttle Time")
        .with_category(CATEGORY)
        .with_inputs(vec!["*"])
        .with_outputs(vec!["*"])
        .with_default_config(vec![
            (
                CONFIG_TIME.into(),
                AgentConfigEntry::new(AgentValue::new_string(TIME_DEFAULT), "string")
                    .with_description("(ex. 10s, 5m, 100ms, 1h, 1d)"),
            ),
            (
                CONFIG_MAX_NUM_DATA.into(),
                AgentConfigEntry::new(AgentValue::new_integer(0), "integer")
                    .with_title("max num data")
                    .with_description("0: no data, -1: all data"),
            ),
        ]),
    );
}
