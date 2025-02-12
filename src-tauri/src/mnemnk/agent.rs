use std::collections::{BTreeMap, HashMap};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::Duration;
use std::vec;

use anyhow::{Context as _, Result};
use regex::Regex;
use serde_json::Value;
use tauri::{AppHandle, Manager};
use tauri_plugin_shell::process::{CommandChild, CommandEvent};
use tauri_plugin_shell::ShellExt;
use tokio::sync::mpsc;

use super::settings::{self, AgentSettings, MnemnkSettings};
use super::store;

pub struct AgentCommands {
    agents: HashMap<String, CommandChild>,
    catalog: BTreeMap<String, PathBuf>,
}

#[derive(Clone, Debug)]
enum AgentMessage {
    Read {
        agent: String,
        kind: String,
    },
    Store {
        agent: String,
        kind: String,
        value: Value,
    },
    Write {
        agent: String,
        kind: String,
        value: Value,
    },
}

struct AgentBoard {
    board: HashMap<String, Value>,
    subscribers: HashMap<String, Vec<String>>,
}

pub fn init(app: &AppHandle) {
    let agent_commands = AgentCommands {
        agents: HashMap::new(),
        catalog: search_agents(),
    };
    app.manage(Mutex::new(agent_commands));

    let agent_board = AgentBoard {
        board: HashMap::new(),
        subscribers: HashMap::new(),
    };
    app.manage(Mutex::new(agent_board));

    let agents;
    let settings = app.state::<Mutex<MnemnkSettings>>();
    {
        let settings = settings.lock().unwrap();
        agents = settings.agents.clone();
    }

    let (tx, mut rx) = mpsc::channel(128);

    for (agent, settings) in &agents {
        start(app, agent, settings, tx.clone()).unwrap_or_else(|e| {
            log::error!("Failed to start agent: {}", e);
        });
    }

    // TODO: each message should have an origin field to prevent infinite loop
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(message) = rx.recv().await {
            use AgentMessage::*;

            match message {
                Read { agent, kind } => {
                    let value = read_board(&app_handle, &kind);
                    let agent_commands = app_handle.state::<Mutex<AgentCommands>>();
                    let mut agent_commands = agent_commands.lock().unwrap();
                    let child = agent_commands.agents.get_mut(&agent).unwrap();
                    if let Some(value) = value {
                        if let Err(e) =
                            child.write(format!("READ {} {}\n", kind, value.to_string()).as_bytes())
                        {
                            log::error!("Failed to write: {} {}", agent, e);
                        }
                    } else {
                        if let Err(e) = child.write(format!("READ {}\n", kind).as_bytes()) {
                            log::error!("Failed to write: {} {}", agent, e);
                        }
                    }
                }
                Store { agent, kind, value } => {
                    write_board(&app_handle, &agent, &kind, &value);
                    publish(&app_handle, &agent, &kind, &value);
                    if let Err(e) = store::store(&app_handle, agent, kind, value).await {
                        log::error!("Failed to store: {}", e);
                    }
                }
                Write { agent, kind, value } => {
                    // log::debug!("write {} {} {}", agent, kind, value.to_string());
                    write_board(&app_handle, &agent, &kind, &value);
                    publish(&app_handle, &agent, &kind, &value);
                }
            }
        }
    });
}

