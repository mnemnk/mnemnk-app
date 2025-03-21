use anyhow::Result;
use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager};

use super::{env::AgentEnv, AgentMessage};
use crate::mnemnk::store;

const EMIT_PUBLISH: &str = "mnemnk:write_board";

pub async fn board_message(app: &AppHandle, source_agent: String, kind: String, value: Value) {
    let subscribers;
    let env = app.state::<AgentEnv>();
    {
        let env_edges = env.edges.lock().unwrap();
        subscribers = env_edges.get(&kind).cloned();
    }
    let enabled_nodes;
    {
        let env_enabled_nodes = env.enabled_nodes.lock().unwrap();
        enabled_nodes = env_enabled_nodes.clone();
    }
    if let Some(subscribers) = subscribers {
        for subscriber in subscribers {
            let (sub_node, _src_handle, sub_handle) = subscriber;
            if !enabled_nodes.contains(&sub_node) {
                continue;
            }

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
            .await;
        }
    }
}

// Processing .WRITE $agent_id $kind $value
pub async fn write_message(app: &AppHandle, source_agent: String, kind: String, value: Value) {
    // Retrieve targets and enabled_nodes from agent_commands
    // Nodes that are not enabled should have been removed from targets in sync_agent_flow,
    // but consider the possibility that enabled_nodes may have changed since then.

    let env = app.state::<AgentEnv>();

    let targets;
    {
        let env_edges = env.edges.lock().unwrap();
        targets = env_edges.get(&source_agent).cloned();
    }

    let enabled_nodes;
    {
        let env_enabled_nodes = env.enabled_nodes.lock().unwrap();
        enabled_nodes = env_enabled_nodes.clone();
    }

    if targets.is_none() {
        return;
    }

    for target in targets.unwrap() {
        // In reality, targets are normalized to id/source_handle/target_handle in sync_agent_flows,
        // so unwrap should not fail.

        let (target_node, source_handle, target_handle) = target;
        if !enabled_nodes.contains(&target_node) {
            continue;
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
        .await;
    }
}

async fn send_message_to(
    app: &AppHandle,
    env: &AgentEnv,
    source_agent: String,
    target_node: &str,
    kind: String,
    value: Value,
) {
    if target_node.starts_with("$") {
        if target_node.starts_with("$board_") {
            if let Err(e) =
                write_message_to_board(app, env, source_agent, target_node.to_string(), kind, value)
                    .await
            {
                log::error!("Failed to write board: {}", e);
            };
        } else if target_node.starts_with("$database_") {
            if let Err(e) = store::store(&app, source_agent, kind, value).await {
                log::error!("Failed to store: {}", e);
            }
        } else {
            log::error!("Unknown target: {}", target_node);
        }
    } else {
        write_message_to_agent(env, &source_agent, &target_node, &kind, &value);
    }
}

fn write_message_to_agent(env: &AgentEnv, source: &str, target: &str, kind: &str, value: &Value) {
    let mut env_commands = env.commands.lock().unwrap();
    if let Some(command) = env_commands.get_mut(target) {
        command
            .write(format!(".IN {} {} {}\n", source, kind, value.to_string()).as_bytes())
            .unwrap_or_else(|e| {
                log::error!("Failed to write to {}: {}", target, e);
            });
    } else {
        log::error!("Agent not found: {}", target);
    }
}

#[derive(Clone, Debug, Serialize)]
struct WriteBoardMessage {
    agent: String,
    kind: String,
    value: Value,
}

async fn write_message_to_board(
    app: &AppHandle,
    env: &AgentEnv,
    source_agent: String,
    target_board: String,
    kind: String,
    value: Value,
) -> Result<()> {
    let board_name: String;
    {
        let board_names = env.board_names.lock().unwrap();
        if let Some(bn) = board_names.get(&target_board) {
            board_name = if bn == "" || bn == "*" {
                kind.clone()
            } else {
                bn.clone()
            };
        } else {
            board_name = kind.clone();
        }
    }
    {
        let mut board_values = env.board_values.lock().unwrap();
        board_values.insert(board_name.clone(), value.clone());
    }

    send_board(env, source_agent.clone(), board_name.clone(), value.clone()).await;

    // remove image from the value. it's too big to send to frontend
    let mut value = value;
    if value.get("image").is_some() {
        value.as_object_mut().unwrap().remove("image");
    }

    // emit the message to frontend
    let message = WriteBoardMessage {
        agent: source_agent,
        kind: board_name,
        value,
    };
    app.emit(EMIT_PUBLISH, Some(message))?;

    Ok(())
}

async fn send_board(env: &AgentEnv, agent: String, kind: String, value: Value) {
    let main_tx = env.tx.clone();
    main_tx
        .send(AgentMessage::Board { agent, kind, value })
        .await
        .unwrap_or_else(|e| {
            log::error!("Failed to send message: {}", e);
        });
}
