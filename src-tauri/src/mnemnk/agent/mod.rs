use anyhow::Result;
use serde_json::Value;
use tauri::{AppHandle, Manager, State};

mod agent;
mod builtins;
mod data;
mod definition;
mod env;
mod flow;
mod message;

pub use agent::{emit_error, Agent, AgentConfig, AgentConfigs, AgentStatus, AsAgent, AsAgentData};
pub use data::{AgentData, AgentValue};
pub use definition::{
    AgentConfigEntry, AgentDefinition, AgentDefinitionError, AgentDefinitions,
    AgentDisplayConfigEntry,
};
pub use env::AgentEnv;
pub use flow::{AgentFlow, AgentFlowEdge, AgentFlowNode};

pub fn init(app: &AppHandle) -> Result<()> {
    AgentEnv::init(app)?;
    flow::init(app)?;
    Ok(())
}

pub fn ready(app: &AppHandle) -> Result<()> {
    flow::ready(app)?;

    let env = app.state::<AgentEnv>();
    env.spawn_message_loop()?;

    Ok(())
}

pub fn quit(app: &AppHandle) {
    let env = app.state::<AgentEnv>();
    env.quit();
}

// Tauri Commands

#[tauri::command]
pub fn get_agent_defs_cmd(env: State<AgentEnv>) -> Result<Value, String> {
    let defs: AgentDefinitions;
    {
        let env_defs = env.defs.lock().unwrap();
        defs = env_defs.clone();
    }
    serde_json::to_value(defs).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn set_agent_config_cmd(
    app: AppHandle,
    agent_id: String,
    config: AgentConfig,
) -> Result<(), String> {
    let env = app.state::<AgentEnv>();
    env.set_agent_config(&agent_id, config)
        .await
        .map_err(|e| e.to_string())
}
#[tauri::command]
pub fn start_agent_cmd(env: State<AgentEnv>, agent_id: String) -> Result<(), String> {
    env.start_agent(&agent_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn stop_agent_cmd(env: State<AgentEnv>, agent_id: String) -> Result<(), String> {
    env.stop_agent(&agent_id).map_err(|e| e.to_string())
}

// flow commands

#[tauri::command]
pub fn get_agent_flows_cmd(env: State<AgentEnv>) -> Result<Value, String> {
    let agent_flows;
    {
        let flows = env.flows.lock().unwrap();
        agent_flows = flows.clone();
    }
    let value = serde_json::to_value(&agent_flows).map_err(|e| e.to_string())?;
    Ok(value)
}

#[tauri::command]
pub fn new_agent_flow_cmd(env: State<AgentEnv>, name: String) -> Result<AgentFlow, String> {
    let flow = env.new_agent_flow(&name).map_err(|e| e.to_string())?;
    Ok(flow)
}

#[tauri::command]
pub fn rename_agent_flow_cmd(
    app: AppHandle,
    env: State<AgentEnv>,
    old_name: String,
    new_name: String,
) -> Result<String, String> {
    flow::rename_agent_flow(&app, &env, &old_name, &new_name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn delete_agent_flow_cmd(
    app: AppHandle,
    env: State<AgentEnv>,
    name: String,
) -> Result<(), String> {
    flow::delete_agent_flow(&app, &env, &name).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn insert_agent_flow_cmd(env: State<AgentEnv>, agent_flow: AgentFlow) -> Result<(), String> {
    flow::insert_agent_flow(env, agent_flow).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_agent_flow_cmd(
    app: AppHandle,
    env: State<AgentEnv>,
    agent_flow: AgentFlow,
) -> Result<(), String> {
    flow::save_agent_flow(&app, env, agent_flow).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_agent_flow_cmd(env: State<AgentEnv>, path: String) -> Result<AgentFlow, String> {
    flow::import_agent_flow(&env, path).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_agent_flow_node_cmd(
    env: State<AgentEnv>,
    flow_name: String,
    node: AgentFlowNode,
) -> Result<(), String> {
    flow::add_agent_flow_node(&env, &flow_name, &node).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_agent_flow_node_cmd(
    env: State<AgentEnv>,
    flow_name: String,
    node_id: String,
) -> Result<(), String> {
    flow::remove_agent_flow_node(&env, &flow_name, &node_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn add_agent_flow_edge_cmd(
    env: State<AgentEnv>,
    flow_name: String,
    edge: AgentFlowEdge,
) -> Result<(), String> {
    flow::add_agent_flow_edge(&env, &flow_name, &edge).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn remove_agent_flow_edge_cmd(
    env: State<AgentEnv>,
    flow_name: String,
    edge_id: String,
) -> Result<(), String> {
    flow::remove_agent_flow_edge(&env, &flow_name, &edge_id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn copy_sub_flow_cmd(
    nodes: Vec<AgentFlowNode>,
    edges: Vec<AgentFlowEdge>,
) -> (Vec<AgentFlowNode>, Vec<AgentFlowEdge>) {
    let node_refs: Vec<&AgentFlowNode> = nodes.iter().collect();
    let edge_refs: Vec<&AgentFlowEdge> = edges.iter().collect();
    flow::copy_sub_flow(node_refs, edge_refs)
}
