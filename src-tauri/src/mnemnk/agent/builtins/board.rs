use anyhow::{Context as _, Result};
use tauri::AppHandle;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::{
    Agent, AgentConfig, AgentConfigEntry, AgentData, AgentDefinition, AgentDefinitions, AgentValue,
    AsAgent, AsAgentData,
};

// const EMIT_PUBLISH: &str = "mnemnk:write_board";

struct BoardInAgent {
    data: AsAgentData,
    board_name: Option<String>,
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
            data: AsAgentData::new(app, id, def_name, config),
            board_name,
        })
    }

    fn data(&self) -> &AsAgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AsAgentData {
        &mut self.data
    }

    fn set_config(&mut self, config: AgentConfig) -> Result<()> {
        self.board_name = normalize_board_name(&config);
        Ok(())
    }

    fn process(&mut self, ch: String, data: AgentData) -> Result<()> {
        let mut board_name = self.board_name.clone().unwrap_or_default();
        if board_name.is_empty() {
            // if board_name is not set, stop processing
            return Ok(());
        }
        if board_name == "*" {
            if ch.is_empty() {
                // ch should not be empty, but just in case
                return Ok(());
            }
            board_name = ch;
        }
        let env = self.env();
        {
            let mut board_data = env.board_data.lock().unwrap();
            board_data.insert(board_name.clone(), data.clone());
        }
        env.try_send_board_out(board_name.clone(), data.clone())
            .context("Failed to send board")?;

        Ok(())
    }
}

struct BoardOutAgent {
    data: AsAgentData,
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
            data: AsAgentData::new(app, id, def_name, config),
            board_name,
        })
    }

    fn data(&self) -> &AsAgentData {
        &self.data
    }

    fn mut_data(&mut self) -> &mut AsAgentData {
        &mut self.data
    }

    fn start(&mut self) -> Result<()> {
        if let Some(board_name) = &self.board_name {
            let env = self.env();
            let mut board_out_agents = env.board_out_agents.lock().unwrap();
            if let Some(nodes) = board_out_agents.get_mut(board_name) {
                nodes.push(self.data.id.clone());
            } else {
                board_out_agents.insert(board_name.clone(), vec![self.data.id.clone()]);
            }
        }
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        if let Some(board_name) = &self.board_name {
            let env = self.env();
            let mut board_out_agents = env.board_out_agents.lock().unwrap();
            if let Some(nodes) = board_out_agents.get_mut(board_name) {
                nodes.retain(|x| x != &self.data.id);
            }
        }
        Ok(())
    }

    fn set_config(&mut self, config: AgentConfig) -> Result<()> {
        let board_name = normalize_board_name(&config);
        if self.board_name != board_name {
            if let Some(board_name) = &self.board_name {
                let env = self.env();
                let mut board_out_agents = env.board_out_agents.lock().unwrap();
                if let Some(nodes) = board_out_agents.get_mut(board_name) {
                    nodes.retain(|x| x != &self.data.id);
                }
            }
            if let Some(board_name) = &board_name {
                let env = self.env();
                let mut board_out_agents = env.board_out_agents.lock().unwrap();
                if let Some(nodes) = board_out_agents.get_mut(board_name) {
                    nodes.push(self.data.id.clone());
                } else {
                    board_out_agents.insert(board_name.clone(), vec![self.data.id.clone()]);
                }
            }
            self.board_name = board_name;
        }
        Ok(())
    }

    fn process(&mut self, _ch: String, _data: AgentData) -> Result<()> {
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
        AgentDefinition::new("Board", "$board_in", Some(new_boxed::<BoardInAgent>))
            .with_title("Board In")
            .with_category("Core")
            .with_inputs(vec!["*"])
            .with_default_config(vec![(
                "board_name".into(),
                AgentConfigEntry::new(AgentValue::new_string("".to_string()), "string")
                    .with_title("Board Name")
                    .with_description("* = source kind"),
            )]),
    );

    // BoardOutAgent
    defs.insert(
        "$board_out".into(),
        AgentDefinition::new("Board", "$board_out", Some(new_boxed::<BoardOutAgent>))
            .with_title("Board Out")
            .with_category("Core")
            .with_outputs(vec!["*"])
            .with_default_config(vec![(
                "board_name".into(),
                AgentConfigEntry::new(AgentValue::new_string("".to_string()), "string")
                    .with_title("Board Name"),
            )]),
    );
}
