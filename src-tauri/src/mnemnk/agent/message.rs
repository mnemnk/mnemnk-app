use anyhow::{Context as _, Result};
use tauri::{AppHandle, Manager};

use crate::mnemnk::store;

use super::{
    agent::{AgentData, AgentStatus},
    env::AgentEnv,
};

#[derive(Clone, Debug)]
pub enum AgentMessage {
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
    let main_tx = env
        .tx
        .lock()
        .unwrap()
        .clone()
        .context("tx is not initialized")?;
    main_tx
        .send(AgentMessage::AgentOut { agent, ch, data })
        .await
        .context("Failed to send AgentOut message")
}

pub fn try_send_agent_out(
    env: &AgentEnv,
    agent: String,
    ch: String,
    data: AgentData,
) -> Result<()> {
    let main_tx = env
        .tx
        .lock()
        .unwrap()
        .clone()
        .context("tx is not initialized")?;
    main_tx
        .try_send(AgentMessage::AgentOut { agent, ch, data })
        .context("Failed to try_send AgentOut message")
}

pub fn try_send_board_out(env: &AgentEnv, name: String, data: AgentData) -> Result<()> {
    let main_tx = env
        .tx
        .lock()
        .unwrap()
        .clone()
        .context("tx is not initialized")?;
    main_tx
        .try_send(AgentMessage::BoardOut { name, data })
        .context("Failed to try_send BoardOut message")
}

pub fn try_send_store(app: &AppHandle, data: AgentData) -> Result<()> {
    let env = app.state::<AgentEnv>();
    let main_tx = env
        .tx
        .lock()
        .unwrap()
        .clone()
        .context("tx is not initialized")?;
    main_tx
        .try_send(AgentMessage::Store { data })
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
        // In reality, targets are normalized to id/source_handle/target_handle in sync_agent_flows,
        // so unwrap should not fail.

        let (target_agent, source_handle, target_handle) = target;

        if source_handle != ch && source_handle != "*" {
            // Skip if source_handle does not match with the given ch
            continue;
        }

        {
            let env_agents = env.agents.lock().unwrap();
            if !env_agents.contains_key(&target_agent) {
                continue;
            }
        }

        send_agent_in(&env, &target_agent, target_handle.clone(), data.clone())
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
            send_agent_in(&env, &target_agent, target_handle, data.clone())
        }
    }
}

fn send_agent_in(env: &AgentEnv, agent_id: &str, ch: String, data: AgentData) {
    if let Some(agent) = env.agents.lock().unwrap().get_mut(agent_id) {
        if *agent.status() != AgentStatus::Run {
            return;
        }
        agent.input(ch, data).unwrap_or_else(|e| {
            log::error!("Failed to send message to {}: {}", agent_id, e);
        });
    }
}

pub fn store(app_handle: &AppHandle, data: AgentData) {
    store::store(app_handle, data);
}
