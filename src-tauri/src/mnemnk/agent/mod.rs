use anyhow::Result;
use serde_json::Value;
use tauri::AppHandle;
use tokio::sync::mpsc;

pub mod board;
pub mod command;
pub mod config;
pub mod flow;

pub async fn init(app: &AppHandle) -> Result<()> {
    config::init_agent_configs(app)?;

    let (tx, rx) = mpsc::channel(128);

    command::init_agent_commands(app, tx.clone())?;

    board::init_agent_boards(app);

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
    Write {
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
                Write { agent, kind, value } => {
                    board::write_message(&app_handle, agent, kind, value).await;
                }
            }
        }
    });
}
