use anyhow::Result;
use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager};

use super::agent::{AgentConfig, AgentData, AsAgent};
use super::env::AgentEnv;
use super::message::try_send_board;

const EMIT_PUBLISH: &str = "mnemnk:write_board";

#[derive(Clone, Debug, Serialize)]
struct WriteBoardMessage {
    kind: String,
    value: Value,
}

pub struct BoardInAgent {
    data: AgentData,
    board_name: Option<String>,
}

impl BoardInAgent {
    pub fn new(id: String, def_name: String, config: Option<AgentConfig>) -> Result<Self> {
        let board_name = config.as_ref().and_then(normalize_board_name);
        Ok(Self {
            data: AgentData {
                id,
                status: Default::default(),
                def_name,
                config,
            },
            board_name,
        })
    }
}

impl AsAgent for BoardInAgent {
    fn data(&self) -> &AgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AgentData {
        &mut self.data
    }

    fn set_config(&mut self, _app: &AppHandle, config: AgentConfig) -> Result<()> {
        self.board_name = normalize_board_name(&config);
        self.data.config = Some(config);
        Ok(())
    }

    fn input(&mut self, app: &AppHandle, kind: String, value: Value) -> Result<()> {
        let mut board_name = self.board_name.clone().unwrap_or_default();
        if board_name.is_empty() {
            // if board_name is not set, stop processing
            return Ok(());
        }
        if board_name == "*" {
            if kind.is_empty() {
                // kind should not be empty, but just in case
                return Ok(());
            }
            board_name = kind;
        }
        {
            let env = app.state::<AgentEnv>();
            let mut board_values = env.board_values.lock().unwrap();
            board_values.insert(board_name.clone(), value.clone());
        }
        let app = app.clone();
        let env = app.state::<AgentEnv>();
        try_send_board(&env, board_name.clone(), value.clone());

        emit_publish(&app, board_name, value);

        Ok(())
    }
}

pub struct BoardOutAgent {
    data: AgentData,
    board_name: Option<String>,
}

impl BoardOutAgent {
    pub fn new(id: String, def_name: String, config: Option<AgentConfig>) -> Result<Self> {
        let board_name = config.as_ref().and_then(normalize_board_name);
        Ok(Self {
            data: AgentData {
                id,
                status: Default::default(),
                def_name,
                config,
            },
            board_name,
        })
    }
}

impl AsAgent for BoardOutAgent {
    fn data(&self) -> &AgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AgentData {
        &mut self.data
    }

    fn start(&mut self, app: &AppHandle) -> Result<()> {
        if let Some(board_name) = &self.board_name {
            let env = app.state::<AgentEnv>();
            let mut board_nodes = env.board_nodes.lock().unwrap();
            if let Some(nodes) = board_nodes.get_mut(board_name) {
                nodes.push(self.data.id.clone());
            } else {
                board_nodes.insert(board_name.clone(), vec![self.data.id.clone()]);
            }
        }
        Ok(())
    }

    fn stop(&mut self, app: &AppHandle) -> Result<()> {
        if let Some(board_name) = &self.board_name {
            let env = app.state::<AgentEnv>();
            let mut board_nodes = env.board_nodes.lock().unwrap();
            if let Some(nodes) = board_nodes.get_mut(board_name) {
                nodes.retain(|x| x != &self.data.id);
            }
        }
        Ok(())
    }

    fn set_config(&mut self, app: &AppHandle, config: AgentConfig) -> Result<()> {
        let board_name = normalize_board_name(&config);
        if self.board_name != board_name {
            if let Some(board_name) = &self.board_name {
                let env = app.state::<AgentEnv>();
                let mut board_nodes = env.board_nodes.lock().unwrap();
                if let Some(nodes) = board_nodes.get_mut(board_name) {
                    nodes.retain(|x| x != &self.data.id);
                }
            }
            if let Some(board_name) = &board_name {
                let env = app.state::<AgentEnv>();
                let mut board_nodes = env.board_nodes.lock().unwrap();
                if let Some(nodes) = board_nodes.get_mut(board_name) {
                    nodes.push(self.data.id.clone());
                } else {
                    board_nodes.insert(board_name.clone(), vec![self.data.id.clone()]);
                }
            }
            self.board_name = board_name;
        }
        self.data.config = Some(config);
        Ok(())
    }

    fn input(&mut self, _app: &AppHandle, _kind: String, _value: Value) -> Result<()> {
        Ok(())
    }
}

fn normalize_board_name(config: &AgentConfig) -> Option<String> {
    let Some(board_name) = config.get("board_name") else {
        // board_name is not set
        return None;
    };
    let board_name = board_name.as_str().unwrap_or_default().trim();
    if board_name.is_empty() {
        // board_name is empty
        return None;
    }
    return Some(board_name.to_string());
}

fn emit_publish(app: &AppHandle, kind: String, value: Value) {
    // remove image from the value. it's too big to send to frontend
    let mut value = value;
    if value.get("image").is_some() {
        value.as_object_mut().unwrap().remove("image");
    }

    // emit the message to frontend
    let message = WriteBoardMessage { kind, value };
    app.emit(EMIT_PUBLISH, Some(message)).unwrap_or_else(|e| {
        log::error!("Failed to emit message: {}", e);
    });
}
