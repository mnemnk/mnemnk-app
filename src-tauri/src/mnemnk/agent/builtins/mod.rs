use super::AgentDefinitions;

mod api;
mod board;
mod command;
mod data;
mod database;
mod display;
mod file;
mod filter;
mod image;
mod input;
mod operator;
mod rhai_script;
mod rig;
mod stream;
mod string;
mod time;
mod utils;

pub(super) use command::CommandAgent;

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    api::init_agent_defs(defs);
    board::init_agent_defs(defs);
    data::init_agent_defs(defs);
    database::init_agent_defs(defs);
    display::init_agent_defs(defs);
    file::init_agent_defs(defs);
    filter::init_agent_defs(defs);
    image::init_agent_defs(defs);
    input::init_agent_defs(defs);
    operator::init_agent_defs(defs);
    rhai_script::init_agent_defs(defs);
    rig::init_agent_defs(defs);
    stream::init_agent_defs(defs);
    string::init_agent_defs(defs);
    time::init_agent_defs(defs);
    utils::init_agent_defs(defs);
}
