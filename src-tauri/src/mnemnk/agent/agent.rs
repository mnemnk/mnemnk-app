use std::collections::HashMap;

use anyhow::{Context as _, Result};
use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, State};
use thiserror::Error;

use crate::mnemnk::settings;

use super::env::AgentEnv;

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Unknown agent def name: {0}")]
    UnknownDefName(String),

    #[error("Unknown agent def kind: {0}")]
    UnknownDefKind(String),
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum AgentStatus {
    #[default]
    Init,
    Start,
    Run,
    Stop,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentData {
    pub kind: String,
    pub value: Value,
}

pub trait Agent {
    fn new(
        app: AppHandle,
        id: String,
        def_name: String,
        config: Option<AgentConfig>,
    ) -> Result<Self>
    where
        Self: Sized;

    fn app(&self) -> &AppHandle;

    fn env(&self) -> State<AgentEnv> {
        self.app().state::<AgentEnv>()
    }

    fn id(&self) -> &str;

    fn status(&self) -> &AgentStatus;

    #[allow(unused)]
    fn def_name(&self) -> &str;

    fn config(&self) -> Option<&AgentConfig>;

    fn set_config(&mut self, config: AgentConfig) -> Result<()>;

    fn global_config(&self) -> Option<AgentConfig> {
        settings::get_agent_global_config(self.app(), self.def_name())
    }

    fn merged_config(&self) -> Option<AgentConfig> {
        let mut merged_config = self.global_config().unwrap_or_default();
        let config = self.config();
        if let Some(config) = config {
            for (key, value) in config.iter() {
                merged_config.insert(key.clone(), value.clone());
            }
        }
        Some(merged_config)
    }

    fn start(&mut self) -> Result<()>;

    fn stop(&mut self) -> Result<()>;

    fn input(&mut self, ch: String, data: AgentData) -> Result<()>;

    fn try_output(&self, ch: String, data: AgentData) -> Result<()> {
        let env = self.env();
        env.try_send_agent_out(self.id().to_string(), ch, data)
    }

    fn emit_display(&self, key: String, value: Value) -> Result<()> {
        let message = DisplayMessage {
            agent_id: self.id().to_string(),
            key,
            value,
        };
        self.app()
            .emit(EMIT_DISPLAY, message)
            .context("Failed to emit display message")?;
        Ok(())
    }

    fn emit_error(&self, message: String) -> Result<()> {
        let error_message = ErrorMessage {
            agent_id: self.id().to_string(),
            message,
        };
        self.app()
            .emit(EMIT_ERROR, error_message)
            .context("Failed to emit error message")?;
        Ok(())
    }
}

const EMIT_DISPLAY: &str = "mnemnk:display";
const EMIT_ERROR: &str = "mnemnk:error";

#[derive(Clone, Serialize)]
struct DisplayMessage {
    agent_id: String,
    key: String,
    value: Value,
}

#[derive(Clone, Serialize)]
struct ErrorMessage {
    agent_id: String,
    message: String,
}

pub fn emit_error(app: &AppHandle, agent_id: String, message: String) -> Result<()> {
    let error_message = ErrorMessage { agent_id, message };
    app.emit(EMIT_ERROR, error_message)
        .context("Failed to emit error message")?;
    Ok(())
}

pub struct AsAgentData {
    pub app: AppHandle,

    pub id: String,
    pub status: AgentStatus,
    pub def_name: String,
    pub config: Option<AgentConfig>,
}

pub type AgentConfigs = HashMap<String, AgentConfig>;
pub type AgentConfig = HashMap<String, Value>;

pub trait AsAgent {
    fn new(
        app: AppHandle,
        id: String,
        def_name: String,
        config: Option<AgentConfig>,
    ) -> Result<Self>
    where
        Self: Sized;

    fn data(&self) -> &AsAgentData;

    fn mut_data(&mut self) -> &mut AsAgentData;

    fn config(&self) -> Option<&AgentConfig> {
        self.data().config.as_ref()
    }

    fn set_config(&mut self, config: AgentConfig) -> Result<()> {
        self.mut_data().config = Some(config);
        Ok(())
    }

    fn start(&mut self) -> Result<()> {
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    fn input(&mut self, ch: String, data: AgentData) -> Result<()>;
}

impl<T: AsAgent> Agent for T {
    fn new(
        app: AppHandle,
        id: String,
        def_name: String,
        config: Option<AgentConfig>,
    ) -> Result<Self> {
        let mut agent = T::new(app, id, def_name, config)?;
        agent.mut_data().status = AgentStatus::Init;
        Ok(agent)
    }

    fn app(&self) -> &AppHandle {
        &self.data().app
    }

    fn id(&self) -> &str {
        self.data().id.as_str()
    }

    fn status(&self) -> &AgentStatus {
        &self.data().status
    }

    fn def_name(&self) -> &str {
        self.data().def_name.as_str()
    }

    fn config(&self) -> Option<&AgentConfig> {
        self.config()
    }

    fn set_config(&mut self, config: AgentConfig) -> Result<()> {
        self.set_config(config)
    }

    fn start(&mut self) -> Result<()> {
        self.mut_data().status = AgentStatus::Start;
        self.start()?;
        self.mut_data().status = AgentStatus::Run;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.mut_data().status = AgentStatus::Stop;
        self.stop()?;
        self.mut_data().status = AgentStatus::Init;
        Ok(())
    }

    fn input(&mut self, ch: String, data: AgentData) -> Result<()> {
        self.input(ch, data)
    }
}

pub trait AsyncAgent: Agent + Send + Sync + 'static {}
impl<T: Agent + Send + Sync + 'static> AsyncAgent for T {}

pub fn new_boxed<T: AsyncAgent>(
    app: AppHandle,
    id: String,
    def_name: String,
    config: Option<AgentConfig>,
) -> Result<Box<dyn AsyncAgent>> {
    Ok(Box::new(T::new(app, id, def_name, config)?))
}

pub fn agent_new(
    app: AppHandle,
    env: &AgentEnv,
    agent_id: String,
    def_name: &str,
    config: Option<AgentConfig>,
) -> Result<Box<dyn AsyncAgent>> {
    let def;
    {
        let env_defs = env.defs.lock().unwrap();
        def = env_defs
            .get(def_name)
            .ok_or_else(|| AgentError::UnknownDefName(def_name.to_string()))?
            .clone();
    }

    if let Some(new_boxed) = def.new_boxed {
        return new_boxed(app, agent_id, def_name.to_string(), config);
    }

    match def.kind.as_str() {
        "Command" => {
            return new_boxed::<super::builtins::CommandAgent>(
                app,
                agent_id,
                def_name.to_string(),
                config,
            );
        }
        _ => return Err(AgentError::UnknownDefKind(def.kind.to_string()).into()),
    }
}
