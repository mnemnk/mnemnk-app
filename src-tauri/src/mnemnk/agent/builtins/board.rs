use anyhow::{Context as _, Result};
use serde::Serialize;
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter};

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentConfigEntry, AgentData, AgentDefinition, AgentDefinitions,
    AgentStatus, AsAgent,
};

const EMIT_PUBLISH: &str = "mnemnk:write_board";

#[derive(Clone, Debug, Serialize)]
struct WriteBoardMessage {
    kind: String,
    value: Value,
}

struct BoardInAgent {
    data: AgentData,
    board_name: Option<String>,
}

impl BoardInAgent {
    fn emit_publish(&self, kind: String, value: Value) {
        // remove image from the value. it's too big to send to frontend
        let mut value = value;
        if value.get("image").is_some() {
            value.as_object_mut().unwrap().remove("image");
        }

        // emit the message to frontend
        let message = WriteBoardMessage { kind, value };
        self.app()
            .emit(EMIT_PUBLISH, Some(message))
            .unwrap_or_else(|e| {
                log::error!("Failed to emit message: {}", e);
            });
    }
}

impl AsAgent for BoardInAgent {
    fn new(
        app: AppHandle,
        id: String,
        def_name: String,
        config: Option<AgentConfig>,
    ) -> Result<Self> {
        let board_name = config.as_ref().and_then(normalize_board_name);
        Ok(Self {
            data: AgentData {
                app,
                id,
                status: Default::default(),
                def_name,
                config,
            },
            board_name,
        })
    }

    fn data(&self) -> &AgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AgentData {
        &mut self.data
    }

    fn set_config(&mut self, config: AgentConfig) -> Result<()> {
        self.board_name = normalize_board_name(&config);
        self.data.config = Some(config);
        Ok(())
    }

    fn input(&mut self, kind: String, value: Value) -> Result<()> {
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
        let env = self.env();
        {
            let mut board_values = env.board_values.lock().unwrap();
            board_values.insert(board_name.clone(), value.clone());
        }
        env.try_send_board_out(board_name.clone(), value.clone())
            .context("Failed to send board")?;

        self.emit_publish(board_name, value);

        Ok(())
    }
}

struct BoardOutAgent {
    data: AgentData,
    board_name: Option<String>,
}

impl AsAgent for BoardOutAgent {
    fn new(
        app: AppHandle,
        id: String,
        def_name: String,
        config: Option<AgentConfig>,
    ) -> Result<Self> {
        let board_name = config.as_ref().and_then(normalize_board_name);
        Ok(Self {
            data: AgentData {
                app,
                id,
                status: Default::default(),
                def_name,
                config,
            },
            board_name,
        })
    }

    fn data(&self) -> &AgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AgentData {
        &mut self.data
    }

    fn start(&mut self) -> Result<()> {
        if let Some(board_name) = &self.board_name {
            let env = self.env();
            let mut board_nodes = env.board_nodes.lock().unwrap();
            if let Some(nodes) = board_nodes.get_mut(board_name) {
                nodes.push(self.data.id.clone());
            } else {
                board_nodes.insert(board_name.clone(), vec![self.data.id.clone()]);
            }
        }
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        if let Some(board_name) = &self.board_name {
            let env = self.env();
            let mut board_nodes = env.board_nodes.lock().unwrap();
            if let Some(nodes) = board_nodes.get_mut(board_name) {
                nodes.retain(|x| x != &self.data.id);
            }
        }
        Ok(())
    }

    fn set_config(&mut self, config: AgentConfig) -> Result<()> {
        let board_name = normalize_board_name(&config);
        if self.board_name != board_name {
            if *self.status() == AgentStatus::Run {
                if let Some(board_name) = &self.board_name {
                    let env = self.env();
                    let mut board_nodes = env.board_nodes.lock().unwrap();
                    if let Some(nodes) = board_nodes.get_mut(board_name) {
                        nodes.retain(|x| x != &self.data.id);
                    }
                }
                if let Some(board_name) = &board_name {
                    let env = self.env();
                    let mut board_nodes = env.board_nodes.lock().unwrap();
                    if let Some(nodes) = board_nodes.get_mut(board_name) {
                        nodes.push(self.data.id.clone());
                    } else {
                        board_nodes.insert(board_name.clone(), vec![self.data.id.clone()]);
                    }
                }
            }
            self.board_name = board_name;
        }
        self.data.config = Some(config);
        Ok(())
    }

    fn input(&mut self, _kind: String, _value: Value) -> Result<()> {
        // do nothing
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

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    // BoardInAgent
    defs.insert(
        "$board_in".into(),
        AgentDefinition::new(
            "BoardIn",
            "$board_in",
            Some(new_boxed::<super::board::BoardInAgent>),
        )
        .with_title("Board In")
        .with_category("Core")
        .with_inputs(vec!["*"])
        .with_default_config(vec![(
            "board_name".into(),
            AgentConfigEntry::new(json!(""), "string")
                .with_title("Board Name")
                .with_description("* = source kind"),
        )]),
    );

    // BoardOutAgent
    defs.insert(
        "$board_out".into(),
        AgentDefinition::new(
            "BoardOut",
            "$board_out",
            Some(new_boxed::<super::board::BoardOutAgent>),
        )
        .with_title("Board Out")
        .with_category("Core")
        .with_outputs(vec!["*"])
        .with_default_config(vec![(
            "board_name".into(),
            AgentConfigEntry::new(json!(""), "string").with_title("Board Name"),
        )]),
    );
}
