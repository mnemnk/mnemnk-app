use anyhow::Result;
use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager};

use super::agent::{AgentConfig, AgentData, AsAgent};
use super::env::AgentEnv;
use super::message::send_board;

const EMIT_PUBLISH: &str = "mnemnk:write_board";

#[derive(Clone, Debug, Serialize)]
struct WriteBoardMessage {
    agent: String,
    kind: String,
    value: Value,
}

pub struct BoardAgent {
    data: AgentData,
    board_name: Option<String>,
}

impl AsAgent for BoardAgent {
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

    fn update(&mut self, app: &AppHandle, config: Option<AgentConfig>) -> Result<()> {
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
        self.data.config = config;
        Ok(())
    }

    fn input(&self, app: &AppHandle, source: String, kind: String, value: Value) -> Result<()> {
        let kind = self.board_name.clone().unwrap_or(kind.to_string());
        if kind.is_empty() {
            return Ok(());
        }

        {
            let env = app.state::<AgentEnv>();
            let mut board_values = env.board_values.lock().unwrap();
            board_values.insert(kind.clone(), value.clone());
        }
        let app = app.clone();
        let env = app.state::<AgentEnv>();
        send_board(&env, source.clone(), kind.clone(), value.clone());

        emit_publish(source, value, kind, app);

        Ok(())
    }
}

fn emit_publish(source: String, value: Value, kind: String, app: AppHandle) {
    // remove image from the value. it's too big to send to frontend
    let mut value = value;
    if value.get("image").is_some() {
        value.as_object_mut().unwrap().remove("image");
    }

    // emit the message to frontend
    let message = WriteBoardMessage {
        agent: source,
        kind,
        value,
    };
    app.emit(EMIT_PUBLISH, Some(message)).unwrap_or_else(|e| {
        log::error!("Failed to emit message: {}", e);
    });
}

impl BoardAgent {
    pub fn new(id: String, def_name: String, config: Option<AgentConfig>) -> Result<Self> {
        let board_name = normalize_board_name(&config);
        Ok(Self {
            data: AgentData {
                id,
                def_name,
                config,
            },
            board_name,
        })
    }
}

fn normalize_board_name(config: &Option<AgentConfig>) -> Option<String> {
    let Some(config) = config else {
        // config is not set
        return None;
    };
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
