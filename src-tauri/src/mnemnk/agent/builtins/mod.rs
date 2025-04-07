use super::AgentDefinitions;

mod board;
mod core;
mod database;
mod display;
mod input;
mod string;

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    board::init_agent_defs(defs);
    core::init_agent_defs(defs);
    database::init_agent_defs(defs);
    display::init_agent_defs(defs);
    input::init_agent_defs(defs);
    string::init_agent_defs(defs);
}
