use super::AgentDefinitions;

mod api;
mod board;
mod command;
mod core;
mod database;
mod display;
mod input;
mod rhai_script;
mod string;
mod time;
mod utils;

pub(super) use command::CommandAgent;

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    api::init_agent_defs(defs);
    board::init_agent_defs(defs);
    core::init_agent_defs(defs);
    database::init_agent_defs(defs);
    display::init_agent_defs(defs);
    input::init_agent_defs(defs);
    rhai_script::init_agent_defs(defs);
    string::init_agent_defs(defs);
    time::init_agent_defs(defs);
    utils::init_agent_defs(defs);
}
