use std::collections::HashSet;

use active_win_pos_rs::get_active_window;
use anyhow::Result;
use chrono::Utc;
use clap::Parser;
use log::{debug, error, info};
use serde_json::Value;
use tokio::io::{stdin, AsyncBufReadExt, BufReader};
use tokio::signal::ctrl_c;
use tokio::time;

const AGENT_NAME: &str = "mnemnk-application";
const KIND: &str = "application";

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct AgentConfig {
    interval: u64,
    ignore: Vec<String>,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            interval: 10,
            ignore: default_ignore(),
        }
    }
}

#[cfg(target_os = "linux")]
fn default_ignore() -> Vec<String> {
    vec![]
}

#[cfg(target_os = "macos")]
fn default_ignore() -> Vec<String> {
    vec!["scrnsave.scr".to_string()]
}

#[cfg(target_os = "windows")]
fn default_ignore() -> Vec<String> {
    vec!["LockApp.exe".to_string()]
}

impl From<&str> for AgentConfig {
    fn from(s: &str) -> Self {
        let mut config = AgentConfig::default();
        if let Value::Object(c) = serde_json::from_str(s).unwrap_or(Value::Null) {
            if let Some(interval) = c.get("interval") {
                config.interval = interval.as_u64().unwrap();
            }
            if let Some(ignore) = c.get("ignore") {
                config.ignore = ignore
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_str().unwrap().to_string())
                    .collect();
            }
        }
        config
    }
}

#[derive(Clone, Debug, PartialEq, serde::Serialize)]
struct ApplicationEvent {
    t: i64,
    // process_id: i64,
    // path: String,
    name: String,
    title: String,
    x: i64,
    y: i64,
    width: i64,
    height: i64,
    text: String,
}

struct ApplicationAgent {
    config: AgentConfig,
    last_event: Option<ApplicationEvent>,
    ignore: HashSet<String>,
}

impl ApplicationAgent {
    fn new(config: AgentConfig) -> Self {
        Self {
            ignore: config.ignore.clone().into_iter().collect(),
            config,
            last_event: None,
        }
    }

    async fn run(&mut self) -> Result<()> {
        let mut interval = time::interval(time::Duration::from_secs(self.config.interval));

        let mut reader = BufReader::new(stdin());
        let mut line = String::new();

        // Main loop with graceful shutdown
        loop {
            tokio::select! {
                // Wait for next interval tick
                _ = interval.tick() => {
                    if let Err(e) = self.execute_task().await {
                        error!("Failed to execute task: {}", e);
                    }
                }
                // Read from stdin
                _ = reader.read_line(&mut line) => {
                    if let Err(e) = self.process_line(&line).await {
                        error!("Failed to process line: {}", e);
                    }
                    line.clear();
                }

                // Handle Ctrl+C
                _ = ctrl_c() => {
                    info!("\nShutting down mnemnk-application.");
                    break;
                }
            }
        }
        Ok(())
    }

    async fn execute_task(&mut self) -> Result<()> {
        let app_event = check_application().await;

        if self.is_same(&app_event) {
            log::debug!("Same as the last application event");
            return Ok(());
        }

        if self.is_ignored(&app_event) {
            log::debug!("Ignored application event");
            return Ok(());
        }

        if let Some(app_event) = app_event {
            // debug!("check_application: {:?}", app_event);
            let app_event_json = serde_json::to_string(&app_event)?;
            println!("STORE {} {}", KIND, app_event_json);
        }
        Ok(())
    }

    fn is_same(&mut self, app_event: &Option<ApplicationEvent>) -> bool {
        if let Some(app_event) = app_event {
            if let Some(last_event) = &self.last_event {
                if app_event.x == last_event.x
                    && app_event.y == last_event.y
                    && app_event.width == last_event.width
                    && app_event.height == last_event.height
                    && app_event.text == last_event.text
                {
                    return true;
                }
            }
        }
        self.last_event = app_event.clone();
        false
    }

    fn is_ignored(&self, app_event: &Option<ApplicationEvent>) -> bool {
        if let Some(app_event) = app_event {
            if self.ignore.contains(&app_event.name) {
                return true;
            }
        }
        false
    }

    async fn process_line(&self, line: &str) -> Result<()> {
        debug!("process_line: {}", line);

        if let Some((cmd, _args)) = parse_line(line) {
            match cmd {
                // "GET_CONFIG" => {
                //     println!("CONFIG {}", serde_json::to_string(&self.config)?);
                // }
                "QUIT" => {
                    log::info!("QUIT {}.", AGENT_NAME);
                    std::process::exit(0);
                }
                _ => {
                    error!("Unknown command: {}", cmd);
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(short = 'c', long = "config", help = "JSON config string")]
    config: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    // env_logger::from_env(env_logger::Env::default().default_filter_or("debug")).init();

    let args = Args::parse();
    let config = args.config.as_deref().unwrap_or_default().into();
    println!("CONFIG {}", serde_json::to_string(&config)?);

    info!("Starting {}.", AGENT_NAME);
    let mut agent = ApplicationAgent::new(config);
    agent.run().await?;

    Ok(())
}

async fn check_application() -> Option<ApplicationEvent> {
    debug!("check_application");
    match get_active_window() {
        Ok(win) => {
            // let path = win.process_path.to_string_lossy().to_string();
            let text = format!("{} {}", win.app_name, win.title).trim().to_string();
            let info = ApplicationEvent {
                t: Utc::now().timestamp_millis(),
                // process_id: win.process_id as i64,
                // path: path,
                name: win.app_name,
                title: win.title,
                x: win.position.x as i64,
                y: win.position.y as i64,
                width: win.position.width as i64,
                height: win.position.height as i64,
                text: text,
            };
            Some(info)
        }
        Err(_) => None,
    }
}

fn parse_line(line: &str) -> Option<(&str, &str)> {
    if line.is_empty() {
        return None;
    }

    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    if let Some((cmd, args)) = line.split_once(" ") {
        Some((cmd, args))
    } else {
        Some((line, ""))
    }
}

// fn get_config(config: &AgentConfig, _args: &str) -> Result<()> {
//     println!("CONFIG {}", serde_json::to_string(config)?);
//     Ok(())
// }

// fn set_config(config: &mut AgentConfig, args: &str) -> Result<()> {
//     let new_config: AgentConfig = serde_json::from_str(args)?;
//     *config = new_config;
//     // TODO: it's necessary to restart for the new config to take effect
//     Ok(())
// }
