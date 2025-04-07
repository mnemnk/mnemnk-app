use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context as _, Result};
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager, State};

use super::env::AgentEnv;
use crate::mnemnk::settings;

pub type AgentFlows = HashMap<String, AgentFlow>;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AgentFlow {
    pub nodes: Vec<AgentFlowNode>,
    pub edges: Vec<AgentFlowEdge>,

    #[serde(skip_serializing_if = "Option::is_none")]
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

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<AgentFlowNodeConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<f64>,
}

pub type AgentFlowNodeConfig = HashMap<String, Value>;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AgentFlowEdge {
    pub id: String,
    pub source: String,
    pub source_handle: String,
    pub target: String,
    pub target_handle: String,
}

pub(super) fn init_agent_flows(app: &AppHandle) -> Result<()> {
    let dir = agent_flows_dir(app).context("Agent flows directory not found")?;
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
    let env = app.state::<AgentEnv>();
    for (_name, agent_flow) in &agent_flows {
        env.add_agent_flow(agent_flow).unwrap_or_else(|e| {
            log::error!("Failed to add agent flow: {}", e);
        });
    }
    for (name, _agent_flow) in &agent_flows {
        env.start_agent_flow(name).unwrap_or_else(|e| {
            log::error!("Failed to start agent flow: {}", e);
        });
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
        let flow = read_agent_flow(path)?;
        flows.insert(flow.name.clone().unwrap(), flow);
    }
    Ok(flows)
}

fn read_agent_flow(path: PathBuf) -> Result<AgentFlow> {
    if !path.is_file() || path.extension().unwrap_or_default() != "json" {
        return Err(anyhow::anyhow!("Invalid file extension"));
    }
    let content = std::fs::read_to_string(&path)?;
    let mut flow: AgentFlow = serde_json::from_str(&content)?;

    let name = path.file_stem().unwrap().to_string_lossy().to_string();
    flow.name = Some(name);

    flow.path = Some(path);

    Ok(flow)
}

pub fn rename_agent_flow(env: &AgentEnv, old_name: &str, new_name: &str) -> Result<String> {
    let new_name = env.rename_agent_flow(old_name, new_name)?;
    let mut flows = env.flows.lock().unwrap();
    let Some(flow) = flows.get_mut(&new_name) else {
        bail!("flow::rename_agent_flow: Agent flow {} not found", new_name);
    };
    if let Some(path) = &flow.path {
        // Rename the file
        let new_path = path.with_file_name(&new_name).with_extension("json");
        std::fs::rename(&path, &new_path)?;
        flow.path = Some(new_path);
    }
    Ok(new_name)
}

pub fn delete_agent_flow(env: &AgentEnv, name: &str) -> Result<()> {
    let mut flows = env.flows.lock().unwrap();
    let Some(flow) = flows.remove(name) else {
        bail!("flow::delete_agent_flow: Agent flow {} not found", name);
    };

    let Some(path) = &flow.path else {
        // Flow is not saved to a file, so just remove it from the flows
        return Ok(());
    };

    // Delete the file
    std::fs::remove_file(path).context("Failed to delete agent flow file")?;

    Ok(())
}

pub fn save_agent_flow(app: &AppHandle, env: State<AgentEnv>, agent_flow: AgentFlow) -> Result<()> {
    let name = agent_flow
        .name
        .clone()
        .context("Agent flow name not found")?;
    let path;
    {
        let agent_flows = env.flows.lock().unwrap();
        let flow = agent_flows.get(&name).context("Agent flow not found")?;
        if let Some(p) = &flow.path {
            path = p.clone();
        } else {
            // If flow.path is None, this is the first time saving the AgentFlow, so set the path
            let dir = agent_flows_dir(&app).context("Agent flows directory not found")?;
            path = dir.join(name.clone()).with_extension("json");
        }
    }

    // remove the name field from the saving flow before saving
    let mut agent_flow_copy = agent_flow.clone();
    agent_flow_copy.name = None;
    let content = serde_json::to_string_pretty(&agent_flow_copy)?;
    std::fs::write(&path, content)?;

    // update the path in the flow
    {
        let mut flows = env.flows.lock().unwrap();
        let Some(flow) = flows.get_mut(&name) else {
            bail!("Agent flow {} not found", name);
        };
        flow.path = Some(path);
    }

    Ok(())
}

