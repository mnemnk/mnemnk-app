use serde_json::Value;
use tauri::{AppHandle, Manager};
use tokio::sync::mpsc;

use super::env::AgentEnv;

#[derive(Clone, Debug)]
pub enum AgentMessage {
    BoardOut {
        // original agent id
        agent: String,
        kind: String,
        value: Value,
    },
    AgentOut {
        agent: String,
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
                BoardOut { agent, kind, value } => {
                    board_out(&app_handle, agent, kind, value).await;
                }
                AgentOut { agent, kind, value } => {
                    agent_out(&app_handle, agent, kind, value).await;
                }
            }
        }
    });
}

async fn board_out(app: &AppHandle, source_agent: String, kind: String, value: Value) {
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
            send_message_to(
                app,
                &env,
                source_agent.clone(),
                &sub_node,
                target_kind,
                value.clone(),
            )
        }
    }
}

// Processing .OUT $agent_id $kind $value
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

        if source_handle != kind && source_handle != "*" {
            // Skip if source_handle does not match with kind
            continue;
        }
        let kind = if target_handle == "*" {
            kind.clone()
        } else {
            // Use target_handle as kind if it is specified
            target_handle.clone()
        };

        send_message_to(
            app,
            &env,
            source_agent.clone(),
            &target_node,
            kind,
            value.clone(),
        )
    }
}

fn send_message_to(
    app: &AppHandle,
    env: &AgentEnv,
    source_agent: String,
    target_id: &str,
    kind: String,
    value: Value,
) {
    if let Some(target_node) = env.agents.lock().unwrap().get_mut(target_id) {
        target_node
            .input(app, source_agent, kind, value)
            .unwrap_or_else(|e| {
                log::error!("Failed to send message to {}: {}", target_id, e);
            });
    }
}

pub(super) fn send_board(env: &AgentEnv, agent: String, kind: String, value: Value) {
    let main_tx = env.tx.clone();
    main_tx
        .try_send(AgentMessage::BoardOut { agent, kind, value })
        .unwrap_or_else(|e| {
            log::error!("Failed to send message: {}", e);
        });
}
