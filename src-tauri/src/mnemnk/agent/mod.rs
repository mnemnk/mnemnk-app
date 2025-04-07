use anyhow::Result;
use tauri::AppHandle;
use tokio::sync::mpsc;

pub mod agent;
pub mod env;
pub mod flow;

mod builtins;
mod command;
mod definition;
mod message;

pub use agent::{Agent, AgentConfig, AgentConfigs, AgentData, AgentStatus, AsAgent};
pub use definition::{
    AgentConfigEntry, AgentDefinition, AgentDefinitionError, AgentDefinitions,
    AgentDisplayConfigEntry,
};
pub use env::AgentEnv;
pub use message::try_send_store;

pub async fn init(app: &AppHandle) -> Result<()> {
    let (tx, rx) = mpsc::channel(4096);
    AgentEnv::init(app, tx.clone())?;

    flow::init_agent_flows(app)?;

    message::spawn_main_loop(app, rx);

    Ok(())
}

pub fn quit(app: &AppHandle) {
    command::quit(app);
}
