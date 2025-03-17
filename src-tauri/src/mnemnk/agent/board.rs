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

pub async fn board_message(app: &AppHandle, kind: String, value: Value) {
    let subscribers;
    let agent_boards = app.state::<Mutex<AgentBoards>>();
    {
        let agent_board = agent_boards.lock().unwrap();
        subscribers = agent_board.subscribers.get(&kind).cloned(); // TODO: use board_name instead of kind
    }
    if let Some(subscribers) = subscribers {
        let enabled_nodes;
        let agent_commands = app.state::<Mutex<AgentCommands>>();
        {
            let agent_commands = agent_commands.lock().unwrap();
            enabled_nodes = agent_commands.enabled_nodes.clone();
        }
        for subscriber in subscribers {
            if !enabled_nodes.contains(&subscriber) {
                continue;
            }
            if subscriber.starts_with("$") {
                if subscriber.starts_with("$board_") {
                    if let Err(e) = write_message_to_board(
                        &app,
                        subscriber.clone(),
                        kind.clone(),
                        value.clone(),
                    )
                    .await
                    {
                        log::error!("Failed to write board: {}", e);
                    };
                } else if subscriber.starts_with("$database_") {
                    if let Err(e) =
                        store::store(&app, kind.clone(), kind.clone(), value.clone()).await
                    {
                        log::error!("Failed to store: {}", e);
                    }
                } else {
                    log::error!("Unknown subscriber: {}", subscriber);
                }
            } else {
                write_message_to_agent(&agent_commands, &kind, &subscriber, &kind, &value);
            }
        }
    }
}

pub async fn write_message(app: &AppHandle, agent_id: String, kind: String, value: Value) {
    let targets;
    let enabled_nodes;
    let agent_commands = app.state::<Mutex<AgentCommands>>();
    {
        let agent_commands = agent_commands.lock().unwrap();
        targets = agent_commands.edges.get(&agent_id).cloned();
        enabled_nodes = agent_commands.enabled_nodes.clone();
    }
    if let Some(targets) = targets {
        for target in targets {
            if !enabled_nodes.contains(&target) {
                continue;
            }
            if target.starts_with("$") {
                if target.starts_with("$board_") {
                    if let Err(e) =
                        write_message_to_board(&app, target.clone(), kind.clone(), value.clone())
                            .await
                    {
                        log::error!("Failed to write board: {}", e);
                    };
                } else if target.starts_with("$database_") {
                    if let Err(e) =
                        store::store(&app, agent_id.clone(), kind.clone(), value.clone()).await
                    {
                        log::error!("Failed to store: {}", e);
                    }
                } else {
                    log::error!("Unknown target: {}", target);
                }
            } else {
                write_message_to_agent(&agent_commands, &agent_id, &target, &kind, &value);
            }
        }
    }
}

fn write_message_to_agent(
    agent_commands: &State<Mutex<AgentCommands>>,
    agent_id: &str,
    target: &str,
    kind: &str,
    value: &Value,
) {
    let mut agent_commands = agent_commands.lock().unwrap();
    if let Some(command) = agent_commands.commands.get_mut(target) {
        command
            .write(format!(".PUBLISH {} {} {}\n", agent_id, kind, value.to_string()).as_bytes())
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
    board_id: String,
    kind: String,
    value: Value,
) -> Result<()> {
    // update board value
    let board_name;
    let agent_boards = app.state::<Mutex<AgentBoards>>();
    {
        let mut agent_boards = agent_boards.lock().unwrap();
        if let Some(bn) = agent_boards.board_names.get(&board_id) {
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

    send_board(&app, board_name.clone(), value.clone()).await;

    // remove image from the value. it's too big to send to frontend
    let mut value = value;
    if value.get("image").is_some() {
        value.as_object_mut().unwrap().remove("image");
    }

    // emit the message to frontend
    let message = WriteBoardMessage {
        agent: board_id,
        kind: board_name,
        value,
    };
    app.emit(EMIT_PUBLISH, Some(message))?;

    Ok(())
}

async fn send_board(app: &AppHandle, kind: String, value: Value) {
    let agent_commands = app.state::<Mutex<AgentCommands>>();
    let main_tx;
    {
        let agent_commands = agent_commands.lock().unwrap();
        main_tx = agent_commands.tx.clone();
    }
    main_tx
        .send(AgentMessage::Board { kind, value })
        .await
        .unwrap_or_else(|e| {
            log::error!("Failed to send message: {}", e);
        });
}
