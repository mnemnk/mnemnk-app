use anyhow::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_shell::process::CommandChild;
use tokio::sync::mpsc;

use crate::mnemnk::settings;

use super::agent::AsyncAgent;
use super::definition::{init_agent_defs, AgentDefinition, AgentDefinitions};
use super::message::AgentMessage;

pub struct AgentEnv {
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