fn start(
    app: &AppHandle,
    agent: &str,
    settings: &AgentSettings,
    main_tx: mpsc::Sender<AgentMessage>,
) -> Result<()> {
    // sanitize the agent name
    let re = Regex::new(r"[^a-zA-Z0-9_-]").unwrap();
    if let Some(_) = re.find(agent) {
        log::error!("Invalid agent name: {}", re.replace_all(agent, "X"));
        return Err(anyhow::anyhow!("Invalid agent name"));
    }

    if !settings.enabled.unwrap_or(false) {
        log::info!("Agent {} is disabled", agent);
        return Ok(());
    }

    let agent_commands = app.state::<Mutex<AgentCommands>>();
    let agent_path;
    {
        let agent_commands = agent_commands.lock().unwrap();
        if agent_commands.agents.contains_key(agent) {
            log::error!("Agent {} is already running", agent);
            return Err(anyhow::anyhow!("Agent is already running"));
        }
        agent_path = agent_commands
            .catalog
            .get(agent)
            .context("Agent not found")?
            .clone();
    }

    log::info!("Starting agent: {}", agent);

    let config = settings.config.clone().unwrap_or(Value::Null);
    let sidecar_command = if config == Value::Null {
        app.shell().command(agent_path.as_os_str())
    } else {
        app.shell()
            .command(agent_path.as_os_str())
            .args(vec!["-c", serde_json::to_string(&config).unwrap().as_str()])
    };

    let (mut rx, child) = sidecar_command.spawn().context("Failed to spawn sidecar")?;

    let agent_commands = app.state::<Mutex<AgentCommands>>();
    {
        let mut agent_commands = agent_commands.lock().unwrap();
        agent_commands.agents.insert(agent.to_string(), child);
    }

    let app_handle = app.clone();
    let agent = agent.to_string();
    tauri::async_runtime::spawn(async move {
        // read events such as stdout
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line_bytes) => {
                    let line = String::from_utf8_lossy(&line_bytes);
                    log::debug!("stdout from {}: {:.200}", &agent, &line);

                    let (cmd, args) = parse_stdout(&line);
                    match cmd {
                        "CONFIG" => {
                            let value = serde_json::from_str::<Value>(args);
                            if let Err(e) = value {
                                log::error!("Failed to parse config: {}", e);
                                continue;
                            }
                            recieve_config(&app_handle, &agent, value.unwrap()).unwrap_or_else(
                                |e| {
                                    log::error!("Failed to receive config: {}", e);
                                },
                            )
                        }
                        "READ" => {
                            let kind = args.to_string();
                            if kind.is_empty() {
                                log::error!("Invalid READ command: {:.40}", &line);
                                continue;
                            }
                            main_tx
                                .send(AgentMessage::Read {
                                    agent: agent.clone(),
                                    kind,
                                })
                                .await
                                .unwrap_or_else(|e| {
                                    log::error!("Failed to send message: {}", e);
                                });
                        }
                        "STORE" => {
                            let kind_value = args.split_once(" ");
                            if kind_value.is_none() {
                                log::error!("Invalid STORE command: {:.40}", &line);
                                continue;
                            }
                            let (kind, value) = kind_value.unwrap();
                            let value = serde_json::from_str::<Value>(value);
                            if value.is_err() {
                                log::error!("Failed to parse value: {:.40}", &line);
                                continue;
                            }
                            main_tx
                                .send(AgentMessage::Store {
                                    agent: agent.clone(),
                                    kind: kind.to_string(),
                                    value: value.unwrap(),
                                })
                                .await
                                .unwrap_or_else(|e| {
                                    log::error!("Failed to send message: {}", e);
                                });
                        }
                        "SUBSCRIBE" => {
                            let kind = args.to_string();
                            subscribe(&app_handle, &agent, &kind);
                        }
                        "WRITE" => {
                            let kind_value = args.split_once(" ");
                            if kind_value.is_none() {
                                log::error!("Invalid WRITE command: {}", line);
                                continue;
                            }
                            let (kind, value) = kind_value.unwrap();
                            let value = serde_json::from_str::<Value>(value);
                            if value.is_err() {
                                log::error!("Failed to parse value: {}", line);
                                continue;
                            }
                            main_tx
                                .send(AgentMessage::Write {
                                    agent: agent.clone(),
                                    kind: kind.to_string(),
                                    value: value.unwrap(),
                                })
                                .await
                                .unwrap_or_else(|e| {
                                    log::error!("Failed to send message: {}", e);
                                });
                        }
                        _ => {
                            log::error!("Unknown command: {} {}", agent, cmd);
                        }
                    }
                }

                CommandEvent::Stderr(line_bytes) => {
                    let line = String::from_utf8_lossy(&line_bytes);
                    log::error!("{} {}", agent, line);
                }

                CommandEvent::Terminated(status) => {
                    log::info!("Agent exited: {} with status: {:?}", agent, status);
                    let agent_commands = app_handle.state::<Mutex<AgentCommands>>();
                    {
                        let mut agent_commands = agent_commands.lock().unwrap();
                        agent_commands.agents.remove(&agent);
                    }
                    break;
                }

                CommandEvent::Error(e) => {
                    log::error!("Error: {} {}", agent, e);
                }

                _ => {
                    log::error!("Unknown event: {} {:?}", agent, event);
                }
            }
        }
    });
    Ok(())
}

// pub fn stop(app: &AppHandle, program: &str) -> Result<()> {
//     let agent_commands = app.state::<Mutex<AgentCommands>>();
//     let mut agent_commands = agent_commands.lock().unwrap();
//     let sidecar_command = agent_commands
//         .agents
//         .remove(program)
//         .context("Agent not found")?;
//     sidecar_command.kill().context("Failed to kill sidecar")?;
//     Ok(())
// }

