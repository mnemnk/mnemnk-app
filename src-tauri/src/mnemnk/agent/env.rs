use anyhow::{bail, Context as _, Result};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_shell::process::CommandChild;
use tokio::sync::mpsc;

use crate::mnemnk::settings;

use super::agent::{self, AgentMessage, AsyncAgent};
use super::config::AgentConfig;
use super::data::AgentData;
use super::definition::{init_agent_defs, AgentDefaultConfig, AgentDefinitions};
use super::flow::{AgentFlow, AgentFlowEdge, AgentFlowNode, AgentFlows};
use super::message::{self, EnvAgentMessage};
use super::AgentContext;

const EMIT_DISPLAY: &str = "mnemnk:display";
const EMIT_ERROR: &str = "mnemnk:error";
const EMIT_INPUT: &str = "mnemnk:input";

#[derive(Clone)]
pub enum AgentMessageSender {
    Sync(std::sync::mpsc::Sender<AgentMessage>),
    Async(mpsc::Sender<AgentMessage>),
}

pub struct AgentEnv {
    // AppHandle
    app: AppHandle,

    // agent flows
    pub flows: Mutex<AgentFlows>,

    // agent def name -> agent definition
    pub defs: Mutex<AgentDefinitions>,

    // agent id -> agent
    pub agents: Mutex<HashMap<String, Arc<Mutex<Box<dyn AsyncAgent>>>>>,

    // agent id -> sender
    pub agent_txs: Mutex<HashMap<String, AgentMessageSender>>,

    // sourece agent id -> [target agent id / source handle / target handle]
    pub edges: Mutex<HashMap<String, Vec<(String, String, String)>>>,

    // agent id -> child process
    pub commands: Mutex<HashMap<String, CommandChild>>,

    // board name -> [board out agent id]
    pub board_out_agents: Mutex<HashMap<String, Vec<String>>>,

    // board name -> data
    pub board_data: Mutex<HashMap<String, AgentData>>,

    // Rhai engine
    pub rhai_engine: rhai::Engine,

    // message sender
    pub tx: Mutex<Option<mpsc::Sender<EnvAgentMessage>>>,
}

