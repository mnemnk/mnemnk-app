use anyhow::{Context as _, Result};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::time::Duration;
use std::vec;
use tauri::{AppHandle, Manager};
use tauri_plugin_shell::process::{CommandChild, CommandEvent};
use tauri_plugin_shell::ShellExt;
use tokio::sync::mpsc;

use super::config::AgentConfigs;
use super::flow::{find_agent_node, AgentFlows};
use super::AgentMessage;

pub struct AgentCommands {
    // node id -> child process
    pub commands: HashMap<String, CommandChild>,

    // message sender
    pub tx: mpsc::Sender<AgentMessage>,

    // enabled node ids
    pub enabled_nodes: HashSet<String>,

    // node id -> node ids
    pub edges: HashMap<String, Vec<String>>,
}

pub fn init_agent_commands(app: &AppHandle, tx: mpsc::Sender<AgentMessage>) -> Result<()> {
    let agent_commands = AgentCommands {
        commands: HashMap::new(),
        tx,
        enabled_nodes: HashSet::new(),
        edges: HashMap::new(),
    };
    app.manage(Mutex::new(agent_commands));
    Ok(())
}

pub fn start_agent(app: &AppHandle, agent_id: &str) -> Result<()> {
    let agent_name: String;
    let config;
    let flows = app.state::<Mutex<AgentFlows>>();
    {
        let flows = flows.lock().unwrap();
        if let Some(agent_node) = find_agent_node(&flows, agent_id) {
            agent_name = agent_node.name.clone();
            config = agent_node.config.clone();
        } else {
            log::error!("Agent setting for {} not found", agent_id);
            return Err(anyhow::anyhow!("Agent setting not found"));
        }
    }

    let agent_configs = app.state::<Mutex<AgentConfigs>>();
    let agent_path;
    {
        let agent_configs = agent_configs.lock().unwrap();
        if agent_configs.contains_key(&agent_name) {
            agent_path = agent_configs.get(&agent_name).unwrap().path.clone();
        } else {
            log::error!("Agent {} not found", agent_name);
            return Err(anyhow::anyhow!("Agent not found"));
        }
    }
    let agent_path = agent_path.unwrap_or(agent_name.clone());

    let agent_commands = app.state::<Mutex<AgentCommands>>();
    let main_tx;
    {
        let agent_commands = agent_commands.lock().unwrap();
        main_tx = agent_commands.tx.clone();
    }

    log::info!("Starting agent: {} {}", agent_name, agent_id);

    let sidecar_command = if config.is_none() {
        app.shell().command(agent_path)
    } else {
        app.shell()
            .command(agent_path)
            .args(vec!["-c", serde_json::to_string(&config).unwrap().as_str()])
    };

    let (mut rx, child) = sidecar_command.spawn().context("Failed to spawn sidecar")?;

    let agent_commands = app.state::<Mutex<AgentCommands>>();
    {
        let mut agent_commands = agent_commands.lock().unwrap();
        agent_commands.commands.insert(agent_id.to_string(), child);
    }

    let app_handle = app.clone();
    let agent_id = agent_id.to_string();
    tauri::async_runtime::spawn(async move {
        // read events such as stdout
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line_bytes) => {
                    if line_bytes.is_empty() || line_bytes[0] != b'.' {
                        log::debug!(
                            "non-command stdout from {} {}: {:.200}",
                            &agent_name,
                            &agent_id,
                            String::from_utf8_lossy(&line_bytes)
                        );
                        continue;
                    }

                    let line = String::from_utf8_lossy(&line_bytes);
                    // log::debug!("stdout from {}: {:.200}", &agent, &line);

                    let (cmd, args) = parse_stdout(&line);
                    match cmd {
                        ".CONFIG" => {
                            // let value = serde_json::from_str::<Value>(args);
                            // if let Err(e) = value {
                            //     log::error!("Failed to parse config: {}", e);
                            //     continue;
                            // }
                            // recieve_config(&app_handle, &agent_name, value.unwrap()).unwrap_or_else(
                            //     |e| {
                            //         log::error!("Failed to receive config: {}", e);
                            //     },
                            // )
                        }
                        ".CONFIG_SCHEMA" => {
                            // let value = serde_json::from_str::<Value>(args);
                            // if let Err(e) = value {
                            //     log::error!("Failed to parse config schema: {}", e);
                            //     continue;
                            // }
                            // recieve_config_schema(&app_handle, &agent_name, value.unwrap())
                            //     .unwrap_or_else(|e| {
                            //         log::error!("Failed to receive config schema: {}", e);
                            //     })
                        }
                        ".STORE" => {
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
                                .send(AgentMessage::Write {
                                    agent: agent_id.clone(),
                                    kind: kind.to_string(),
                                    value: value.unwrap(),
                                })
                                .await
                                .unwrap_or_else(|e| {
                                    log::error!("Failed to send message: {}", e);
                                });
                        }
                        ".SUBSCRIBE" => {
                            // let kind = args.to_string();
                            // subscribe(&app_handle, &agent_id, &kind);
                        }
                        ".WRITE" => {
                            let kind_value = args.split_once(" ");
                            if kind_value.is_none() {
                                log::error!("Invalid WRITE command: {:.40}", &line);
                                continue;
                            }
                            let (kind, value) = kind_value.unwrap();
                            let value = serde_json::from_str::<Value>(value);
                            if value.is_err() {
                                log::error!("Failed to parse value: {:.40}", &line);
                                continue;
                            }
                            main_tx
                                .send(AgentMessage::Write {
                                    agent: agent_id.clone(),
                                    kind: kind.to_string(),
                                    value: value.unwrap(),
                                })
                                .await
                                .unwrap_or_else(|e| {
                                    log::error!("Failed to send message: {}", e);
                                });
                        }
                        _ => {
                            log::error!("Unknown command: {} {}", agent_id, cmd);
                        }
                    }
                }

                CommandEvent::Stderr(line_bytes) => {
                    let line = String::from_utf8_lossy(&line_bytes);
                    log::debug!("stderr from {} {}: {:.200}", agent_name, agent_id, line);
                }

                CommandEvent::Terminated(status) => {
                    log::info!(
                        "Agent exited: {} {} with status: {:?}",
                        agent_name,
                        agent_id,
                        status
                    );
                    // unsubscribe_agent(&app_handle, &agent_id);
                    let agent_commands = app_handle.state::<Mutex<AgentCommands>>();
                    {
                        let mut agent_commands = agent_commands.lock().unwrap();
                        agent_commands.commands.remove(&agent_id);
                        agent_commands.enabled_nodes.remove(&agent_id);
                    }
                    break;
                }

                CommandEvent::Error(e) => {
                    log::error!("CommandEvent Error {} {}: {}", agent_name, agent_id, e);
                }

                _ => {
                    log::error!(
                        "Unknown CommandEvent: {} {} {:?}",
                        agent_name,
                        agent_id,
                        event
                    );
                }
            }
        }
    });
    Ok(())
}

