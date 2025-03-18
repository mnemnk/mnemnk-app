use std::collections::HashMap;
use std::sync::Mutex;

use anyhow::Result;
use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, State};

use super::command::AgentCommands;
use super::AgentMessage;
use crate::mnemnk::store;

const EMIT_PUBLISH: &str = "mnemnk:write_board";

pub struct AgentBoards {
    // node id -> board name
    pub board_names: HashMap<String, String>,

    // board name -> value
    board_values: HashMap<String, Value>,

    // board name -> subscribers (node ids)
    pub subscribers: HashMap<String, Vec<String>>,
}

pub(super) fn init_agent_boards(app: &AppHandle) {
    let agent_boards = AgentBoards {
        board_names: HashMap::new(),
        board_values: HashMap::new(),
        subscribers: HashMap::new(),
    };
    app.manage(Mutex::new(agent_boards));
}

pub async fn board_message(app: &AppHandle, source_agent: String, kind: String, value: Value) {
    let subscribers;
    let agent_boards = app.state::<Mutex<AgentBoards>>();
    {
        let agent_board = agent_boards.lock().unwrap();
        subscribers = agent_board.subscribers.get(&kind).cloned();
    }
    let enabled_nodes;
    let agent_commands = app.state::<Mutex<AgentCommands>>();
    {
        let agent_commands = agent_commands.lock().unwrap();
        enabled_nodes = agent_commands.enabled_nodes.clone();
    }
    if let Some(subscribers) = subscribers {
        for subscriber in subscribers {
            let (sub_node, sub_handle) = subscriber.split_once("/").unwrap_or((&subscriber, "*"));
            if !enabled_nodes.contains(sub_node) {
                continue;
            }

            let kind = if sub_handle == "*" {
                kind.clone()
            } else {
                sub_handle.to_string()
            };

            if sub_node.starts_with("$") {
                if sub_node.starts_with("$board_") {
                    if let Err(e) = write_message_to_board(
                        &app,
                        source_agent.clone(),
                        sub_node.to_string(),
                        kind,
                        value.clone(),
                    )
                    .await
                    {
                        log::error!("Failed to write board: {}", e);
                    };
                } else if sub_node.starts_with("$database_") {
                    if let Err(e) =
                        store::store(&app, source_agent.clone(), kind, value.clone()).await
                    {
                        log::error!("Failed to store: {}", e);
                    }
                } else {
                    log::error!("Unknown subscriber: {}", sub_node);
                }
            } else {
                write_message_to_agent(&agent_commands, &source_agent, &sub_node, &kind, &value);
            }
        }
    }
}

// Processing .WRITE $agent_id $kind $value
pub async fn write_message(app: &AppHandle, source_agent: String, kind: String, value: Value) {
    // Retrieve targets and enabled_nodes from agent_commands
    // Nodes that are not enabled should have been removed from targets in sync_agent_flow,
    // but consider the possibility that enabled_nodes may have changed since then.
    let targets;
    let enabled_nodes;
    let agent_commands = app.state::<Mutex<AgentCommands>>();
    {
        let agent_commands = agent_commands.lock().unwrap();
        targets = agent_commands.edges.get(&source_agent).cloned();
        enabled_nodes = agent_commands.enabled_nodes.clone();
    }

    if targets.is_none() {
        return;
    }

    for target in targets.unwrap() {
        // In reality, targets are normalized to id/source_handle/target_handle in sync_agent_flows,
        // so unwrap should not fail.
        let (target_node, handles) = target.split_once("/").unwrap_or((&target, "*/*"));
        if !enabled_nodes.contains(target_node) {
            continue;
        }

        let (source_handle, target_handle) = handles.split_once("/").unwrap_or(("*", "*"));
        if source_handle != kind && source_handle != "*" {
            // Skip if source_handle does not match with kind
            continue;
        }
        let kind = if target_handle == "*" {
            kind.clone()
        } else {
            // Use target_handle as kind if it is specified
            target_handle.to_string()
        };

        if target_node.starts_with("$") {
            if target_node.starts_with("$board_") {
                if let Err(e) = write_message_to_board(
                    &app,
                    source_agent.clone(),
                    target_node.to_string(),
                    kind.clone(),
                    value.clone(),
                )
                .await
                {
                    log::error!("Failed to write board: {}", e);
                };
            } else if target_node.starts_with("$database_") {
                if let Err(e) =
                    store::store(&app, source_agent.clone(), kind.clone(), value.clone()).await
                {
                    log::error!("Failed to store: {}", e);
                }
            } else {
                log::error!("Unknown target: {}", target_node);
            }
        } else {
            write_message_to_agent(&agent_commands, &source_agent, &target_node, &kind, &value);
        }
    }
}

fn write_message_to_agent(
    agent_commands: &State<Mutex<AgentCommands>>,
    source: &str,
    target: &str,
    kind: &str,
    value: &Value,
) {
    let mut agent_commands = agent_commands.lock().unwrap();
    if let Some(command) = agent_commands.commands.get_mut(target) {
        command
            .write(format!(".PUBLISH {} {} {}\n", source, kind, value.to_string()).as_bytes())
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
    source_agent: String,
    target_board: String,
    kind: String,
    value: Value,
) -> Result<()> {
    // update board value
    let board_name;
    let agent_boards = app.state::<Mutex<AgentBoards>>();
    {
        let mut agent_boards = agent_boards.lock().unwrap();
        if let Some(bn) = agent_boards.board_names.get(&target_board) {
            board_name = if bn == "" || bn == "*" {
                kind.clone()
            } else {
                bn.clone()
            };
        } else {
            board_name = kind.clone();
        }
        agent_boards
            .board_values
            .insert(board_name.clone(), value.clone());
    }

    send_board(
        &app,
        source_agent.clone(),
        board_name.clone(),
        value.clone(),
    )
    .await;

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

async fn send_board(app: &AppHandle, agent: String, kind: String, value: Value) {
    let agent_commands = app.state::<Mutex<AgentCommands>>();
    let main_tx;
    {
        let agent_commands = agent_commands.lock().unwrap();
        main_tx = agent_commands.tx.clone();
    }
    main_tx
        .send(AgentMessage::Board { agent, kind, value })
        .await
        .unwrap_or_else(|e| {
            log::error!("Failed to send message: {}", e);
        });
}
