use anyhow::{Context as _, Result};
use tauri::{AppHandle, Manager};

use crate::mnemnk::store;

use super::{data::AgentData, env::AgentEnv};

#[derive(Clone, Debug)]
pub enum EnvAgentMessage {
    AgentOut {
        agent: String,
        ch: String,
        data: AgentData,
    },
    BoardOut {
        name: String,
        data: AgentData,
    },
    Store {
        data: AgentData,
    },
}

pub async fn send_agent_out(
    env: &AgentEnv,
    agent: String,
    ch: String,
    data: AgentData,
) -> Result<()> {
    let env_tx;
    {
        env_tx = env
            .tx
            .lock()
            .unwrap()
            .clone()
            .context("tx is not initialized")?;
    }
    env_tx
        .send(EnvAgentMessage::AgentOut { agent, ch, data })
        .await
        .context("Failed to send AgentOut message")
}

pub fn try_send_agent_out(
    env: &AgentEnv,
    agent: String,
    ch: String,
    data: AgentData,
) -> Result<()> {
    let env_tx;
    {
        env_tx = env
            .tx
            .lock()
            .unwrap()
            .clone()
            .context("tx is not initialized")?;
    }
    env_tx
        .try_send(EnvAgentMessage::AgentOut { agent, ch, data })
        .context("Failed to try_send AgentOut message")
}

pub fn try_send_board_out(env: &AgentEnv, name: String, data: AgentData) -> Result<()> {
    let env_tx;
    {
        env_tx = env
            .tx
            .lock()
            .unwrap()
            .clone()
            .context("tx is not initialized")?;
    }
    env_tx
        .try_send(EnvAgentMessage::BoardOut { name, data })
        .context("Failed to try_send BoardOut message")
}

pub fn try_send_store(app: &AppHandle, data: AgentData) -> Result<()> {
    let env = app.state::<AgentEnv>();
    let env_tx;
    {
        env_tx = env
            .tx
            .lock()
            .unwrap()
            .clone()
            .context("tx is not initialized")?;
    }
    env_tx
        .try_send(EnvAgentMessage::Store { data })
        .context("Failed to send Store message")
}

// Processing AgentOut message
pub async fn agent_out(app: &AppHandle, source_agent: String, ch: String, data: AgentData) {
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
        let (target_agent, source_handle, target_handle) = target;

        if source_handle != ch && source_handle != "*" {
            // Skip if source_handle does not match with the given ch.
            // "*" is a wildcard, and outputs messages of all channels.
            continue;
        }

        {
            let env_agents = env.agents.lock().unwrap();
            if !env_agents.contains_key(&target_agent) {
                continue;
            }
        }

        let target_ch = if target_handle == "*" {
            // If target_handle is "*", use the ch specified by the source agent
            ch.clone()
        } else {
            target_handle.clone()
        };

        env.agent_input(&target_agent, target_ch, data.clone())
            .await
            .unwrap_or_else(|e| {
                log::error!("Failed to send message to {}: {}", target_agent, e);
            });
    }
}

pub async fn board_out(app: &AppHandle, name: String, data: AgentData) {
    let env = app.state::<AgentEnv>();

    let board_nodes;
    {
        let env_board_nodes = env.board_out_agents.lock().unwrap();
        board_nodes = env_board_nodes.get(&name).cloned();
    }
    let Some(board_nodes) = board_nodes else {
        // board not found
        return;
    };

    for node in board_nodes {
        // Perhaps we could process this by send_message_to BoardOutAgent

        let edges;
        {
            let env_edges = env.edges.lock().unwrap();
            edges = env_edges.get(&node).cloned();
        }
        let Some(edges) = edges else {
            // edges not found
            continue;
        };
        for (target_agent, _source_handle, target_handle) in edges {
            env.agent_input(&target_agent, target_handle, data.clone())
                .await
                .unwrap_or_else(|e| {
                    log::error!("Failed to send message to {}: {}", target_agent, e);
                });
        }
    }
}

pub fn store(app_handle: &AppHandle, data: AgentData) {
    store::store(app_handle, data);
}