pub fn stop_agent(app: &AppHandle, agent_id: &str) -> Result<()> {
    let agent_commands = app.state::<Mutex<AgentCommands>>();
    let mut agent_commands = agent_commands.lock().unwrap();
    agent_commands.enabled_nodes.remove(agent_id);
    if let Some(child) = agent_commands.commands.get_mut(agent_id) {
        child.write(".QUIT\n".as_bytes()).unwrap_or_else(|e| {
            log::error!("Failed to write to {}: {}", agent_id, e);
        });
    }
    Ok(())
}

pub fn update_agent_config(app: &AppHandle, agent_id: &str) -> Result<()> {
    let config;
    let flows = app.state::<Mutex<AgentFlows>>();
    {
        let flows = flows.lock().unwrap();
        if let Some(agent_node) = find_agent_node(&flows, agent_id) {
            config = agent_node.config.clone().unwrap_or(HashMap::new());
        } else {
            log::error!("Agent setting for {} not found", agent_id);
            return Err(anyhow::anyhow!("Agent setting not found"));
        }
    }
    let json_config = serde_json::to_value(config).context("Failed to serialize config")?;

    let agent_commands = app.state::<Mutex<AgentCommands>>();
    {
        let mut agent_commands = agent_commands.lock().unwrap();
        if agent_commands.commands.contains_key(agent_id) {
            // the agent is already running, so update the config
            if let Some(child) = agent_commands.commands.get_mut(agent_id) {
                if let Err(e) =
                    child.write(format!(".CONFIG {}\n", json_config.to_string()).as_bytes())
                {
                    log::error!("Failed to set config to {}: {}", agent_id, e);
                    return Err(anyhow::anyhow!("Failed to set config to agent"));
                }
            }
        }
    }

    Ok(())
}

pub fn quit(app: &AppHandle) {
    let agent_commands = app.state::<Mutex<AgentCommands>>();
    {
        // send QUIT command to all agents
        let mut agent_commands = agent_commands.lock().unwrap();
        let agent_ids = agent_commands
            .commands
            .keys()
            .cloned()
            .collect::<vec::Vec<String>>();
        for agent_id in agent_ids {
            log::info!("Stopping agent: {}", agent_id);
            // we cannot use stop_agent here because it will also try to lock aget_commands.
            if let Some(child) = agent_commands.commands.get_mut(&agent_id) {
                child.write(".QUIT\n".as_bytes()).unwrap_or_else(|e| {
                    log::error!("Failed to write to {}: {}", agent_id, e);
                });
            }
        }
    }

    // wait for all agents to exit
    for _ in 0..20 {
        {
            let agent_commands = agent_commands.lock().unwrap();
            if agent_commands.commands.is_empty() {
                return;
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }

    {
        // kill remaining agents
        let mut agent_commands = agent_commands.lock().unwrap();
        let programs = agent_commands
            .commands
            .keys()
            .cloned()
            .collect::<vec::Vec<String>>();
        for program in programs {
            log::warn!("Killing agent: {}", program);
            if let Some(command) = agent_commands.commands.remove(&program) {
                command.kill().unwrap_or_else(|e| {
                    log::error!("Failed to kill agent: {} {}", program, e);
                });
            }
        }
    }
}

fn parse_stdout(line: &str) -> (&str, &str) {
    let (cmd, args) = line.split_once(" ").unwrap_or((line, ""));
    (cmd.trim(), args.trim())
}
