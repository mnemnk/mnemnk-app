use anyhow::Result;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_shell::process::CommandChild;
use tokio::sync::mpsc;

use super::agent::AsyncAgent;
use super::definition::{init_agent_defs, AgentDefinition, AgentDefinitions};
use super::AgentMessage;

pub struct AgentEnv {
    pub defs: Mutex<AgentDefinitions>,

    pub nodes: Mutex<HashMap<String, Box<dyn AsyncAgent>>>,

    // enabled node ids
    pub enabled_nodes: Mutex<HashSet<String>>,

    // node id -> node ids / subscriber handle / target handle
    pub edges: Mutex<HashMap<String, Vec<(String, String, String)>>>,

    // node id -> child process
    pub commands: Mutex<HashMap<String, CommandChild>>,

    // node id -> board name
    pub board_names: Mutex<HashMap<String, String>>,

    // board name -> value
    pub board_values: Mutex<HashMap<String, Value>>,

    // message sender
    pub tx: mpsc::Sender<AgentMessage>,
}

impl AgentEnv {
    fn new(tx: mpsc::Sender<AgentMessage>) -> Self {
        Self {
            defs: Default::default(),
            nodes: Default::default(),
            enabled_nodes: Default::default(),
            edges: Default::default(),
            commands: Default::default(),
            board_names: Default::default(),
            board_values: Default::default(),
            tx,
        }
    }

    pub fn init(app: &AppHandle, tx: mpsc::Sender<AgentMessage>) -> Result<()> {
        let env = Self::new(tx);

        let agent_defs = init_agent_defs(app)?;
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
