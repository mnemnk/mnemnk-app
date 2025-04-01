use anyhow::{bail, Context as _, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_shell::process::CommandChild;
use tokio::sync::mpsc;

use crate::mnemnk::settings;

use super::agent::{self, AsyncAgent};
use super::definition::{init_agent_defs, AgentDefinition, AgentDefinitions};
use super::flow::{AgentFlow, AgentFlowEdge, AgentFlowNode, AgentFlows};
use super::message::AgentMessage;

pub struct AgentEnv {
    // agent flows
    pub flows: Mutex<AgentFlows>,

    // agent def name -> agent definition
    pub defs: Mutex<AgentDefinitions>,

    // node id -> agent
    pub agents: Mutex<HashMap<String, Box<dyn AsyncAgent>>>,

    // node id -> [node ids / subscriber handle / target handle]
    pub edges: Mutex<HashMap<String, Vec<(String, String, String)>>>,

    // node id -> child process
    pub commands: Mutex<HashMap<String, CommandChild>>,

    // board name -> [node id]
    pub board_nodes: Mutex<HashMap<String, Vec<String>>>,

    // board name -> value
    pub board_values: Mutex<HashMap<String, Value>>,

    // message sender
    pub tx: mpsc::Sender<AgentMessage>,
}

impl AgentEnv {
    fn new(tx: mpsc::Sender<AgentMessage>) -> Self {
        Self {
            flows: Default::default(),
            defs: Default::default(),
            agents: Default::default(),
            edges: Default::default(),
            commands: Default::default(),
            board_nodes: Default::default(),
            board_values: Default::default(),
            tx,
        }
    }

    pub fn init(app: &AppHandle, tx: mpsc::Sender<AgentMessage>) -> Result<()> {
        let env = Self::new(tx);

        let agent_defs = init_agent_defs(app)?;
        settings::init_agent_configs(app, &agent_defs)?;
        {
            let mut defs = env.defs.lock().unwrap();
            *defs = agent_defs;
        }

        app.manage(env);
        Ok(())
    }

    pub fn new_agent_flow(&self, name: &str) -> Result<AgentFlow> {
        let new_name = self.unique_flow_name(name);
        let mut flows = self.flows.lock().unwrap();
        let mut flow = AgentFlow::default();
        flow.name = Some(new_name.clone());
        flows.insert(new_name, flow.clone());
        Ok(flow)
    }

    fn unique_flow_name(&self, name: &str) -> String {
        let flows = self.flows.lock().unwrap();
        let mut new_name = name.to_string();
        let mut i = 1;
        while flows.contains_key(&new_name) {
            new_name = format!("{}{}", name, i);
            i += 1;
        }
        new_name
    }

    pub fn add_agent_flow(&self, agent_flow: &AgentFlow) -> Result<()> {
        let name = agent_flow
            .name
            .clone()
            .context("Agent flow name is missing")?;

        // add the given flow into flows
        {
            let mut flows = self.flows.lock().unwrap();
            if flows.contains_key(&name) {
                bail!("Agent flow {} already exists", name);
            }
            flows.insert(name.clone(), agent_flow.clone());
        }

        // add nodes into agents
        for node in agent_flow.nodes.iter() {
            self.add_agent(node).unwrap_or_else(|e| {
                log::error!("Failed to add_agent_node {}: {}", node.id, e);
            });
        }

        // add edges into edges
        for edge in agent_flow.edges.iter() {
            self.add_edge(edge).unwrap_or_else(|e| {
                log::error!("Failed to add_edge {}: {}", edge.source, e);
            });
        }

        Ok(())
    }

    pub fn add_agent(&self, node: &AgentFlowNode) -> Result<()> {
        let mut agents = self.agents.lock().unwrap();
        if agents.contains_key(&node.id) {
            bail!("Agent {} already exists", node.id);
        }
        if let Ok(agent) = agent::new_agent(self, node.id.clone(), &node.name, node.config.clone())
        {
            agents.insert(node.id.clone(), agent);
            log::info!("Agent {} created", node.id);
        } else {
            bail!("Failed to create agent {}", node.id);
        }
        Ok(())
    }

    pub fn add_edge(&self, edge: &AgentFlowEdge) -> Result<()> {
        let mut edges = self.edges.lock().unwrap();
        if let Some(targets) = edges.get_mut(&edge.source) {
            targets.push((
                edge.target.clone(),
                normalize_handle(&edge.source_handle),
                normalize_handle(&edge.target_handle),
            ));
        } else {
            edges.insert(
                edge.source.clone(),
                vec![(
                    edge.target.clone(),
                    normalize_handle(&edge.source_handle),
                    normalize_handle(&edge.target_handle),
                )],
            );
        }
        Ok(())
    }

    pub fn remove_agent(&self, app: &AppHandle, agent_id: &str) -> Result<()> {
        // remove from edges
        {
            let mut edges = self.edges.lock().unwrap();
            let mut sources_to_remove = Vec::new();
            for (source, targets) in edges.iter_mut() {
                targets.retain(|(target, _, _)| target != agent_id);
                if targets.is_empty() {
                    sources_to_remove.push(source.clone());
                }
            }
            for source in sources_to_remove {
                edges.remove(&source);
            }
            edges.remove(agent_id);
        }

        self.stop_agent(app, agent_id)?;

        // remove from agents
        {
            let mut agents = self.agents.lock().unwrap();
            agents.remove(agent_id);
        }

        Ok(())
    }

    pub fn remove_edge(&self, edge: &AgentFlowEdge) {
        let mut edges = self.edges.lock().unwrap();
        if let Some(targets) = edges.get_mut(&edge.source) {
            targets.retain(|(target, source_handle, target_handle)| {
                *target != edge.target
                    || *source_handle != edge.source_handle
                    || *target_handle != edge.target_handle
            });
            if targets.is_empty() {
                edges.remove(&edge.source);
            }
        }
    }

    pub fn start_agent_flow(&self, app: &AppHandle, name: &str) -> Result<()> {
        let flows = self.flows.lock().unwrap();
        let Some(flow) = flows.get(name) else {
            bail!("Agent flow {} not found", name);
        };
        let mut agents = self.agents.lock().unwrap();
        for node in flow.nodes.iter() {
            if !node.enabled {
                continue;
            }
            if let Some(agent) = agents.get_mut(&node.id) {
                if *agent.status() != agent::AgentStatus::Init {
                    bail!("Agent {} is not in Init state", node.id);
                }
                agent.start(app)?;
            }
        }
        Ok(())
    }

    pub fn stop_agent(&self, app: &AppHandle, agent_id: &str) -> Result<()> {
        let mut agents = self.agents.lock().unwrap();
        if let Some(agent) = agents.get_mut(agent_id) {
            if *agent.status() == agent::AgentStatus::Run {
                log::info!("Stopping agent {}", agent_id);
                agent.stop(app)?;
            }
        }
        Ok(())
    }
}

fn normalize_handle(handle: &str) -> String {
    let mut handle = handle;

    // "" -> "*"
    if handle.is_empty() {
        handle = "*";
    }

    handle.to_string()
}

#[tauri::command]
pub fn get_agent_defs_cmd(env: State<AgentEnv>) -> Result<Value, String> {
    let defs: HashMap<String, AgentDefinition>;
    {
        let env_defs = env.defs.lock().unwrap();
        defs = env_defs.clone();
    }
    let value = serde_json::to_value(defs).map_err(|e| e.to_string())?;
    Ok(value)
}