impl AgentEnv {
    fn new(app: AppHandle) -> Self {
        Self {
            app,
            flows: Default::default(),
            defs: Default::default(),
            agents: Default::default(),
            agent_txs: Default::default(),
            edges: Default::default(),
            commands: Default::default(),
            board_out_agents: Default::default(),
            board_data: Default::default(),
            rhai_engine: rhai::Engine::new(),
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
                use EnvAgentMessage::*;

                match message {
                    AgentOut { agent, ctx, data } => {
                        message::agent_out(&app_handle, agent, ctx, data).await;
                    }
                    BoardOut { name, ctx, data } => {
                        message::board_out(&app_handle, name, ctx, data).await;
                    }
                }
            }
        });

        Ok(())
    }

    pub fn get_agent_default_config(&self, def_name: &str) -> Option<AgentDefaultConfig> {
        let defs = self.defs.lock().unwrap();
        let Some(def) = defs.get(def_name) else {
            return None;
        };
        def.default_config.clone()
    }

    pub fn new_agent_flow(&self, name: &str) -> Result<AgentFlow> {
        if !AgentEnv::is_valid_flow_name(name) {
            bail!("Invalid flow name: {}", name);
        }

        let new_name = self.unique_flow_name(name);
        let mut flows = self.flows.lock().unwrap();
        let mut flow = AgentFlow::default();
        flow.name = Some(new_name.clone());
        flows.insert(new_name, flow.clone());
        Ok(flow)
    }

    pub fn rename_agent_flow(&self, old_name: &str, new_name: &str) -> Result<String> {
        if !AgentEnv::is_valid_flow_name(new_name) {
            bail!("Invalid flow name: {}", new_name);
        }

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

    fn is_valid_flow_name(new_name: &str) -> bool {
        // Check if the name is empty
        if new_name.trim().is_empty() {
            return false;
        }

        // Checks for path-like names:
        if new_name.contains('/') {
            // Disallow leading, trailing, or consecutive slashes
            if new_name.starts_with('/') || new_name.ends_with('/') || new_name.contains("//") {
                return false;
            }
            // Disallow segments that starts with "."
            if new_name.split('/').any(|segment| segment.starts_with('.')) {
                return false;
            }
        }

        // Check if the name contains invalid characters
        let invalid_chars = ['\\', ':', '*', '?', '"', '<', '>', '|'];
        for c in invalid_chars {
            if new_name.contains(c) {
                return false;
            }
        }

        true
    }

    fn unique_flow_name(&self, name: &str) -> String {
        let flows = self.flows.lock().unwrap();
        let mut new_name = name.trim().to_string();
        let mut i = 2;
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
            agents.insert(node.id.clone(), Arc::new(Mutex::new(agent)));
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
            if targets
                .iter()
                .any(|(target, source_handle, target_handle)| {
                    *target == edge.target
                        && *source_handle == edge.source_handle
                        && *target_handle == edge.target_handle
                })
            {
                bail!("Edge already exists");
            }
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
        let agent = {
            let agents = self.agents.lock().unwrap();
            let Some(a) = agents.get(agent_id) else {
                bail!("Agent {} not found", agent_id);
            };
            a.clone()
        };
        let def_name = {
            let agent = agent.lock().unwrap();
            agent.def_name().to_string()
        };
        let uses_native_thread = {
            let defs = self.defs.lock().unwrap();
            let Some(def) = defs.get(&def_name) else {
                bail!("Agent {} definition not found", agent_id);
            };
            def.native_thread.unwrap_or(false)
        };
        let agent_status = {
            let agent = agent.lock().unwrap();
            agent.status().clone()
        };
        if agent_status == agent::AgentStatus::Init {
            log::info!("Starting agent {}", agent_id);

            if uses_native_thread {
                let (tx, rx) = std::sync::mpsc::channel();

                {
                    let mut agent_txs = self.agent_txs.lock().unwrap();
                    agent_txs.insert(agent_id.to_string(), AgentMessageSender::Sync(tx.clone()));
                };

                let agent_id = agent_id.to_string();
                std::thread::spawn(move || {
                    if let Err(e) = agent.lock().unwrap().start() {
                        log::error!("Failed to start agent {}: {}", agent_id, e);
                    }

                    while let Ok(message) = rx.recv() {
                        match message {
                            AgentMessage::Input { ctx, data } => {
                                agent
                                    .lock()
                                    .unwrap()
                                    .process(ctx, data)
                                    .unwrap_or_else(|e| {
                                        log::error!("Process Error {}: {}", agent_id, e);
                                    });
                            }
                            AgentMessage::Config { config } => {
                                agent
                                    .lock()
                                    .unwrap()
                                    .set_config(config)
                                    .unwrap_or_else(|e| {
                                        log::error!("Config Error {}: {}", agent_id, e);
                                    });
                            }
                            AgentMessage::Stop => {
                                break;
                            }
                        }
                    }
                });
            } else {
                let (tx, mut rx) = mpsc::channel(32);

                {
                    let mut agent_txs = self.agent_txs.lock().unwrap();
                    agent_txs.insert(agent_id.to_string(), AgentMessageSender::Async(tx.clone()));
                };

                let agent_id = agent_id.to_string();
                tauri::async_runtime::spawn(async move {
                    if let Err(e) = agent.lock().unwrap().start() {
                        log::error!("Failed to start agent {}: {}", agent_id, e);
                    }

                    while let Some(message) = rx.recv().await {
                        match message {
                            AgentMessage::Input { ctx, data } => {
                                agent
                                    .lock()
                                    .unwrap()
                                    .process(ctx, data)
                                    .unwrap_or_else(|e| {
                                        log::error!("Process Error {}: {}", agent_id, e);
                                    });
                            }
                            AgentMessage::Config { config } => {
                                agent
                                    .lock()
                                    .unwrap()
                                    .set_config(config)
                                    .unwrap_or_else(|e| {
                                        log::error!("Config Error {}: {}", agent_id, e);
                                    });
                            }
                            AgentMessage::Stop => {
                                rx.close();
                                return;
                            }
                        }
                    }
                });
            }
        }
        Ok(())
    }

    pub fn stop_agent(&self, agent_id: &str) -> Result<()> {
        let agent = {
            let agents = self.agents.lock().unwrap();
            let Some(a) = agents.get(agent_id) else {
                bail!("Agent {} not found", agent_id);
            };
            a.clone()
        };

        let agent_status = {
            let agent = agent.lock().unwrap();
            agent.status().clone()
        };
        if agent_status == agent::AgentStatus::Start {
            log::info!("Stopping agent {}", agent_id);

            {
                let mut agent_txs = self.agent_txs.lock().unwrap();
                if let Some(tx) = agent_txs.remove(agent_id) {
                    match tx {
                        AgentMessageSender::Sync(tx) => {
                            tx.send(AgentMessage::Stop).unwrap_or_else(|e| {
                                log::error!(
                                    "Failed to send stop message to agent {}: {}",
                                    agent_id,
                                    e
                                );
                            });
                        }
                        AgentMessageSender::Async(tx) => {
                            tx.try_send(AgentMessage::Stop).unwrap_or_else(|e| {
                                log::error!(
                                    "Failed to send stop message to agent {}: {}",
                                    agent_id,
                                    e
                                );
                            });
                        }
                    }
                }
            }

            agent.lock().unwrap().stop()?;
        }

        Ok(())
    }

    pub async fn set_agent_config(&self, agent_id: &str, config: AgentConfig) -> Result<()> {
        let agent = {
            let agents = self.agents.lock().unwrap();
            let Some(a) = agents.get(agent_id) else {
                bail!("Agent {} not found", agent_id);
            };
            a.clone()
        };

        let agent_status = {
            let agent = agent.lock().unwrap();
            agent.status().clone()
        };
        if agent_status == agent::AgentStatus::Init {
            agent.lock().unwrap().set_config(config.clone())?;
        } else if agent_status == agent::AgentStatus::Start {
            let tx = {
                let agent_txs = self.agent_txs.lock().unwrap();
                let Some(tx) = agent_txs.get(agent_id) else {
                    bail!("Agent tx for {} not found", agent_id);
                };
                tx.clone()
            };
            let message = AgentMessage::Config { config };
            match tx {
                AgentMessageSender::Sync(tx) => {
                    tx.send(message).context("Failed to send config message")?;
                }
                AgentMessageSender::Async(tx) => {
                    tx.send(message)
                        .await
                        .context("Failed to send config message")?;
                }
            }
        }
        Ok(())
    }

    pub async fn agent_input(
        &self,
        agent_id: &str,
        ctx: AgentContext,
        data: AgentData,
    ) -> Result<()> {
        let agent = {
            let agents = self.agents.lock().unwrap();
            let Some(a) = agents.get(agent_id) else {
                bail!("Agent {} not found", agent_id);
            };
            a.clone()
        };

        let agent_status = {
            let agent = agent.lock().unwrap();
            agent.status().clone()
        };
        if agent_status == agent::AgentStatus::Start {
            let ch = ctx.ch().to_string();
            let message = AgentMessage::Input { ctx, data };

            let tx = {
                let agent_txs = self.agent_txs.lock().unwrap();
                let Some(tx) = agent_txs.get(agent_id) else {
                    bail!("Agent tx for {} not found", agent_id);
                };
                tx.clone()
            };
            match tx {
                AgentMessageSender::Sync(tx) => {
                    tx.send(message).context("Failed to send input message")?;
                }
                AgentMessageSender::Async(tx) => {
                    tx.send(message)
                        .await
                        .context("Failed to send input message")?;
                }
            }

            self.emit_input(agent_id.to_string(), ch)
                .unwrap_or_else(|e| {
                    log::error!("Failed to emit input message: {}", e);
                });
        }
        Ok(())
    }

    pub async fn send_agent_out(
        &self,
        agent_id: String,
        ctx: AgentContext,
        data: AgentData,
    ) -> Result<()> {
        message::send_agent_out(self, agent_id, ctx, data).await
    }

    pub fn try_send_agent_out(
        &self,
        agent_id: String,
        ctx: AgentContext,
        data: AgentData,
    ) -> Result<()> {
        message::try_send_agent_out(self, agent_id, ctx, data)
    }

    pub fn try_send_board_out(
        &self,
        name: String,
        ctx: AgentContext,
        data: AgentData,
    ) -> Result<()> {
        message::try_send_board_out(self, name, ctx, data)
    }

    // Returns a list of agent IDs connected to the given agent's input handles
    #[allow(unused)]
    fn connected_input_agents(&self, agent_id: &str) -> Result<HashMap<String, Vec<String>>> {
        let mut result = HashMap::new();

        // Look through all edges to find connections to this agent's input handles
        let edges = self.edges.lock().unwrap();
        for (source_id, targets) in edges.iter() {
            for (target_id, _source_handle, target_handle) in targets {
                if target_id == agent_id {
                    // This edge connects to one of our input handles
                    if !result.contains_key(target_handle) {
                        result.insert(target_handle.clone(), Vec::new());
                    }
                    result
                        .get_mut(target_handle)
                        .unwrap()
                        .push(source_id.clone());
                }
            }
        }

        Ok(result)
    }

    // Returns a list of agent IDs connected to the given agent's output handles
    #[allow(unused)]
    fn connected_output_agents(&self, agent_id: &str) -> Result<HashMap<String, Vec<String>>> {
        let mut result = HashMap::new();

        // Look for edges where this agent is the source
        let edges = self.edges.lock().unwrap();
        if let Some(targets) = edges.get(agent_id) {
            for (target_id, source_handle, _target_handle) in targets {
                if !result.contains_key(source_handle) {
                    result.insert(source_handle.clone(), Vec::new());
                }
                result
                    .get_mut(source_handle)
                    .unwrap()
                    .push(target_id.clone());
            }
        }

        Ok(result)
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

    pub fn emit_error(&self, agent_id: String, message: String) -> Result<()> {
        #[derive(Clone, Serialize)]
        struct ErrorMessage {
            agent_id: String,
            message: String,
        }

        self.app
            .emit(EMIT_ERROR, ErrorMessage { agent_id, message })
            .context("Failed to emit error message")?;

        Ok(())
    }

    pub fn emit_input(&self, agent_id: String, ch: String) -> Result<()> {
        #[derive(Clone, Serialize)]
        struct InputMessage {
            agent_id: String,
            ch: String,
        }

        self.app
            .emit(EMIT_INPUT, InputMessage { agent_id, ch })
            .context("Failed to emit input message")?;

        Ok(())
    }

    pub fn emit_display(&self, agent_id: String, key: String, data: AgentData) -> Result<()> {
        #[derive(Clone, Serialize)]
        struct DisplayMessage {
            agent_id: String,
            key: String,
            data: AgentData,
        }

        self.app
            .emit(
                EMIT_DISPLAY,
                DisplayMessage {
                    agent_id,
                    key,
                    data,
                },
            )
            .context("Failed to emit display message")?;

        Ok(())
    }
}
