use anyhow::{Context as _, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;
use std::vec;
use tauri::{AppHandle, Manager};
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

use super::agent::{Agent, AgentConfig, AgentData, AsAgent};
use super::definition::agents_dir;
use super::env::AgentEnv;
use super::flow::{find_agent_node, AgentFlows};
use super::message::AgentMessage;

pub struct CommandAgent {
    data: AgentData,
}

impl AsAgent for CommandAgent {
    fn data(&self) -> &AgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AgentData {
        &mut self.data
    }

    fn start(&mut self, app: &AppHandle) -> Result<()> {
        start_agent(
            app,
            self.data.id.clone(),
            &self.data.def_name,
            self.data.config.clone(),
        )?;
        Ok(())
    }

    fn update(&mut self, app: &AppHandle, config: Option<AgentConfig>) -> Result<()> {
        self.data.config = config.clone();
        update_agent_config(app, &self.data.id)
    }

    fn stop(&mut self, app: &AppHandle) -> Result<()> {
        stop_agent(app, &self.data.id)
    }

    fn input(&self, app: &AppHandle, source: String, kind: String, value: Value) -> Result<()> {
        let env = app.state::<AgentEnv>();
        let mut env_commands = env.commands.lock().unwrap();

        let command = env_commands.get_mut(&self.data.id).ok_or_else(|| {
            log::error!("command for agent not found: {}", &self.data.id);
            anyhow::anyhow!("command for agent not found")
        })?;

        command
            .write(format!(".IN {} {} {}\n", source, kind, value.to_string()).as_bytes())
            .map_err(|e| {
                log::error!("Failed to write to {}: {}", &self.data.id, e);
                anyhow::anyhow!("Failed to write to agent")
            })?;
        Ok(())
    }
}

impl CommandAgent {
    pub fn new(id: String, def_name: String, config: Option<AgentConfig>) -> Result<Self> {
        Ok(Self {
            data: AgentData {
                id,
                def_name,
                config,
            },
        })
    }
}

pub fn start_agent(
    app: &AppHandle,
    agent_id: String,
    def_name: &str,
    config: Option<AgentConfig>,
) -> Result<CommandAgent> {
    let agent = CommandAgent {
        data: AgentData {
            id: agent_id.clone(),
            def_name: def_name.to_string(),
            config,
        },
    };

    let env = app.state::<AgentEnv>();

    let agent_path;
    {
        let env_defs = env.defs.lock().unwrap();
        if env_defs.contains_key(def_name) {
            let def = env_defs.get(def_name).unwrap();
            agent_path = def.path.clone();
        } else {
            log::error!("Agent {} not found", def_name);
            return Err(anyhow::anyhow!("Agent not found"));
        }
    }
    if agent_path.is_none() {
        log::error!("Agent path not found: {}", def_name);
        return Err(anyhow::anyhow!("Agent path not found"));
    }
    let path = agent_path.unwrap();

    let dir = agents_dir(app);
    if dir.is_none() {
        return Err(anyhow::anyhow!("Agents directory not found"));
    }
    let agent_dir = dir.unwrap().join(&def_name);

    let main_tx = env.tx.clone();

    log::info!("Starting agent: {} {}", def_name, agent_id);

    let sidecar_command = if agent.config().is_none() {
        app.shell().command(path).current_dir(agent_dir)
    } else {
        app.shell()
            .command(path)
            .args(vec![
                "-c",
                serde_json::to_string(&agent.config()).unwrap().as_str(),
            ])
            .current_dir(agent_dir)
    };

    let (mut rx, child) = sidecar_command.spawn().context("Failed to spawn sidecar")?;

    {
        let mut agent_commands = env.commands.lock().unwrap();
        agent_commands.insert(agent_id.to_string(), child);
    }

    let app_handle = app.clone();
    let agent_id = agent_id.to_string();
    let def_name = def_name.to_string();
    tauri::async_runtime::spawn(async move {
        // read events such as stdout
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line_bytes) => {
                    if line_bytes.is_empty() || line_bytes[0] != b'.' {
                        log::debug!(
                            "non-command stdout from {} {}: {:.200}",
                            &def_name,
                            &agent_id,
                            String::from_utf8_lossy(&line_bytes)
                        );
                        continue;
                    }

                    let line = String::from_utf8_lossy(&line_bytes);
                    let (cmd, args) = parse_stdout(&line);
                    match cmd {
                        ".OUT" => {
                            let kind_value = args.split_once(" ");
                            if kind_value.is_none() {
                                log::error!("Invalid OUT command: {:.40}", &line);
                                continue;
                            }
                            let (kind, value) = kind_value.unwrap();
                            let value = serde_json::from_str::<Value>(value);
                            if value.is_err() {
                                log::error!("Failed to parse value: {:.40}", &line);
                                continue;
                            }
                            main_tx
                                .send(AgentMessage::AgentOut {
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
                    log::debug!("stderr from {} {}: {:.200}", def_name, agent_id, line);
                }

                CommandEvent::Terminated(status) => {
                    log::info!(
                        "Agent exited: {} {} with status: {:?}",
                        def_name,
                        agent_id,
                        status
                    );
                    let env = app_handle.state::<AgentEnv>();
                    {
                        let mut commands = env.commands.lock().unwrap();
                        commands.remove(&agent_id);
                    }
                    break;
                }

                CommandEvent::Error(e) => {
                    log::error!("CommandEvent Error {} {}: {}", def_name, agent_id, e);
                }

                _ => {
                    log::error!(
                        "Unknown CommandEvent: {} {} {:?}",
                        def_name,
                        agent_id,
                        event
                    );
                }
            }
        }
    });

    Ok(agent)
}

pub fn stop_agent(app: &AppHandle, agent_id: &str) -> Result<()> {
    let env = app.state::<AgentEnv>();
    {
        let mut commands = env.commands.lock().unwrap();
        if let Some(child) = commands.get_mut(agent_id) {
            child.write(".QUIT\n".as_bytes()).unwrap_or_else(|e| {
                log::error!("Failed to write to {}: {}", agent_id, e);
            });
        }
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

    let env = app.state::<AgentEnv>();
    {
        let mut agent_commands = env.commands.lock().unwrap();
        if let Some(child) = agent_commands.get_mut(agent_id) {
            // the agent is already running, so update the config
            if let Err(e) = child.write(format!(".CONFIG {}\n", json_config.to_string()).as_bytes())
            {
                log::error!("Failed to set config to {}: {}", agent_id, e);
                return Err(anyhow::anyhow!("Failed to set config to agent"));
            }
        }
    }

    Ok(())
}

pub fn quit(app: &AppHandle) {
    let env = app.state::<AgentEnv>();
    {
        // send QUIT command to all agents
        let mut agent_commands = env.commands.lock().unwrap();
        let agent_ids = agent_commands.keys().cloned().collect::<vec::Vec<String>>();
        for agent_id in agent_ids {
            log::info!("Stopping agent: {}", agent_id);
            // we cannot use stop_agent here because it will also try to lock aget_commands.
            if let Some(child) = agent_commands.get_mut(&agent_id) {
                child.write(".QUIT\n".as_bytes()).unwrap_or_else(|e| {
                    log::error!("Failed to write to {}: {}", agent_id, e);
                });
            }
        }
    }

    // wait for all agents to exit
    for _ in 0..20 {
        {
            let agent_commands = env.commands.lock().unwrap();
            if agent_commands.is_empty() {
                return;
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }

    {
        // kill remaining agents
        let mut agent_commands = env.commands.lock().unwrap();
        let programs = agent_commands.keys().cloned().collect::<vec::Vec<String>>();
        for program in programs {
            log::warn!("Killing agent: {}", program);
            if let Some(command) = agent_commands.remove(&program) {
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
