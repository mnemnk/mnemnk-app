use std::sync::{Arc, Mutex};
use std::vec;

use anyhow::{bail, Context as _, Result};
use regex::Regex;
use tauri::async_runtime::JoinHandle;
use tauri::{AppHandle, Manager};

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentConfigEntry, AgentData, AgentDefinition, AgentDefinitions,
    AgentDisplayConfigEntry, AgentEnv, AgentStatus, AgentValue, AsAgent, AsAgentData,
};

// Counter
struct CounterAgent {
    data: AsAgentData,
    count: i64,
}

impl AsAgent for CounterAgent {
    fn new(
        app: AppHandle,
        id: String,
        def_name: String,
        config: Option<AgentConfig>,
    ) -> Result<Self> {
        Ok(Self {
            data: AsAgentData::new(app, id, def_name, config),
            count: 0,
        })
    }

    fn data(&self) -> &AsAgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AsAgentData {
        &mut self.data
    }

    fn start(&mut self) -> Result<()> {
        self.count = 0;
        self.emit_display("count".to_string(), AgentData::new_integer(0))?;
        Ok(())
    }

    fn process(&mut self, ch: String, _data: AgentData) -> Result<()> {
        if ch == "reset" {
            self.count = 0;
        } else if ch == "in" {
            self.count += 1;
        }
        self.try_output("count".to_string(), AgentData::new_integer(self.count))?;
        self.emit_display("count".to_string(), AgentData::new_integer(self.count))
    }
}

// Interval Timer Agent
pub struct IntervalTimerAgent {
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
                if new_interval != self.interval_ms && *self.status() == AgentStatus::Start {
                    // Restart the timer with the new interval
                    self.stop_timer()?;
                    self.start_timer()?;
                }
                self.interval_ms = new_interval;
            }
        }
        Ok(())
    }

    fn process(&mut self, _ch: String, _data: AgentData) -> Result<()> {
        // This agent doesn't process input, it just outputs on a timer
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

// Memory Agent
//
// Retains the last `n` of the input data and outputs them.
// The output data `kind` matches that of the first data.
struct MemoryAgent {
    data: AsAgentData,
    memory: Vec<AgentData>,
}

const DEFAULT_N: i64 = 10;

fn get_n(config: &Option<AgentConfig>) -> i64 {
    if let Some(config) = config {
        if let Some(n) = config.get("n") {
            return n.as_i64().unwrap_or(DEFAULT_N);
        }
    }
    DEFAULT_N
}

impl AsAgent for MemoryAgent {
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
            memory: vec![],
        })
    }

    fn data(&self) -> &AsAgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AsAgentData {
        &mut self.data
    }

    fn set_config(&mut self, config: AgentConfig) -> Result<()> {
        // Update n if it has changed
        if let Some(n) = config.get("n") {
            let new_n = n.as_i64().unwrap_or(DEFAULT_N);
            if new_n > 0 {
                let new_n = new_n as usize;

                // If the new n is smaller than the current number of data,
                // trim the oldest data to fit the new n
                if new_n < self.memory.len() {
                    let data_to_remove = self.memory.len() - new_n;
                    self.memory.drain(0..data_to_remove);
                }
            }
        }

        Ok(())
    }

    fn process(&mut self, ch: String, data: AgentData) -> Result<()> {
        if ch == "reset" {
            // Reset command empties the memory
            self.memory.clear();

            self.try_output("reset".to_string(), AgentData::new_unit())?;
        } else if ch == "in" {
            // Add new data to memory
            self.memory.push(data.clone());

            // Trim to max size if needed
            let n = get_n(&self.data().config);
            if n > 0 {
                let n = n as usize;
                // If the memory is larger than n, remove the oldest item
                if self.memory.len() > n {
                    self.memory.remove(0); // Remove oldest item
                }
            }

            // Output the memory array
            let memory_array =
                AgentValue::new_array(self.memory.iter().map(|data| data.value.clone()).collect());
            self.try_output(
                "memory".to_string(),
                AgentData {
                    kind: self.memory[0].kind.clone(),
                    value: memory_array,
                },
            )?;
        }

        Ok(())
    }
}

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    // Display Data
    defs.insert(
        "$counter".into(),
        AgentDefinition::new("Builtin", "$counter", Some(new_boxed::<CounterAgent>))
            .with_title("Counter")
            // .with_description("Display value on the node")
            .with_category("Core/Utils")
            .with_inputs(vec!["in", "reset"])
            .with_outputs(vec!["count"])
            .with_display_config(vec![(
                "count".into(),
                AgentDisplayConfigEntry::new("integer").with_hide_title(),
            )]),
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
        .with_category("Core/Utils")
        .with_outputs(vec!["unit"])
        .with_default_config(vec![(
            "interval".into(),
            AgentConfigEntry::new(AgentValue::new_string("10s".to_string()), "string")
                .with_title("Interval")
                .with_description("Time interval (ex. 10s, 5m, 100ms, 1h, 1d)"),
        )]),
    );

    // MemoryAgent
    defs.insert(
        "$memory".into(),
        AgentDefinition::new("Memory", "$memory", Some(new_boxed::<MemoryAgent>))
            .with_title("Memory")
            .with_description("Stores recent input data")
            .with_category("Core/Utils")
            .with_inputs(vec!["in", "reset"])
            .with_outputs(vec!["memory", "reset"])
            .with_default_config(vec![(
                "n".into(),
                AgentConfigEntry::new(AgentValue::new_integer(10), "integer"),
            )]),
    );
}
