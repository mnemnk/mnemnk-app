use std::collections::HashMap;

use anyhow::Result;
use serde_json::Value;
use tauri::AppHandle;
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
    fn id(&self) -> &str;

    fn status(&self) -> &AgentStatus;

    #[allow(unused)]
    fn def_name(&self) -> &str;

    fn config(&self, app: &AppHandle) -> Option<&AgentConfig>;

    fn set_config(&mut self, app: &AppHandle, config: Option<AgentConfig>) -> Result<()>;

    fn global_config(&self, app: &AppHandle) -> Option<AgentConfig> {
        settings::get_agent_config(app, self.def_name())
    }

    fn merged_config(&self, app: &AppHandle) -> Option<AgentConfig> {
        let mut merged_config = self.global_config(app).unwrap_or_default();
        let config = self.config(app);
        if let Some(config) = config {
            for (key, value) in config.iter() {
                merged_config.insert(key.clone(), value.clone());
            }
        }
        Some(merged_config)
    }

    fn start(&mut self, app: &AppHandle) -> Result<()>;

    fn stop(&mut self, app: &AppHandle) -> Result<()>;

    fn input(&mut self, app: &AppHandle, kind: String, value: Value) -> Result<()>;
}

pub struct AgentData {
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

    fn config(&self, _app: &AppHandle) -> Option<&AgentConfig> {
        self.data().config.as_ref()
    }

    fn set_config(&mut self, _app: &AppHandle, config: Option<AgentConfig>) -> Result<()> {
        self.mut_data().config = config;
        Ok(())
    }

    fn start(&mut self, _app: &AppHandle) -> Result<()> {
        Ok(())
    }

    fn stop(&mut self, _app: &AppHandle) -> Result<()> {
        Ok(())
    }

    fn input(&mut self, app: &AppHandle, kind: String, value: Value) -> Result<()>;
}

impl<T: AsAgent> Agent for T {
    fn id(&self) -> &str {
        self.data().id.as_str()
    }

    fn status(&self) -> &AgentStatus {
        &self.data().status
    }

    fn def_name(&self) -> &str {
        self.data().def_name.as_str()
    }

    fn config(&self, app: &AppHandle) -> Option<&AgentConfig> {
        self.config(app)
    }

    fn set_config(&mut self, app: &AppHandle, config: Option<AgentConfig>) -> Result<()> {
        self.set_config(app, config)
    }

    fn start(&mut self, app: &AppHandle) -> Result<()> {
        self.mut_data().status = AgentStatus::Starting;
        self.start(app)?;
        self.mut_data().status = AgentStatus::Run;
        Ok(())
    }

    fn stop(&mut self, app: &AppHandle) -> Result<()> {
        self.mut_data().status = AgentStatus::Stopping;
        self.stop(app)?;
        self.mut_data().status = AgentStatus::Init;
        Ok(())
    }

    fn input(&mut self, app: &AppHandle, kind: String, value: Value) -> Result<()> {
        self.input(app, kind, value)
    }
}

pub trait AsyncAgent: Agent + Send + Sync {}
impl<T: Agent + Send + Sync> AsyncAgent for T {}

pub fn new_agent(
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
            let agent = super::command::CommandAgent::new(agent_id, def_name.to_string(), config)?;
            return Ok(Box::new(agent));
        }
        "BoardIn" => {
            let agent = super::board::BoardInAgent::new(agent_id, def_name.to_string(), config)?;
            return Ok(Box::new(agent));
        }
        "BoardOut" => {
            let agent = super::board::BoardOutAgent::new(agent_id, def_name.to_string(), config)?;
            return Ok(Box::new(agent));
        }
        "Database" => {
            let agent = super::builtin::DatabaseAgent::new(agent_id, def_name.to_string(), config)?;
            return Ok(Box::new(agent));
        }
        "JsonPath" => {
            let agent = super::builtin::JsonPathAgent::new(agent_id, def_name.to_string(), config)?;
            return Ok(Box::new(agent));
        }
        _ => return Err(AgentError::UnknownDefKind(def.kind.to_string()).into()),
    }
}
