use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicUsize;

use anyhow::{bail, Context as _, Result};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

use super::env::AgentEnv;
use super::AgentConfig;
use crate::mnemnk::settings;

pub type AgentFlows = HashMap<String, AgentFlow>;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AgentFlow {
    pub nodes: Vec<AgentFlowNode>,
    pub edges: Vec<AgentFlowEdge>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub viewport: Option<Viewport>,

    #[serde(skip)]
    // Only set when reading/saving the file under the agent_flows_dir
    path: Option<PathBuf>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Viewport {
    pub x: f64,
    pub y: f64,
    pub zoom: f64,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AgentFlowNode {
    pub id: String,
    pub name: String,
    pub enabled: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<AgentConfig>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<f64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<f64>,
}

impl AgentFlowNode {
    pub fn new(env: &AgentEnv, def_name: String) -> Result<Self> {
        let default_config = env.get_agent_default_config(&def_name);
        let config = if let Some(default_config) = default_config {
            let mut config = AgentConfig::new();
            for (key, entry) in default_config {
                config.set(key, entry.value.clone());
            }
            Some(config)
        } else {
            None
        };

        Ok(Self {
            id: new_node_id(&def_name),
            name: def_name,
            enabled: false,
            config,
            title: None,
            x: None,
            y: None,
            width: None,
            height: None,
        })
    }
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct AgentFlowEdge {
    pub id: String,
    pub source: String,
    pub source_handle: String,
    pub target: String,
    pub target_handle: String,
}

pub fn init(app: &AppHandle) -> Result<()> {
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
    Ok(())
}

pub fn ready(app: &AppHandle) -> Result<()> {
    let env = app.state::<AgentEnv>();
    let agent_flow_names;
    {
        let agent_flows = env.flows.lock().unwrap();
        agent_flow_names = agent_flows.keys().cloned().collect::<Vec<_>>();
    }
    for name in agent_flow_names {
        env.start_agent_flow(&name).unwrap_or_else(|e| {
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
    let base_dir = dir.as_ref().to_path_buf();
    read_agent_flows_recursive(&base_dir, &base_dir, &mut flows)?;
    Ok(flows)
}

fn read_agent_flows_recursive<P: AsRef<Path>>(
    base_dir: &Path,
    current_dir: P,
    flows: &mut AgentFlows,
) -> Result<()> {
    for entry in std::fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            // Recursively process subdirectories
            read_agent_flows_recursive(base_dir, &path, flows)?;
        } else if path.is_file() && path.extension().unwrap_or_default() == "json" {
            // Process JSON files
            let mut flow = read_agent_flow(path.clone())?;

            // Set the flow name to be the relative path from base_dir (without extension)
            if let Some(relative_path) = path.strip_prefix(base_dir).ok() {
                let relative_path_str = relative_path.to_string_lossy();
                // Remove the .json extension
                let flow_name = relative_path_str
                    .trim_end_matches(".json")
                    .replace('\\', "/"); // Ensure consistent path separators

                flow.name = Some(flow_name.clone());
                flows.insert(flow_name, flow);
            }
        }
    }

    Ok(())
}

fn read_agent_flow(path: PathBuf) -> Result<AgentFlow> {
    if !path.is_file() || path.extension().unwrap_or_default() != "json" {
        return Err(anyhow::anyhow!("Invalid file extension"));
    }
    let content = std::fs::read_to_string(&path)?;
    let mut flow: AgentFlow = serde_json::from_str(&content)?;
    let (nodes, edges) = copy_sub_flow(flow.nodes.iter().collect(), flow.edges.iter().collect());
    flow.nodes = nodes;
    flow.edges = edges;
    flow.path = Some(path);
    Ok(flow)
}

pub fn rename_agent_flow(
    app: &AppHandle,
    env: &AgentEnv,
    old_name: &str,
    new_name: &str,
) -> Result<String> {
    if old_name == new_name {
        return Ok(old_name.to_string());
    }

    let new_name = env.rename_agent_flow(old_name, new_name)?;
    let mut flows = env.flows.lock().unwrap();
    let Some(flow) = flows.get_mut(&new_name) else {
        bail!("flow::rename_agent_flow: Agent flow {} not found", new_name);
    };

    // Check if the flow already saved
    if let Some(path) = flow.path.clone() {
        let base_dir = agent_flows_dir(app).context("Agent flows directory not found")?;

        let mut new_path = base_dir.clone();

        let path_components: Vec<&str> = new_name.split('/').collect();
        for &component in &path_components[..path_components.len() - 1] {
            new_path = new_path.join(component);
        }
        // Ensure the parent directory exists
        if !new_path.exists() {
            std::fs::create_dir_all(new_path.clone())?;
        }

        new_path = new_path
            .join(path_components.last().context("no last component")?)
            .with_extension("json");

        // Rename the file
        std::fs::rename(&path, &new_path)?;
        flow.path = Some(new_path);

        // Clean up empty directories
        let mut old_dir = path.parent().context("no parent")?.to_path_buf();
        while old_dir != base_dir {
            // Try to remove directory (will only succeed if empty)
            let _ = std::fs::remove_dir(&old_dir);
            old_dir = old_dir.parent().context("no parent")?.to_path_buf();
        }
    }

    Ok(new_name)
}

pub fn delete_agent_flow(app: &AppHandle, env: &AgentEnv, name: &str) -> Result<()> {
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

    // Clean up empty directories
    let base_dir = agent_flows_dir(app).context("Agent flows directory not found")?;
    let mut old_dir = path.parent().context("no parent")?.to_path_buf();
    while old_dir != base_dir {
        // Try to remove directory (will only succeed if empty)
        let _ = std::fs::remove_dir(&old_dir);
        old_dir = old_dir.parent().context("no parent")?.to_path_buf();
    }

    Ok(())
}

pub fn insert_agent_flow(env: State<AgentEnv>, agent_flow: AgentFlow) -> Result<()> {
    let name = agent_flow
        .name
        .clone()
        .context("Agent flow name not found")?;

    let mut flows = env.flows.lock().unwrap();
    if let Some(flow) = flows.get_mut(&name) {
        // if the flow already exists, we need to copy the path from the existing flow
        let mut agent_flow = agent_flow;
        agent_flow.path = flow.path.clone();
        flows.insert(name, agent_flow);
        return Ok(());
    }
    flows.insert(name, agent_flow);

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
            let mut new_path = agent_flows_dir(&app).context("Agent flows directory not found")?;

            let path_components: Vec<&str> = name.split('/').collect();
            for &component in &path_components[..path_components.len() - 1] {
                new_path = new_path.join(component);
            }
            // Ensure the parent directory exists
            if !new_path.exists() {
                std::fs::create_dir_all(new_path.clone())?;
            }

            path = new_path
                .join(path_components.last().context("no last component")?)
                .with_extension("json");
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
    let mut flow = read_agent_flow(path.clone())?;

    // Get the base name from the file name
    let base_name = path
        .file_stem()
        .context("Failed to get file stem")?
        .to_string_lossy()
        .trim()
        .to_string();
    if base_name.is_empty() {
        bail!("Agent flow name is empty");
    }

    // Unique name for the flow
    let name;
    {
        let agent_flows = env.flows.lock().unwrap();
        name = unique_flow_name(&agent_flows, &base_name);
    }
    flow.name = Some(name.clone());

    // reset path of the flow
    flow.path = None;

    // disable all nodes
    for node in &mut flow.nodes {
        node.enabled = false;
    }

    env.add_agent_flow(&flow)
        .context("Failed to add agent flow")?;

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

pub fn copy_sub_flow(
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

static NODE_ID_COUNTER: AtomicUsize = AtomicUsize::new(1);

fn new_node_id(def_name: &str) -> String {
    let new_id = NODE_ID_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    format!("{}_{}", def_name, new_id)
}

fn new_edge_id(source: &str, source_handle: &str, target: &str, target_handle: &str) -> String {
    format!(
        "xy-edge__{}{}__{}{}",
        source, source_handle, target, target_handle
    )
}

pub fn add_agent_flow_node(env: &AgentEnv, flow_name: &str, node: &AgentFlowNode) -> Result<()> {
    let mut flows = env.flows.lock().unwrap();
    let Some(flow) = flows.get_mut(flow_name) else {
        bail!("Agent flow {} not found", flow_name);
    };
    env.add_agent(&node)?;
    flow.nodes.push(node.clone());
    Ok(())
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
    env.add_edge(edge)?;
    flow.edges.push(edge.clone());
    Ok(())
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
