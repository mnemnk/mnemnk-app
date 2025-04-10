use anyhow::{bail, Context as _, Result};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tauri_plugin_shell::process::CommandChild;
use tokio::sync::mpsc;

use crate::mnemnk::settings;

use super::agent::{self, AgentConfig, AgentData, AsyncAgent};
use super::definition::{init_agent_defs, AgentDefinitions};
use super::flow::{AgentFlow, AgentFlowEdge, AgentFlowNode, AgentFlows};
use super::message::{self, AgentMessage};

pub struct AgentEnv {
    // AppHandle
    app: AppHandle,

    // agent flows
    pub flows: Mutex<AgentFlows>,

    // agent def name -> agent definition
    pub defs: Mutex<AgentDefinitions>,

    // agent id -> agent
    pub agents: Mutex<HashMap<String, Box<dyn AsyncAgent>>>,

    // sourece agent id -> [target agent id / source handle / target handle]
    pub edges: Mutex<HashMap<String, Vec<(String, String, String)>>>,

    // agent id -> child process
    pub commands: Mutex<HashMap<String, CommandChild>>,

    // board name -> [board out agent id]
    pub board_out_agents: Mutex<HashMap<String, Vec<String>>>,

    // board name -> data
    pub board_data: Mutex<HashMap<String, AgentData>>,

    // message sender
    pub tx: Mutex<Option<mpsc::Sender<AgentMessage>>>,
}

impl AgentEnv {
    fn new(app: AppHandle) -> Self {
        Self {
            app,
            flows: Default::default(),
            defs: Default::default(),
            agents: Default::default(),
            edges: Default::default(),
            commands: Default::default(),
            board_out_agents: Default::default(),
            board_data: Default::default(),
            tx: Default::default(),
        }
    }

    pub fn init(app: &AppHandle) -> Result<()> {
        let env = Self::new(app.clone());

        let agent_defs = init_agent_defs(app)?;
        settings::init_agent_global_configs(app, &agent_defs)?;
        {
            let mut defs = env.defs.lock().unwrap();
            *defs = agent_defs;
        }

        app.manage(env);
        Ok(())
    }

    pub fn spawn_message_loop(&self) -> Result<()> {
        // TODO: settings for the channel size
        let (tx, mut rx) = mpsc::channel(4096);
        {
            let mut tx_lock = self.tx.lock().unwrap();
            *tx_lock = Some(tx);
        }

        // spawn the main loop
        let app_handle = self.app.clone();
        tauri::async_runtime::spawn(async move {
            while let Some(message) = rx.recv().await {
                use AgentMessage::*;

                match message {
                    AgentOut { agent, ch, data } => {
                        message::agent_out(&app_handle, agent, ch, data).await;
                    }
                    BoardOut { name, data } => {
                        message::board_out(&app_handle, name, data).await;
                    }
                    Store { data } => {
                        message::store(&app_handle, data);
                    }
                }
            }
        });

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
        // check if the source agent exists
        {
            let agents = self.agents.lock().unwrap();
            if !agents.contains_key(&edge.source) {
                bail!("Source agent {} not found", edge.source);
            }
        }

        // check if handles are valid
        if edge.source_handle.is_empty() {
            bail!("Source handle is empty");
        }
        if edge.target_handle.is_empty() {
            bail!("Target handle is empty");
        }

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

    pub async fn send_agent_out(
        &self,
        agent_id: String,
        ch: String,
        data: AgentData,
    ) -> Result<()> {
        message::send_agent_out(self, agent_id, ch, data).await
    }

    pub fn try_send_agent_out(&self, agent_id: String, ch: String, data: AgentData) -> Result<()> {
        message::try_send_agent_out(self, agent_id, ch, data)
    }

    pub fn try_send_board_out(&self, name: String, data: AgentData) -> Result<()> {
        message::try_send_board_out(self, name, data)
    }

    pub fn quit(&self) {
        {
            // send QUIT command to all agents
            let mut agent_commands = self.commands.lock().unwrap();
            let agent_ids = agent_commands.keys().cloned().collect::<Vec<String>>();
            for agent_id in agent_ids {
                log::info!("Stopping agent: {}", agent_id);
                // we cannot use stop_agent here because it will also try to lock aget_commands.
                if let Some(child) = agent_commands.get_mut(&agent_id) {
                    child.write(".QUIT\n".as_bytes()).unwrap_or_else(|e| {
                        log::error!("Failed to write to {}: {}", agent_id, e);
                    });
                }
            }
        }

        // wait for all agents to exit
        for _ in 0..20 {
            {
                let agent_commands = self.commands.lock().unwrap();
                if agent_commands.is_empty() {
                    return;
                }
            }
            std::thread::sleep(Duration::from_millis(500));
        }

        {
            // kill remaining agents
            let mut agent_commands = self.commands.lock().unwrap();
            let programs = agent_commands.keys().cloned().collect::<Vec<String>>();
            for program in programs {
                log::warn!("Killing agent: {}", program);
                if let Some(command) = agent_commands.remove(&program) {
                    command.kill().unwrap_or_else(|e| {
                        log::error!("Failed to kill agent: {} {}", program, e);
                    });
                }
            }
        }
    }
}
