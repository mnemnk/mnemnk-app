use anyhow::{Context as _, Result};
use serde_json::Value;
use tauri::{AppHandle, Manager};
use tokio::sync::mpsc;

use crate::mnemnk::store;

use super::env::AgentEnv;

#[derive(Clone, Debug)]
pub enum AgentMessage {
    AgentOut {
        agent: String,
        kind: String,
        value: Value,
    },
    BoardOut {
        kind: String,
        value: Value,
    },
    Store {
        kind: String,
        value: Value,
    },
}

pub(super) fn spawn_main_loop(app: &AppHandle, rx: mpsc::Receiver<AgentMessage>) {
    // TODO: each message should have an origin field to prevent infinite loop
    let app_handle = app.clone();
    let mut rx = rx;
    tauri::async_runtime::spawn(async move {
        while let Some(message) = rx.recv().await {
            use AgentMessage::*;

            match message {
                AgentOut { agent, kind, value } => {
                    agent_out(&app_handle, agent, kind, value).await;
                }
                BoardOut { kind, value } => {
                    board_out(&app_handle, kind, value).await;
                }
                Store { kind, value } => {
                    store(&app_handle, kind, value).await;
                }
            }
        }
    });
}

pub async fn send_agent_out(
    env: &AgentEnv,
    agent: String,
    kind: String,
    value: Value,
) -> Result<()> {
    let main_tx = env.tx.clone();
    main_tx
        .send(AgentMessage::AgentOut { agent, kind, value })
        .await
        .context("Failed to send AgentOut message")
}

pub fn try_send_agent_out(env: &AgentEnv, agent: String, kind: String, value: Value) -> Result<()> {
    let main_tx = env.tx.clone();
    main_tx
        .try_send(AgentMessage::AgentOut { agent, kind, value })
        .context("Failed to try_send AgentOut message")
}

pub fn try_send_board_out(env: &AgentEnv, kind: String, value: Value) -> Result<()> {
    let main_tx = env.tx.clone();
    main_tx
        .try_send(AgentMessage::BoardOut { kind, value })
        .context("Failed to try_send BoardOut message")
}

pub fn try_send_store(app: &AppHandle, kind: String, value: Value) -> Result<()> {
    let env = app.state::<AgentEnv>();
    env.tx
        .try_send(AgentMessage::Store { kind, value })
        .context("Failed to send Store message")
}

// Processing AgentOut message
async fn agent_out(app: &AppHandle, source_agent: String, kind: String, value: Value) {
    // Retrieve targets and enabled_nodes from agent_commands
    // Nodes that are not enabled should have been removed from targets in sync_agent_flow,
    // but consider the possibility that enabled_nodes may have changed since then.

    let env = app.state::<AgentEnv>();

    let targets;
    {
        let env_edges = env.edges.lock().unwrap();
        targets = env_edges.get(&source_agent).cloned();
    }

    if targets.is_none() {
        return;
    }

    for target in targets.unwrap() {
        // In reality, targets are normalized to id/source_handle/target_handle in sync_agent_flows,
        // so unwrap should not fail.

        let (target_node, source_handle, target_handle) = target;
        {
            let env_nodes = env.agents.lock().unwrap();
            if !env_nodes.contains_key(&target_node) {
                continue;
            }
        }

        if source_handle != kind && (!source_handle.is_empty() && source_handle != "*") {
            // Skip if source_handle does not match with kind
            continue;
        }
        let kind = if target_handle.is_empty() || target_handle == "*" {
            kind.clone()
        } else {
            // Use target_handle as kind if it is specified
            target_handle.clone()
        };

        send_message_to(app, &env, &target_node, kind, value.clone())
    }
}

async fn board_out(app: &AppHandle, kind: String, value: Value) {
    let env = app.state::<AgentEnv>();

    let board_nodes;
    {
        let env_board_nodes = env.board_nodes.lock().unwrap();
        board_nodes = env_board_nodes.get(&kind).cloned();
    }
    let Some(board_nodes) = board_nodes else {
        // board not found
        return;
    };

    for node in board_nodes {
        let edges;
        {
            let env_edges = env.edges.lock().unwrap();
            edges = env_edges.get(&node).cloned();
        }
        let Some(edges) = edges else {
            // edges not found
            continue;
        };
        for edge in edges {
            let (sub_node, _src_handle, sub_handle) = edge;
            let target_kind = if sub_handle == "*" {
                kind.clone()
            } else {
                sub_handle
            };
            send_message_to(app, &env, &sub_node, target_kind, value.clone())
        }
    }
}

fn send_message_to(app: &AppHandle, env: &AgentEnv, target_id: &str, kind: String, value: Value) {
    if let Some(target_node) = env.agents.lock().unwrap().get_mut(target_id) {
        target_node.input(app, kind, value).unwrap_or_else(|e| {
            log::error!("Failed to send message to {}: {}", target_id, e);
        });
    }
}

async fn store(app_handle: &AppHandle, kind: String, value: Value) {
    store::store(app_handle, kind, value)
        .await
        .unwrap_or_else(|e| {
            log::error!("Failed to store value: {}", e);
        });
}
