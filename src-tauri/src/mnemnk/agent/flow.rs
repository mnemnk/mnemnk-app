use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::vec;

use anyhow::{Context as _, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager, State};

use super::agent;
use super::env::AgentEnv;
use crate::mnemnk::settings;

pub type AgentFlows = HashMap<String, AgentFlow>;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AgentFlow {
    pub nodes: Vec<AgentFlowNode>,
    pub edges: Vec<AgentFlowEdge>,

    pub name: Option<String>,

    #[serde(skip)]
    // Only set when reading/saving the file under the agent_flows_dir
    path: Option<PathBuf>,
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
        let mut agent_flows: AgentFlows = read_agent_flows(&dir)?;
        if agent_flows.is_empty() {
            agent_flows.insert(
                "main".to_string(),
                AgentFlow {
                    name: Some("main".to_string()),
                    ..Default::default()
                },
            );
        }
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
    let mut flows: AgentFlows = Default::default();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() || path.extension().unwrap_or_default() != "json" {
            // TODO: subdirectories support
            continue;
        }
        let mut flow = read_agent_flow(&path)?;

        flow.path = Some(path);

        let name = flow
            .path
            .as_ref()
            .unwrap()
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();
        flows.insert(name, flow);
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

pub(super) fn sync_agent_flows(app: &AppHandle) {
    let agent_flows;
    {
        let state = app.state::<Mutex<AgentFlows>>();
        agent_flows = state.lock().unwrap().clone();
    }

    let mut node_map: HashMap<String, &AgentFlowNode> = HashMap::new();
    for (_name, agent_flow) in &agent_flows {
        for node in &agent_flow.nodes {
            if !node.enabled {
                continue;
            }
            node_map.insert(node.id.clone(), &node);
        }
    }

    let mut edges = HashMap::<String, Vec<(String, String, String)>>::new();
    for (_name, agent_flow) in &agent_flows {
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

    // update edges
    {
        let mut env_edges = env.edges.lock().unwrap();
        *env_edges = edges;
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
    for (_name, agent_flow) in agent_flows {
        if let Some(agent_node) = agent_flow.nodes.iter().find(|x| x.id == agent_id) {
            return Some(agent_node);
        }
    }
    None
}

#[tauri::command]
pub fn get_agent_flows_cmd(agent_flows: State<Mutex<AgentFlows>>) -> Result<Value, String> {
    let agent_flows = agent_flows.lock().unwrap();
    let agent_flows = agent_flows.values().cloned().collect::<Vec<_>>();
    let value = serde_json::to_value(&agent_flows).map_err(|e| e.to_string())?;
    Ok(value)
}

#[tauri::command]
pub fn new_agent_flow_cmd(
    agent_flows: State<Mutex<AgentFlows>>,
    name: String,
) -> Result<AgentFlow, String> {
    let flow = new_agent_flow(agent_flows, &name).map_err(|e| e.to_string())?;
    Ok(flow)
}

fn new_agent_flow(agent_flows: State<Mutex<AgentFlows>>, name: &str) -> Result<AgentFlow> {
    let mut flow = AgentFlow::default();
    {
        let mut agent_flows = agent_flows.lock().unwrap();
        let name = unique_flow_name(&agent_flows, name);
        flow.name = Some(name.clone());
        agent_flows.insert(name.clone(), flow.clone());
    }
    Ok(flow)
}

#[tauri::command]
pub fn save_agent_flow_cmd(
    app: AppHandle,
    agent_flows: State<Mutex<AgentFlows>>,
    agent_flow: AgentFlow,
) -> Result<(), String> {
    save_agent_flow(&app, agent_flows, agent_flow).map_err(|e| e.to_string())
}

// Save the AgentFlow that matches the name in agent_flows.
fn save_agent_flow(
    app: &AppHandle,
    agent_flows: State<Mutex<AgentFlows>>,
    agent_flow: AgentFlow,
) -> Result<()> {
    let name = agent_flow
        .name
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Agent flow name is required"))?
        .clone();
    let path;
    {
        let agent_flows = agent_flows.lock().unwrap();
        let flow = agent_flows.get(&name).context("Agent flow not found")?;
        if let Some(p) = &flow.path {
            path = p.clone();
        } else {
            // If flow.path is None, this is the first time saving the AgentFlow, so set the path
            let dir = agent_flows_dir(&app).context("Agent flows directory not found")?;
            path = dir.join(name.clone()).with_extension("json");
        }
    }
    let content = serde_json::to_string_pretty(&agent_flow)?;
    std::fs::write(&path, content)?;
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn import_agent_flow_cmd(
    agent_flows: State<Mutex<AgentFlows>>,
    path: String,
) -> Result<AgentFlow, String> {
    let path = PathBuf::from(path);
    let mut flow = read_agent_flow(&path).map_err(|e| e.to_string())?;

    let mut agent_flows = agent_flows.lock().unwrap();

    let name = unique_flow_name(
        &agent_flows,
        &path.file_stem().unwrap().to_string_lossy().to_string(),
    );
    flow.name = Some(name.clone());

    agent_flows.insert(name, flow.clone());

    Ok(flow)
}

fn unique_flow_name(agent_flows: &AgentFlows, name: &str) -> String {
    let mut new_name = name.to_string();
    let mut i = 1;
    while agent_flows.contains_key(&new_name) {
        new_name = format!("{} ({})", name, i);
        i += 1;
    }
    new_name
}

#[tauri::command]
pub fn add_agent_node_cmd(
    agent_flows: State<Mutex<AgentFlows>>,
    flow_name: String,
    node: AgentFlowNode,
) -> Result<(), String> {
    let mut agent_flows = agent_flows.lock().unwrap();
    let flow = agent_flows
        .get_mut(&flow_name)
        .ok_or_else(|| "Agent flow not found".to_string())?;
    flow.nodes.push(node);
    Ok(())
}

#[tauri::command]
pub fn delete_agent_node_cmd(
    agent_flows: State<Mutex<AgentFlows>>,
    flow_name: String,
    node_id: String,
) -> Result<(), String> {
    let mut agent_flows = agent_flows.lock().unwrap();
    let flow = agent_flows
        .get_mut(&flow_name)
        .ok_or_else(|| "Agent flow not found".to_string())?;
    flow.nodes.retain(|node| node.id != node_id);
    Ok(())
}

#[tauri::command]
pub fn add_agent_edge_cmd(
    agent_flows: State<Mutex<AgentFlows>>,
    flow_name: String,
    edge: AgentFlowEdge,
) -> Result<(), String> {
    let mut agent_flows = agent_flows.lock().unwrap();
    let flow = agent_flows
        .get_mut(&flow_name)
        .ok_or_else(|| "Agent flow not found".to_string())?;
    flow.edges.push(edge);
    Ok(())
}

#[tauri::command]
pub fn delete_agent_edge_cmd(
    agent_flows: State<Mutex<AgentFlows>>,
    flow_name: String,
    edge_id: String,
) -> Result<(), String> {
    let mut agent_flows = agent_flows.lock().unwrap();
    let flow = agent_flows
        .get_mut(&flow_name)
        .ok_or_else(|| "Agent flow not found".to_string())?;
    flow.edges.retain(|edge| edge.id != edge_id);
    Ok(())
}
