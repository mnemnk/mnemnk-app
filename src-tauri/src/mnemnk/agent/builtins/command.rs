use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::vec;
use tauri::{AppHandle, Manager};
use tauri_plugin_shell::process::CommandEvent;
use tauri_plugin_shell::ShellExt;

use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentContext, AgentData, AgentDefinition, AgentDefinitionError, AgentEnv,
    AsAgent, AsAgentData,
};

pub struct CommandAgent {
    data: AsAgentData,
}

impl AsAgent for CommandAgent {
    fn new(
        app: AppHandle,
        id: String,
        def_name: String,
        config: Option<AgentConfig>,
    ) -> Result<Self> {
        CommandAgent::new(app, id, def_name, config)
    }

    fn data(&self) -> &AsAgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AsAgentData {
        &mut self.data
    }

    fn start(&mut self) -> Result<()> {
        let env = self.env();
        let agent_id = self.id();
        let def_name = self.def_name();

        // get agent command from agent env
        let agent_cmd;
        let agent_args;
        let agent_dir;
        {
            let env_defs = env.defs.lock().unwrap();
            if env_defs.contains_key(def_name) {
                let def = env_defs.get(def_name).unwrap();
                let def_command = def
                    .command
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Agent has no command"))?;
                agent_cmd = def_command.cmd.clone();
                agent_args = def_command.args.clone();
                agent_dir = def_command
                    .dir
                    .clone()
                    .context(format!("Agent path not found: {}", def_name))?;
            } else {
                log::error!("Agent {} not found", def_name);
                return Err(anyhow::anyhow!("Agent not found"));
            }
        }
        if agent_cmd.is_empty() {
            log::error!("Agent command.cmd not found: {}", def_name);
            return Err(anyhow::anyhow!("Agent command.cmd not found"));
        }

        log::info!("Starting agent: {} {}", def_name, agent_id);

        // prepare args
        let mut args = if agent_args.is_none() {
            vec![]
        } else {
            agent_args.unwrap().clone().into_iter().collect()
        };
        let config = self.merged_config();
        if let Some(config) = config {
            args.push("-c".to_string());
            args.push(serde_json::to_string(&config).unwrap());
        }

        // prepare sidecar command
        let sidecar_command = if args.is_empty() {
            self.app().shell().command(agent_cmd).current_dir(agent_dir)
        } else {
            self.app()
                .shell()
                .command(agent_cmd)
                .args(args)
                .current_dir(agent_dir)
        };

        // spawn the sidecar command
        let (mut rx, child) = sidecar_command.spawn().context("Failed to spawn sidecar")?;

        {
            let mut agent_commands = env.commands.lock().unwrap();
            agent_commands.insert(agent_id.to_string(), child);
        }

        let app_handle = self.app().clone();
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
                            ".OUT" => match parse_out_args(args) {
                                Ok((ctx, ch, data)) => {
                                    let new_ctx = ctx.with_ch(ch);
                                    let env = app_handle.state::<AgentEnv>();
                                    env.send_agent_out(agent_id.clone(), new_ctx, data)
                                        .await
                                        .unwrap_or_else(|e| {
                                            log::error!("Failed to send agent out: {}", e);
                                        });
                                }
                                Err(e) => {
                                    log::error!("Failed to parse OUT command: {}", e);
                                }
                            },
                            _ => {
                                log::error!("Unknown command: {} {}", agent_id, cmd);
                            }
                        }
                    }

                    CommandEvent::Stderr(line_bytes) => {
                        let line = String::from_utf8_lossy(&line_bytes);
                        log::debug!("stderr from {} {}: {:}", def_name, agent_id, line);
                        let env = app_handle.state::<AgentEnv>();
                        env.emit_error(agent_id.clone(), line.to_string())
                            .unwrap_or_else(|e| log::error!("Failed to emit error: {}", e));
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
                        rx.close();
                        // TODO: Emit an event to the frontend indicating the agent has stopped
                        return;
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
        Ok(())
    }

    fn set_config(&mut self, _config: AgentConfig) -> Result<()> {
        let merged_config = self.merged_config();
        if merged_config.is_none() {
            return Ok(());
        }
        let json_config =
            serde_json::to_value(merged_config.unwrap()).context("Failed to serialize config")?;
        let agent_id = &self.data.id;
        let env = self.env();
        let mut agent_commands = env.commands.lock().unwrap();
        if let Some(child) = agent_commands.get_mut(agent_id) {
            // the agent is already running, so update the config
            if let Err(e) = child.write(format!(".CONFIG {}\n", json_config.to_string()).as_bytes())
            {
                log::error!("Failed to set config to {}: {}", agent_id, e);
                return Err(anyhow::anyhow!("Failed to set config to agent"));
            }
        }
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        let agent_id = &self.data.id;
        let env = self.env();
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

    fn process(&mut self, ctx: AgentContext, data: AgentData) -> Result<()> {
        #[derive(Debug, Serialize)]
        struct InData {
            ctx: AgentContext,
            data: AgentData,
        }

        let in_data = InData { ctx, data };
        let in_json = serde_json::to_string(&in_data).context("Failed to serialize input data")?;

        let env = self.env();
        let mut env_commands = env.commands.lock().unwrap();
        let command = env_commands
            .get_mut(self.id())
            .context("command not found")?;
        command
            .write(format!(".IN {}\n", in_json).as_bytes())
            .context("Failed to write to command")
    }
}

impl CommandAgent {
    pub fn new(
        app: AppHandle,
        id: String,
        def_name: String,
        config: Option<AgentConfig>,
    ) -> Result<Self> {
        Ok(Self {
            data: AsAgentData::new(app, id, def_name, config),
        })
    }

    pub fn read_def(
        def: &mut AgentDefinition,
        agent_dir: &PathBuf,
    ) -> Result<(), AgentDefinitionError> {
        let command = def.command.as_mut().ok_or_else(|| {
            AgentDefinitionError::MissingEntry(def.name.clone(), "command".into())
        })?;

        // set agent dir
        command.dir = agent_dir.to_string_lossy().to_string().into();

        if command.cmd.is_empty() {
            return Err(AgentDefinitionError::MissingEntry(
                def.name.clone(),
                "command.cmd".into(),
            ));
        }
        if command.cmd.starts_with("./") || command.cmd.starts_with(".\\") {
            // relative path
            let command_path = agent_dir
                .join(&command.cmd[2..])
                .with_extension(std::env::consts::EXE_EXTENSION);
            if !command_path.exists() {
                log::error!(
                    "Command not found: {} for {}",
                    command_path.display(),
                    def.name
                );
                return Err(AgentDefinitionError::InvalidEntry(
                    def.name.clone(),
                    "command.cmd".into(),
                ));
            }
            command.cmd = command_path.to_string_lossy().to_string();
        }
        Ok(())
    }
}

fn parse_stdout(line: &str) -> (&str, &str) {
    let (cmd, args) = line.split_once(" ").unwrap_or((line, ""));
    (cmd.trim(), args.trim())
}

#[derive(Debug, Deserialize)]
struct OutArg {
    ctx: Option<AgentContext>,
    ch: String,
    data: AgentData,
}

fn parse_out_args(args: &str) -> Result<(AgentContext, String, AgentData)> {
    let data: OutArg = serde_json::from_str(args).context("Failed to parse OUT command")?;
    let ctx = data.ctx.unwrap_or_default();
    return Ok((ctx, data.ch, data.data));
}
