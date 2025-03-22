use std::collections::HashMap;

use anyhow::Result;
use serde_json::Value;
use tauri::AppHandle;
use thiserror::Error;

use super::env::AgentEnv;

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Unknown agent def name: {0}")]
    UnknownDefName(String),

    #[error("Unknown agent def kind: {0}")]
    UnknownDefKind(String),
}

pub trait Agent {
    fn id(&self) -> &str;

    #[allow(unused)]
    fn def_name(&self) -> &str;

    fn config(&self) -> Option<&AgentConfig>;

    fn start(&mut self, app: &AppHandle) -> Result<()>;

    fn stop(&mut self, app: &AppHandle) -> Result<()>;

    fn update(&mut self, app: &AppHandle, config: Option<AgentConfig>) -> Result<()>;

    fn input(&self, app: &AppHandle, source: String, kind: String, value: Value) -> Result<()>;
}

pub struct AgentData {
    pub id: String,
    pub def_name: String,
    pub config: Option<AgentConfig>,
}

pub type AgentConfig = HashMap<String, Value>;

pub trait AsAgent {
    fn data(&self) -> &AgentData;

    fn mut_data(&mut self) -> &mut AgentData;

    fn start(&mut self, _app: &AppHandle) -> Result<()> {
        Ok(())
    }

    fn stop(&mut self, _app: &AppHandle) -> Result<()> {
        Ok(())
    }

    fn update(&mut self, _app: &AppHandle, config: Option<AgentConfig>) -> Result<()> {
        self.mut_data().config = config;
        Ok(())
    }

    fn input(&self, app: &AppHandle, source: String, kind: String, value: Value) -> Result<()>;
}

impl<T: AsAgent> Agent for T {
    fn id(&self) -> &str {
        self.data().id.as_str()
    }

    fn def_name(&self) -> &str {
        self.data().def_name.as_str()
    }

    fn config(&self) -> Option<&AgentConfig> {
        self.data().config.as_ref()
    }

    fn start(&mut self, app: &AppHandle) -> Result<()> {
        self.start(app)
    }

    fn stop(&mut self, app: &AppHandle) -> Result<()> {
        self.stop(app)
    }

    fn update(&mut self, app: &AppHandle, config: Option<AgentConfig>) -> Result<()> {
        self.update(app, config)
    }

    fn input(&self, app: &AppHandle, source: String, kind: String, value: Value) -> Result<()> {
        self.input(app, source, kind, value)
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
        "command" => {
            let agent = super::command::CommandAgent::new(agent_id, def_name.to_string(), config)?;
            return Ok(Box::new(agent));
        }
        "board" => {
            let agent = super::board::BoardAgent::new(agent_id, def_name.to_string(), config)?;
            return Ok(Box::new(agent));
        }
        "database" => {
            let agent = super::builtin::DatabaseAgent::new(agent_id, def_name.to_string(), config)?;
            return Ok(Box::new(agent));
        }
        _ => return Err(AgentError::UnknownDefKind(def.kind.to_string()).into()),
    }
}
