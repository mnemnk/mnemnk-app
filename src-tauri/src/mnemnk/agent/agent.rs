use std::collections::HashMap;

use anyhow::Result;
use serde_json::Value;
use tauri::{AppHandle, Manager, State};
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
    Starting,
    Run,
    Stopping,
}

pub trait Agent {
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

    fn input(&mut self, kind: String, value: Value) -> Result<()>;

    fn try_output(&mut self, kind: String, value: Value) -> Result<()> {
        let env = self.env();
        env.try_send_agent_out(self.id().to_string(), kind, value)
    }
}

pub struct AgentData {
    pub app: AppHandle,

    pub id: String,
    pub status: AgentStatus,
    pub def_name: String,
    pub config: Option<AgentConfig>,
}

pub type AgentConfigs = HashMap<String, AgentConfig>;
pub type AgentConfig = HashMap<String, Value>;

pub trait AsAgent {
    fn data(&self) -> &AgentData;

    fn mut_data(&mut self) -> &mut AgentData;

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

    fn input(&mut self, kind: String, value: Value) -> Result<()>;
}

impl<T: AsAgent> Agent for T {
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
        self.mut_data().status = AgentStatus::Starting;
        self.start()?;
        self.mut_data().status = AgentStatus::Run;
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.mut_data().status = AgentStatus::Stopping;
        self.stop()?;
        self.mut_data().status = AgentStatus::Init;
        Ok(())
    }

    fn input(&mut self, kind: String, value: Value) -> Result<()> {
        self.input(kind, value)
    }
}

pub trait AsyncAgent: Agent + Send + Sync {}
impl<T: Agent + Send + Sync> AsyncAgent for T {}

pub fn new_agent(
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

    // TODO: Prepare a mapping from kind to the corresponding new function and use it to create the Agent
    match def.kind.as_str() {
        "Command" => {
            let agent =
                super::command::CommandAgent::new(app, agent_id, def_name.to_string(), config)?;
            return Ok(Box::new(agent));
        }
        "BoardIn" => {
            let agent =
                super::board::BoardInAgent::new(app, agent_id, def_name.to_string(), config)?;
            return Ok(Box::new(agent));
        }
        "BoardOut" => {
            let agent =
                super::board::BoardOutAgent::new(app, agent_id, def_name.to_string(), config)?;
            return Ok(Box::new(agent));
        }
        "Database" => {
            let agent =
                super::builtin::DatabaseAgent::new(app, agent_id, def_name.to_string(), config)?;
            return Ok(Box::new(agent));
        }
        "JsonPath" => {
            let agent =
                super::builtin::JsonPathAgent::new(app, agent_id, def_name.to_string(), config)?;
            return Ok(Box::new(agent));
        }
        _ => return Err(AgentError::UnknownDefKind(def.kind.to_string()).into()),
    }
}

#[tauri::command]
pub fn start_agent_cmd(env: State<AgentEnv>, agent_id: String) -> Result<(), String> {
    env.start_agent(&agent_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn stop_agent_cmd(env: State<AgentEnv>, agent_id: String) -> Result<(), String> {
    env.stop_agent(&agent_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn set_agent_config_cmd(
    env: State<AgentEnv>,
    agent_id: String,
    config: AgentConfig,
) -> Result<(), String> {
    env.set_agent_config(&agent_id, config)
        .map_err(|e| e.to_string())
}
