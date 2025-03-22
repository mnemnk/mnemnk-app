use anyhow::Result;
use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager};

use super::{
    agent::{AgentConfig, AgentData, AsAgent},
    env::AgentEnv,
    AgentMessage,
};

const EMIT_PUBLISH: &str = "mnemnk:write_board";

#[derive(Clone, Debug, Serialize)]
struct WriteBoardMessage {
    agent: String,
    kind: String,
    value: Value,
}

pub struct BoardAgent {
    data: AgentData,
    board_name: Option<String>,
}

impl AsAgent for BoardAgent {
    fn data(&self) -> &AgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AgentData {
        &mut self.data
    }

    fn start(&mut self, app: &AppHandle) -> Result<()> {
        if let Some(board_name) = &self.board_name {
            let env = app.state::<AgentEnv>();
            let mut board_nodes = env.board_nodes.lock().unwrap();
            if let Some(nodes) = board_nodes.get_mut(board_name) {
                nodes.push(self.data.id.clone());
            } else {
                board_nodes.insert(board_name.clone(), vec![self.data.id.clone()]);
            }
        }
        Ok(())
    }

    fn stop(&mut self, app: &AppHandle) -> Result<()> {
        if let Some(board_name) = &self.board_name {
            let env = app.state::<AgentEnv>();
            let mut board_nodes = env.board_nodes.lock().unwrap();
            if let Some(nodes) = board_nodes.get_mut(board_name) {
                nodes.retain(|x| x != &self.data.id);
            }
        }
        Ok(())
    }

    fn update(&mut self, app: &AppHandle, config: Option<AgentConfig>) -> Result<()> {
        let board_name = normalize_board_name(&config);
        if self.board_name != board_name {
            if let Some(board_name) = &self.board_name {
                let env = app.state::<AgentEnv>();
                let mut board_nodes = env.board_nodes.lock().unwrap();
                if let Some(nodes) = board_nodes.get_mut(board_name) {
                    nodes.retain(|x| x != &self.data.id);
                }
            }
            if let Some(board_name) = &board_name {
                let env = app.state::<AgentEnv>();
                let mut board_nodes = env.board_nodes.lock().unwrap();
                if let Some(nodes) = board_nodes.get_mut(board_name) {
                    nodes.push(self.data.id.clone());
                } else {
                    board_nodes.insert(board_name.clone(), vec![self.data.id.clone()]);
                }
            }
            self.board_name = board_name;
        }
        Ok(())
    }

    fn input(&self, app: &AppHandle, source: String, kind: String, value: Value) -> Result<()> {
        let kind = self.board_name.clone().unwrap_or(kind.to_string());
        if kind.is_empty() {
            return Ok(());
        }

        {
            let env = app.state::<AgentEnv>();
            let mut board_values = env.board_values.lock().unwrap();
            board_values.insert(kind.clone(), value.clone());
        }
        let app = app.clone();
        let env = app.state::<AgentEnv>();
        send_board(&env, source.clone(), kind.clone(), value.clone());

        // remove image from the value. it's too big to send to frontend
        let mut value = value;
        if value.get("image").is_some() {
            value.as_object_mut().unwrap().remove("image");
        }

        // emit the message to frontend
        let message = WriteBoardMessage {
            agent: source,
            kind,
            value,
        };
        app.emit(EMIT_PUBLISH, Some(message)).unwrap_or_else(|e| {
            log::error!("Failed to emit message: {}", e);
        });
        Ok(())
    }
}

impl BoardAgent {
    pub fn new(id: String, def_name: String, config: Option<AgentConfig>) -> Result<Self> {
        let board_name = normalize_board_name(&config);
        Ok(Self {
            data: AgentData {
                id,
                def_name,
                config,
            },
            board_name,
        })
    }
}

fn normalize_board_name(config: &Option<AgentConfig>) -> Option<String> {
    let Some(config) = config else {
        // config is not set
        return None;
    };
    let Some(board_name) = config.get("board_name") else {
        // board_name is not set
        return None;
    };
    let board_name = board_name.as_str().unwrap_or_default().trim();
    if board_name.is_empty() {
        // board_name is empty
        return None;
    }
    return Some(board_name.to_string());
}

pub async fn board_message(app: &AppHandle, source_agent: String, kind: String, value: Value) {
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

fn send_board(env: &AgentEnv, agent: String, kind: String, value: Value) {
    let main_tx = env.tx.clone();
    main_tx
        .try_send(AgentMessage::Board { agent, kind, value })
        .unwrap_or_else(|e| {
            log::error!("Failed to send message: {}", e);
        });
}
