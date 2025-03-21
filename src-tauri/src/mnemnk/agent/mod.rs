use anyhow::Result;
use serde_json::Value;
use tauri::AppHandle;
use tokio::sync::mpsc;

pub mod board;
pub mod bultin;
pub mod command;
pub mod config;
pub mod env;
pub mod flow;

use env::AgentEnv;

pub async fn init(app: &AppHandle) -> Result<()> {
    let (tx, rx) = mpsc::channel(4096);
    AgentEnv::init(app, tx.clone())?;

    flow::init_agent_flows(app)?;
    flow::sync_agent_flows(app);

    spawn_main_loop(app, rx);

    Ok(())
}

pub fn quit(app: &AppHandle) {
    command::quit(app);
}

#[derive(Clone, Debug)]
pub enum AgentMessage {
    Board {
        // original agent id
        agent: String,
        kind: String,
        value: Value,
    },
    Output {
        agent: String,
        kind: String,
        value: Value,
    },
}

fn spawn_main_loop(app: &AppHandle, rx: mpsc::Receiver<AgentMessage>) {
    // TODO: each message should have an origin field to prevent infinite loop
    let app_handle = app.clone();
    let mut rx = rx;
    tauri::async_runtime::spawn(async move {
        while let Some(message) = rx.recv().await {
            use AgentMessage::*;

            match message {
                Board { agent, kind, value } => {
                    board::board_message(&app_handle, agent, kind, value).await;
                }
                Output { agent, kind, value } => {
                    board::write_message(&app_handle, agent, kind, value).await;
                }
            }
        }
    });
}
