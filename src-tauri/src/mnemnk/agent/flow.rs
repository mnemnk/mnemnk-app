use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::vec;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager, State};

use super::agent;
use super::env::AgentEnv;
use crate::mnemnk::settings;

pub type AgentFlows = Vec<AgentFlow>;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AgentFlow {
    pub nodes: Vec<AgentFlowNode>,
    pub edges: Vec<AgentFlowEdge>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AgentFlowNode {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub config: Option<AgentFlowNodeConfig>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
}

pub type AgentFlowNodeConfig = HashMap<String, Value>;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AgentFlowEdge {
    pub id: String,
    pub source: String,
    pub source_handle: Option<String>,
    pub target: String,
    pub target_handle: Option<String>,
}

pub(super) fn init_agent_flows(app: &AppHandle) -> Result<()> {
    if let Some(dir) = agent_flows_dir(app) {
        let agent_flows = read_agent_flows(&dir)?;
        app.manage(Mutex::new(agent_flows));
    } else {
        return Err(anyhow::anyhow!("Agent flows directory not found"));
    }
    Ok(())
}

fn agent_flows_dir(app: &AppHandle) -> Option<PathBuf> {
    let mnemnk_dir = settings::mnemnk_dir(app);
    if mnemnk_dir.is_none() {
        return None;
    }
    let agent_flows_dir = PathBuf::from(mnemnk_dir.unwrap()).join("agent_flows");
    if !agent_flows_dir.exists() {
        std::fs::create_dir(&agent_flows_dir).expect("Failed to create agent flows directory");
    }
    Some(agent_flows_dir)
}

fn read_agent_flows<P: AsRef<Path>>(dir: P) -> Result<AgentFlows> {
    let mut flows = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() || path.extension().unwrap_or_default() != "json" {
            continue;
        }
        let flow = read_agent_flow(&path)?;
        flows.push(flow);
    }
    Ok(flows)
}

fn read_agent_flow(path: &PathBuf) -> Result<AgentFlow> {
    if !path.is_file() || path.extension().unwrap_or_default() != "json" {
        return Err(anyhow::anyhow!("Invalid file extension"));
    }
    let content = std::fs::read_to_string(path)?;
    let flow = serde_json::from_str(&content)?;
    Ok(flow)
}

fn save_agent_flows(flows: &AgentFlows, dir: &PathBuf) -> Result<()> {
    for (i, flow) in flows.iter().enumerate() {
        let path = dir.join(format!("{}.json", i));
        let content = serde_json::to_string_pretty(flow)?;
        std::fs::write(&path, content)?;
    }
    Ok(())
}

