use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::vec;

use anyhow::{bail, Context as _, Result};
use regex::Regex;
use tauri::async_runtime::JoinHandle;
use tauri::{AppHandle, Manager};

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentConfigEntry, AgentData, AgentDefinition, AgentDefinitions, AgentEnv,
    AgentStatus, AgentValue, AsAgent, AsAgentData,
};

// Delay Agent
struct DelayAgent {
    data: AsAgentData,
    num_waiting_tasks: Arc<Mutex<i64>>,
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
            num_waiting_tasks: Arc::new(Mutex::new(0)),
        })
    }

    fn data(&self) -> &AsAgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AsAgentData {
        &mut self.data
    }

    fn process(&mut self, ch: String, data: AgentData) -> Result<()> {
        const DEFAULT_DELAY_MS: i64 = 1000; // Default delay in milliseconds

        let config = self.data.config.as_ref().context("Missing config")?;

        let delay_ms = if let Some(delay) = config.get("delay") {
            delay.as_i64().unwrap_or(DEFAULT_DELAY_MS)
        } else {
            DEFAULT_DELAY_MS
        };

        let max_tasks = if let Some(max_tasks) = config.get("max_tasks") {
            max_tasks.as_i64().unwrap_or(10)
        } else {
            10
        };

        let agent_id = self.id().to_string();
        let app_handle = self.app().clone();
        let channel = ch.clone();
        let agent_data = data.clone();

        // To avoid generating too many tasks.
        {
            let num_waiting_tasks = self.num_waiting_tasks.clone();
            let mut num_waiting_tasks = num_waiting_tasks.lock().unwrap();
            if *num_waiting_tasks >= max_tasks {
                return Ok(());
            }
            *num_waiting_tasks += 1;
        }

        let num_waiting_tasks = self.num_waiting_tasks.clone();
        tauri::async_runtime::spawn(async move {
            tokio::time::sleep(Duration::from_millis(delay_ms as u64)).await;

            let env = app_handle.state::<crate::mnemnk::agent::AgentEnv>();
            if let Err(e) = env.send_agent_out(agent_id, channel, agent_data).await {
                log::error!("Failed to send delayed output: {}", e);
            }

            let mut num_waiting_tasks = num_waiting_tasks.lock().unwrap();
            *num_waiting_tasks -= 1;
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
                    "unit".to_string(),
                    AgentData::new_unit(),
                ) {
                    log::error!("Failed to send interval timer output: {}", e);
                }
            }
        });

        // Store the task handle
        if let Ok(mut timer_handle) = self.timer_handle.lock() {
            *timer_handle = Some(handle);
        }

        Ok(())
    }

    fn stop_timer(&mut self) -> Result<()> {
        // Cancel the timer task
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
        const DEFAULT_INTERVAL_MS: u64 = 10000; // 10 seconds in milliseconds

        let interval_ms = if let Some(config) = &config {
            if let Some(interval) = config.get("interval") {
                if let Some(interval_str) = interval.as_str() {
                    parse_duration_to_ms(interval_str).unwrap_or(DEFAULT_INTERVAL_MS)
                } else {
                    DEFAULT_INTERVAL_MS
                }
            } else {
                DEFAULT_INTERVAL_MS
            }
        } else {
            DEFAULT_INTERVAL_MS
        };

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
        if let Some(interval) = config.get("interval") {
            if let Some(interval_str) = interval.as_str() {
                let new_interval = parse_duration_to_ms(interval_str)?;
                if new_interval != self.interval_ms {
                    self.interval_ms = new_interval;
                    if *self.status() == AgentStatus::Start {
                        // Restart the timer with the new interval
                        self.stop_timer()?;
                        self.start_timer()?;
                    }
                }
            }
        }
        Ok(())
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

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    // Delay Agent
    defs.insert(
        "$delay".into(),
        AgentDefinition::new("Builtin", "$delay", Some(new_boxed::<DelayAgent>))
            .with_title("Delay")
            .with_description("Delays output by a specified time")
            .with_category("Core/Time")
            .with_inputs(vec!["*"])
            .with_outputs(vec!["*"])
            .with_default_config(vec![
                (
                    "delay".into(),
                    AgentConfigEntry::new(AgentValue::new_integer(1000), "integer")
                        .with_title("delay (ms)"),
                ),
                (
                    "max_tasks".into(),
                    AgentConfigEntry::new(AgentValue::new_integer(10), "integer")
                        .with_title("max tasks"),
                ),
            ]),
    );

    // Interval Timer Agent
    defs.insert(
        "$interval_timer".into(),
        AgentDefinition::new(
            "Builtin",
            "$interval_timer",
            Some(new_boxed::<IntervalTimerAgent>),
        )
        .with_title("Interval Timer")
        .with_description("Outputs a unit signal at specified intervals")
        .with_category("Core/Time")
        .with_outputs(vec!["unit"])
        .with_default_config(vec![(
            "interval".into(),
            AgentConfigEntry::new(AgentValue::new_string("10s"), "string")
                .with_description("(ex. 10s, 5m, 100ms, 1h, 1d)"),
        )]),
    );
}
