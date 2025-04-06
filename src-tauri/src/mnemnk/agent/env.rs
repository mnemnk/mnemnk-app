use anyhow::{bail, Context as _, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_shell::process::CommandChild;
use tokio::sync::mpsc;

use crate::mnemnk::settings;

use super::agent::{self, AgentConfig, AsyncAgent};
use super::definition::{init_agent_defs, AgentDefinition, AgentDefinitions};
use super::flow::{AgentFlow, AgentFlowEdge, AgentFlowNode, AgentFlows};
use super::message::{self, AgentMessage};

pub struct AgentEnv {
    // AppHandle
    app: AppHandle,

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
    fn new(app: AppHandle, tx: mpsc::Sender<AgentMessage>) -> Self {
        Self {
            app,
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
        let env = Self::new(app.clone(), tx);

        let agent_defs = init_agent_defs(app)?;
        settings::init_agent_global_configs(app, &agent_defs)?;
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

    pub fn rename_agent_flow(&self, old_name: &str, new_name: &str) -> Result<String> {
        // check if the new name is already used
        let new_name = self.unique_flow_name(new_name);

        let mut flows = self.flows.lock().unwrap();

        // remove the original flow
        let Some(mut flow) = flows.remove(old_name) else {
            bail!(
                "env::rename_agent_flow: Agent flow {} could not remove",
                old_name
            );
        };

        // insert renamed flow
        flow.name = Some(new_name.clone());
        flows.insert(new_name.clone(), flow);
        Ok(new_name)
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
        if let Ok(agent) = agent::agent_new(
            self.app.clone(),
            self,
            node.id.clone(),
            &node.name,
            node.config.clone(),
        ) {
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
                edge.source_handle.clone(),
                edge.target_handle.clone(),
            ));
        } else {
            edges.insert(
                edge.source.clone(),
                vec![(
                    edge.target.clone(),
                    edge.source_handle.clone(),
                    edge.target_handle.clone(),
                )],
            );
        }
        Ok(())
    }

    pub fn remove_agent(&self, agent_id: &str) -> Result<()> {
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

        self.stop_agent(agent_id)?;

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

    pub fn start_agent_flow(&self, name: &str) -> Result<()> {
        let flows = self.flows.lock().unwrap();
        let Some(flow) = flows.get(name) else {
            bail!("Agent flow {} not found", name);
        };
        for node in flow.nodes.iter() {
            if !node.enabled {
                continue;
            }
            self.start_agent(&node.id).unwrap_or_else(|e| {
                log::error!("Failed to start agent {}: {}", node.id, e);
            });
        }
        Ok(())
    }

    pub fn start_agent(&self, agent_id: &str) -> Result<()> {
        let mut agents = self.agents.lock().unwrap();
        let Some(agent) = agents.get_mut(agent_id) else {
            bail!("Agent {} not found", agent_id);
        };
        if *agent.status() == agent::AgentStatus::Init {
            log::info!("Starting agent {}", agent_id);
            agent.start()?;
        }
        Ok(())
    }

    pub fn stop_agent(&self, agent_id: &str) -> Result<()> {
        let mut agents = self.agents.lock().unwrap();
        let Some(agent) = agents.get_mut(agent_id) else {
            bail!("Agent {} not found", agent_id);
        };
        if *agent.status() == agent::AgentStatus::Run
            || *agent.status() == agent::AgentStatus::Start
        {
            log::info!("Stopping agent {}", agent_id);
            agent.stop()?;
        }
        Ok(())
    }

    pub fn set_agent_config(&self, agent_id: &str, config: AgentConfig) -> Result<()> {
        let mut agents = self.agents.lock().unwrap();
        let Some(agent) = agents.get_mut(agent_id) else {
            bail!("Agent {} not found", agent_id);
        };
        agent.set_config(config)
    }

    pub async fn send_agent_out(&self, agent_id: String, kind: String, value: Value) -> Result<()> {
        message::send_agent_out(self, agent_id, kind, value).await
    }

    pub fn try_send_agent_out(&self, agent_id: String, kind: String, value: Value) -> Result<()> {
        message::try_send_agent_out(self, agent_id, kind, value)
    }

    pub fn try_send_board_out(&self, kind: String, value: Value) -> Result<()> {
        message::try_send_board_out(self, kind, value)
    }
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