pub(super) fn sync_agent_flows(app: &AppHandle) {
    let agent_flows;
    {
        let state = app.state::<Mutex<AgentFlows>>();
        agent_flows = state.lock().unwrap().clone();
    }

    let mut board_names = HashMap::<String, String>::new();
    let mut node_map: HashMap<String, &AgentFlowNode> = HashMap::new();
    for agent_flow in &agent_flows {
        for node in &agent_flow.nodes {
            if !node.enabled {
                continue;
            }
            node_map.insert(node.id.clone(), &node);

            if node.name.starts_with("$") {
                if node.name == "$board" {
                    if let Some(board_name) = node
                        .config
                        .as_ref()
                        .and_then(|x| x.get("board_name").cloned())
                    {
                        if let Some(board_name_str) = board_name.as_str() {
                            board_names.insert(node.id.clone(), board_name_str.to_string());
                        }
                    }
                }
            }
        }
    }

    let mut edges = HashMap::<String, Vec<(String, String, String)>>::new();
    for agent_flow in &agent_flows {
        for edge in &agent_flow.edges {
            if !node_map.contains_key(&edge.source) || !node_map.contains_key(&edge.target) {
                continue;
            }

            let target = (
                edge.target.clone(),
                normalize_handle(&edge.source_handle),
                normalize_handle(&edge.target_handle),
            );
            if let Some(targets) = edges.get_mut(&edge.source) {
                targets.push(target);
            } else {
                edges.insert(edge.source.clone(), vec![target]);
            }
        }
    }

    let env = app.state::<AgentEnv>();

    // sync agents
    // TODO: move into AgentEnv

    let new_nodes: HashSet<_> = node_map.keys().cloned().collect();
    let old_nodes: HashSet<_>;
    {
        let env_nodes = env.agents.lock().unwrap();
        old_nodes = env_nodes.keys().cloned().collect();
    }

    // check if any agents need to be stopped
    for agent_id in old_nodes.difference(&new_nodes) {
        let mut env_nodes = env.agents.lock().unwrap();
        let Some(agent) = env_nodes.get_mut(agent_id) else {
            // maybe already stopped
            continue;
        };
        log::info!("Stopping agent: {}", agent_id);
        agent.stop(app).unwrap_or_else(|e| {
            log::error!("Failed to stop agent: {}", e);
        });
    }

    // update config for running agents
    for agent_id in new_nodes.intersection(&old_nodes) {
        let node = node_map[agent_id];

        let mut env_nodes = env.agents.lock().unwrap();
        let Some(agent) = env_nodes.get_mut(agent_id) else {
            // maybe already stopped
            continue;
        };
        if let Err(e) = agent.update(app, node.config.clone()) {
            log::error!("Failed to sync agent: {}", e);
        }
    }

    // initialize new agents
    for agent_id in new_nodes.difference(&old_nodes) {
        let node = node_map[agent_id];

        match agent::new_agent(&env, agent_id.to_string(), &node.name, node.config.clone()) {
            Ok(agent) => {
                let mut env_nodes = env.agents.lock().unwrap();
                log::info!("New agent: {}", agent.id());
                env_nodes.insert(agent.id().to_string(), agent);
            }
            Err(e) => {
                log::error!("Failed to create agent: {}", e);
                continue;
            }
        }
    }
    // start new agents
    for agent_id in new_nodes.difference(&old_nodes) {
        let mut env_nodes = env.agents.lock().unwrap();
        let Some(agent) = env_nodes.get_mut(agent_id) else {
            continue;
        };
        log::info!("Starting agent: {}", agent_id);
        if let Err(e) = agent.start(app) {
            log::error!("Failed to start agent: {}", e);
            // remove agent from env
            env_nodes.remove(agent_id);
        }
    }

    {
        let mut env_edges = env.edges.lock().unwrap();
        *env_edges = edges;
    }
    {
        let mut env_board_names = env.board_names.lock().unwrap();
        *env_board_names = board_names;
    }
}

fn normalize_handle(handle: &Option<String>) -> String {
    // None -> "*"
    let mut handle = handle.as_deref().unwrap_or("*");

    // "" -> "*"
    if handle.is_empty() {
        handle = "*";
    }

    handle.to_string()
}

pub fn find_agent_node<'a>(
    agent_flows: &'a AgentFlows,
    agent_id: &str,
) -> Option<&'a AgentFlowNode> {
    for agent_flow in agent_flows {
        if let Some(agent_node) = agent_flow.nodes.iter().find(|x| x.id == agent_id) {
            return Some(agent_node);
        }
    }
    None
}

#[tauri::command]
pub fn get_agent_flows_cmd(agent_flows: State<Mutex<AgentFlows>>) -> Result<Value, String> {
    let agent_flows = agent_flows.lock().unwrap();
    let agent_flows = agent_flows.clone();
    let value = serde_json::to_value(&agent_flows).map_err(|e| e.to_string())?;
    Ok(value)
}

#[tauri::command(rename_all = "snake_case")]
pub fn save_agent_flow_cmd(
    app: AppHandle,
    agent_flows: State<Mutex<AgentFlows>>,
    agent_flow: AgentFlow,
    idx: usize,
) -> Result<(), String> {
    let flows;
    {
        let mut agent_flows = agent_flows.lock().unwrap();
        if idx < agent_flows.len() {
            agent_flows[idx] = agent_flow;
        } else {
            agent_flows.push(agent_flow);
        }
        flows = agent_flows.clone();
    }
    let dir = agent_flows_dir(&app);
    if dir.is_none() {
        return Err("Agent flows directory not found".to_string());
    }
    save_agent_flows(&flows, &dir.unwrap()).map_err(|e| e.to_string())?;
    sync_agent_flows(&app);
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn read_agent_flow_cmd(path: String) -> Result<AgentFlow, String> {
    let path = PathBuf::from(path);
    read_agent_flow(&path).map_err(|e| e.to_string())
}