pub fn import_agent_flow(env: &AgentEnv, path: String) -> Result<AgentFlow> {
    let path = PathBuf::from(path);
    let mut flow = read_agent_flow(path)?;

    // refresh the node and edge ids
    let (nodes, edges) = copy_sub_flow(flow.nodes.iter().collect(), flow.edges.iter().collect());
    flow.nodes = nodes;
    flow.edges = edges;

    // disable all nodes
    for node in &mut flow.nodes {
        node.enabled = false;
    }

    // Unique name for the flow
    let mut agent_flows = env.flows.lock().unwrap();
    let name = unique_flow_name(
        &agent_flows,
        flow.name.as_deref().context("Agent flow name not set")?,
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

fn copy_sub_flow(
    nodes: Vec<&AgentFlowNode>,
    edges: Vec<&AgentFlowEdge>,
) -> (Vec<AgentFlowNode>, Vec<AgentFlowEdge>) {
    let mut new_nodes = Vec::new();
    let mut node_id_map = HashMap::new();
    for node in nodes {
        let new_id = new_node_id(node.name.as_str());
        node_id_map.insert(node.id.clone(), new_id.clone());
        let mut new_node = node.clone();
        new_node.id = new_id;
        new_nodes.push(new_node);
    }

    let mut new_edges = Vec::new();
    for edge in edges {
        let Some(source) = node_id_map.get(&edge.source) else {
            continue;
        };
        let Some(target) = node_id_map.get(&edge.target) else {
            continue;
        };
        let mut new_edge = edge.clone();
        new_edge.id = new_edge_id(source, &edge.source_handle, target, &edge.target_handle);
        new_edge.source = source.clone();
        new_edge.target = target.clone();
        new_edges.push(new_edge);
    }

    (new_nodes, new_edges)
}

fn new_node_id(def_name: &str) -> String {
    format!("{}_{}", def_name, nanoid!())
}

fn new_edge_id(source: &str, source_handle: &str, target: &str, target_handle: &str) -> String {
    format!(
        "xy-edge__{}{}-{}{}",
        source, source_handle, target, target_handle
    )
}

pub fn add_agent_flow_node(env: &AgentEnv, flow_name: &str, node: &AgentFlowNode) -> Result<()> {
    let mut flows = env.flows.lock().unwrap();
    let Some(flow) = flows.get_mut(flow_name) else {
        bail!("Agent flow {} not found", flow_name);
    };
    flow.nodes.push(node.clone());
    env.add_agent(&node)
}

pub fn remove_agent_flow_node(env: &AgentEnv, flow_name: &str, node_id: &str) -> Result<()> {
    let mut flows = env.flows.lock().unwrap();
    let Some(flow) = flows.get_mut(flow_name) else {
        bail!("Agent flow {} not found", flow_name);
    };
    flow.nodes.retain(|node| node.id != node_id);
    env.remove_agent(&node_id)
}

pub fn add_agent_flow_edge(env: &AgentEnv, flow_name: &str, edge: &AgentFlowEdge) -> Result<()> {
    let mut flows = env.flows.lock().unwrap();
    let Some(flow) = flows.get_mut(flow_name) else {
        bail!("Agent flow {} not found", flow_name);
    };
    flow.edges.push(edge.clone());
    env.add_edge(edge)
}

pub fn remove_agent_flow_edge(env: &AgentEnv, flow_name: &str, edge_id: &str) -> Result<()> {
    let mut flows = env.flows.lock().unwrap();
    let Some(flow) = flows.get_mut(flow_name) else {
        bail!("Agent flow {} not found", flow_name);
    };
    if let Some(idx) = flow.edges.iter().position(|edge| edge.id == edge_id) {
        env.remove_edge(&flow.edges[idx]);
        flow.edges.remove(idx);
    }
    Ok(())
}