pub fn quit(app: &AppHandle) {
    let agent_commands = app.state::<Mutex<AgentCommands>>();
    {
        // send QUIT command to all agents
        let mut agent_commands = agent_commands.lock().unwrap();
        let programs = agent_commands
            .agents
            .keys()
            .cloned()
            .collect::<vec::Vec<String>>();
        for program in programs {
            log::info!("Stopping agent: {}", program);
            if let Some(child) = agent_commands.agents.get_mut(&program) {
                child.write("QUIT\n".as_bytes()).unwrap_or_else(|e| {
                    log::error!("Failed to write to {}: {}", program, e);
                });
            }
        }
    }

    // wait for all agents to exit
    for _ in 0..20 {
        {
            let agent_commands = app.state::<Mutex<AgentCommands>>();
            let agent_commands = agent_commands.lock().unwrap();
            if agent_commands.agents.is_empty() {
                return;
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }

    {
        // kill remaining agents
        let mut agent_commands = agent_commands.lock().unwrap();
        let programs = agent_commands
            .agents
            .keys()
            .cloned()
            .collect::<vec::Vec<String>>();
        for program in programs {
            log::warn!("Killing agent: {}", program);
            if let Some(command) = agent_commands.agents.remove(&program) {
                command.kill().unwrap_or_else(|e| {
                    log::error!("Failed to kill agent: {} {}", program, e);
                });
            }
        }
    }
}

fn app_path() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    path
}

fn user_path() -> Vec<PathBuf> {
    let mut path_dirs = if let Ok(path) = env::var("PATH") {
        env::split_paths(&path).collect()
    } else {
        vec![]
    };
    path_dirs.insert(0, app_path());
    path_dirs
}

// is_executable and search_agents are based on cargo/main.rs

#[cfg(unix)]
fn is_executable<P: AsRef<Path>>(path: P) -> bool {
    use std::os::unix::prelude::*;
    fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(windows)]
fn is_executable<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().is_file()
}

fn search_agents() -> BTreeMap<String, PathBuf> {
    let prefix = "mnemnk-";
    let suffix = env::consts::EXE_SUFFIX;
    let mut agents = BTreeMap::new();
    for path in user_path() {
        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            _ => continue,
        };
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                let Some(filename) = path.file_name().and_then(|s| s.to_str()) else {
                    continue;
                };
                let Some(name) = filename
                    .strip_prefix(prefix)
                    .and_then(|s| s.strip_suffix(suffix))
                else {
                    continue;
                };
                if name == "app" {
                    continue;
                }
                if is_executable(&path) {
                    agents.insert(name.to_string(), path);
                }
            }
        }
    }
    dbg!(&agents);
    agents
}

fn parse_stdout(line: &str) -> (&str, &str) {
    let (cmd, args) = line.split_once(" ").unwrap_or((line, ""));
    (cmd.trim(), args.trim())
}

fn recieve_config(app: &AppHandle, agent: &str, config: Value) -> Result<()> {
    let settings = app.state::<Mutex<MnemnkSettings>>();
    {
        let mut settings = settings.lock().unwrap();
        let agent_settings = settings.agents.get_mut(agent).unwrap();
        agent_settings.config = Some(config);
        // settings.agents.insert(agent.to_string(), agent_settings);
    }
    settings::save(app);
    Ok(())
}

fn write_board(app: &AppHandle, _agent: &str, kind: &str, value: &Value) {
    let agent_board = app.state::<Mutex<AgentBoard>>();
    {
        let mut agent_board = agent_board.lock().unwrap();
        agent_board.board.insert(kind.to_string(), value.clone());
    }
}

fn read_board(app: &AppHandle, agent: &str) -> Option<Value> {
    let agent_board = app.state::<Mutex<AgentBoard>>();
    let agent_board = agent_board.lock().unwrap();
    agent_board.board.get(agent).cloned()
}

fn publish(app: &AppHandle, agent: &str, kind: &str, value: &Value) {
    log::debug!("publish {} {}", agent, kind);
    let agent_board = app.state::<Mutex<AgentBoard>>();
    let subscribers;
    {
        let agent_board = agent_board.lock().unwrap();
        subscribers = agent_board.subscribers.get(kind).cloned();
    }
    if let Some(subscribers) = subscribers {
        let agent_commands = app.state::<Mutex<AgentCommands>>();
        for subscriber in subscribers {
            if subscriber == agent {
                continue;
            }
            let mut agent_commands = agent_commands.lock().unwrap();
            if let Some(child) = agent_commands.agents.get_mut(&subscriber) {
                log::debug!("PUBLISH {} {} {}", agent, kind, value.to_string());
                child
                    .write(format!("PUBLISH {} {} {}\n", agent, kind, value.to_string()).as_bytes())
                    .unwrap_or_else(|e| {
                        log::error!("Failed to write to {}: {}", subscriber, e);
                    });
            }
        }
    }
}

fn subscribe(app: &AppHandle, agent: &str, kind: &str) {
    log::debug!("subscribe {} {}", agent, kind);
    let agent_board = app.state::<Mutex<AgentBoard>>();
    {
        let mut agent_board = agent_board.lock().unwrap();
        if let Some(subscribers) = agent_board.subscribers.get_mut(kind) {
            subscribers.push(agent.to_string());
        } else {
            agent_board
                .subscribers
                .insert(kind.to_string(), vec![agent.to_string()]);
        }
    }
}
