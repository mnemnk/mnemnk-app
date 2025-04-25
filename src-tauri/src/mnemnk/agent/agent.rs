use anyhow::Result;
use tauri::{AppHandle, Manager, State};
use thiserror::Error;

use crate::mnemnk::settings;

use super::config::AgentConfig;
use super::data::AgentData;
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
    Stop,
}

pub enum AgentMessage {
    Input { ch: String, data: AgentData },
    Config { config: AgentConfig },
    Stop,
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

    fn id(&self) -> &str;

    fn status(&self) -> &AgentStatus;

    // #[allow(unused)]
    fn def_name(&self) -> &str;

    fn config(&self) -> Option<&AgentConfig>;

    fn set_config(&mut self, config: AgentConfig) -> Result<()>;

    fn start(&mut self) -> Result<()>;

    fn stop(&mut self) -> Result<()>;

    fn process(&mut self, ch: String, data: AgentData) -> Result<()>;

    // Utility methods

    fn env(&self) -> State<AgentEnv> {
        self.app().state::<AgentEnv>()
    }

    fn global_config(&self) -> Option<AgentConfig> {
        settings::get_agent_global_config(self.app(), self.def_name())
    }

    fn merged_config(&self) -> Option<AgentConfig> {
        let mut merged_config = self.global_config().unwrap_or_default();
        let config = self.config();
        if let Some(config) = config {
            for (key, value) in config {
                merged_config.set(key.clone(), value.clone());
            }
        }
        Some(merged_config)
    }
}

pub struct AsAgentData {
    pub app: AppHandle,

    pub id: String,
    pub status: AgentStatus,
    pub def_name: String,
    pub config: Option<AgentConfig>,
}

impl AsAgentData {
    pub fn new(app: AppHandle, id: String, def_name: String, config: Option<AgentConfig>) -> Self {
        Self {
            app,
            id,
            status: AgentStatus::Init,
            def_name,
            config,
        }
    }
}

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

    fn set_config(&mut self, _config: AgentConfig) -> Result<()> {
        Ok(())
    }

    fn start(&mut self) -> Result<()> {
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        Ok(())
    }

    fn process(&mut self, _ch: String, _data: AgentData) -> Result<()> {
        Ok(())
    }
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
        self.data().config.as_ref()
    }

    fn set_config(&mut self, config: AgentConfig) -> Result<()> {
        self.mut_data().config = Some(config.clone());
        self.set_config(config)
    }

    fn start(&mut self) -> Result<()> {
        self.mut_data().status = AgentStatus::Start;

        if let Err(e) = self.start() {
            self.env()
                .emit_error(self.id().to_string(), e.to_string())?;
            return Err(e);
        }

        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        self.mut_data().status = AgentStatus::Stop;
        self.stop()?;
        self.mut_data().status = AgentStatus::Init;
        Ok(())
    }

    fn process(&mut self, ch: String, data: AgentData) -> Result<()> {
        if let Err(e) = self.process(ch, data) {
            self.env()
                .emit_error(self.id().to_string(), e.to_string())?;
            return Err(e);
        }
        Ok(())
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
