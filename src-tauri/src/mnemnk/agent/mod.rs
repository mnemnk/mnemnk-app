use anyhow::Result;
use tauri::AppHandle;
use tokio::sync::mpsc;

pub mod env;
pub mod flow;

mod agent;
mod board;
mod builtin;
mod command;
mod definition;
mod message;

use env::AgentEnv;

pub async fn init(app: &AppHandle) -> Result<()> {
    let (tx, rx) = mpsc::channel(4096);
    AgentEnv::init(app, tx.clone())?;

    flow::init_agent_flows(app)?;
    flow::sync_agent_flows(app);

    message::spawn_main_loop(app, rx);

    Ok(())
}

pub fn quit(app: &AppHandle) {
    command::quit(app);
}
