use anyhow::Result;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_shell::process::CommandChild;
use tokio::sync::mpsc;

use super::definition::{init_agent_defs, AgentDefinitions};
use super::AgentMessage;

// pub trait Agent {
//     fn id(&self) -> &str;
//     fn name(&self) -> &str;
//     fn enabled(&self) -> bool;
//     fn config(&self) -> Option<HashMap<String, Value>>;
// }

pub struct AgentEnv {
    pub defs: Mutex<AgentDefinitions>,

    // pub nodes: Mutex<HashMap<String, Box<dyn Agent + Send>>>,

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
    pub fn new(tx: mpsc::Sender<AgentMessage>) -> Self {
        Self {
            defs: Default::default(),
            // nodes: Default::default(),
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

        let agent_configs = init_agent_defs(app)?;
        {
            let mut configs = env.defs.lock().unwrap();
            *configs = agent_configs;
        }

        app.manage(env);
        Ok(())
    }
}

#[tauri::command]
pub fn get_agent_configs_cmd(env: State<AgentEnv>) -> Result<Value, String> {
    let configs;
    {
        let env_configs = env.defs.lock().unwrap();
        configs = env_configs.clone();
    }
    let value = serde_json::to_value(&configs).map_err(|e| e.to_string())?;
    Ok(value)
}
