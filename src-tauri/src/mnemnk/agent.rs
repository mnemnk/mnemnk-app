use std::collections::{BTreeMap, HashMap};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};
use std::time::Duration;
use std::vec;

use anyhow::{Context as _, Result};
use regex::Regex;
use serde::Serialize;
use serde_json::Value;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_shell::process::{CommandChild, CommandEvent};
use tauri_plugin_shell::ShellExt;
use tokio::sync::mpsc;

use super::settings::{self, AgentFlow, AgentFlowNode, AgentSettings, MnemnkSettings};
use super::store;

const EMIT_PUBLISH: &str = "mnemnk:write_board";

pub struct AgentCommands {
    agents: HashMap<String, CommandChild>,
    catalog: BTreeMap<String, PathBuf>,
    tx: mpsc::Sender<AgentMessage>,
}

#[derive(Clone, Debug)]
enum AgentMessage {
    Read {
        agent: String,
        kind: String,
    },
    Store {
        agent: String,
        kind: String,
        value: Value,
    },
    Write {
        agent: String,
        kind: String,
        value: Value,
    },
}

struct AgentBoard {
    board: HashMap<String, Value>,
    subscribers: HashMap<String, Vec<String>>,
}

pub async fn init(app: &AppHandle) {
    let (tx, mut rx) = mpsc::channel(128);

    let agent_commands = AgentCommands {
        agents: HashMap::new(),
        catalog: search_agents(app).await,
        tx,
    };
    app.manage(Mutex::new(agent_commands));

    let agent_board = AgentBoard {
        board: HashMap::new(),
        subscribers: HashMap::new(),
    };
    app.manage(Mutex::new(agent_board));

    let agent_flows;
    let settings = app.state::<Mutex<MnemnkSettings>>();
    {
        let settings = settings.lock().unwrap();
        agent_flows = settings.agent_flows.clone();
    }

    for agent_flow in agent_flows {
        for node in agent_flow.nodes {
            if node.enabled {
                start_agent(app, &node.id).unwrap_or_else(|e| {
                    log::error!("Failed to start agent: {}", e);
                });
            }
        }
    }

    // TODO: each message should have an origin field to prevent infinite loop
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        while let Some(message) = rx.recv().await {
            use AgentMessage::*;

            match message {
                Read { agent, kind } => {
                    let value = read_board(&app_handle, &kind);
                    let agent_commands = app_handle.state::<Mutex<AgentCommands>>();
                    let mut agent_commands = agent_commands.lock().unwrap();
                    let child = agent_commands.agents.get_mut(&agent).unwrap();
                    if let Some(value) = value {
                        if let Err(e) = child
                            .write(format!(".READ_RET {} {}\n", kind, value.to_string()).as_bytes())
                        {
                            log::error!("Failed to write: {} {}", agent, e);
                        }
                    } else {
                        if let Err(e) = child.write(format!(".READ_RET {}\n", kind).as_bytes()) {
                            log::error!("Failed to write: {} {}", agent, e);
                        }
                    }
                }
                Store { agent, kind, value } => {
                    if let Err(e) = write_board(&app_handle, &agent, &kind, &value) {
                        log::error!("Failed to write board: {}", e);
                    }
                    publish(&app_handle, &agent, &kind, &value);
                    if let Err(e) = store::store(&app_handle, agent, kind, value).await {
                        log::error!("Failed to store: {}", e);
                    }
                }
                Write { agent, kind, value } => {
                    if let Err(e) = write_board(&app_handle, &agent, &kind, &value) {
                        log::error!("Failed to write board: {}", e);
                    }
                    publish(&app_handle, &agent, &kind, &value);
                }
            }
        }
    });
}

fn start_agent(app: &AppHandle, agent_id: &str) -> Result<()> {
    let agent_name: String;
    let config: Value;
    let settings = app.state::<Mutex<MnemnkSettings>>();
    {
        let settings = settings.lock().unwrap();
        if let Some(agent_node) = find_agent_node(&settings, agent_id) {
            agent_name = agent_node.name.clone();
            config = agent_node.config.clone().unwrap_or(Value::Null);
        } else {
            log::error!("Agent {} not found", agent_id);
            return Err(anyhow::anyhow!("Agent not found"));
        }
    }

    let agent_commands = app.state::<Mutex<AgentCommands>>();
    let agent_path;
    let main_tx;
    {
        let agent_commands = agent_commands.lock().unwrap();
        if agent_commands.agents.contains_key(agent_id) {
            log::error!("Agent {} is already running", agent_id);
            return Err(anyhow::anyhow!("Agent is already running"));
        }
        agent_path = agent_commands
            .catalog
            .get(&agent_name)
            .context("Agent not found")?
            .clone();
        main_tx = agent_commands.tx.clone();
    }

    log::info!("Starting agent: {} {}", agent_name, agent_id);

    let sidecar_command = if config == Value::Null {
        app.shell().command(agent_path.as_os_str())
    } else {
        app.shell()
            .command(agent_path.as_os_str())
            .args(vec!["-c", serde_json::to_string(&config).unwrap().as_str()])
    };

    let (mut rx, child) = sidecar_command.spawn().context("Failed to spawn sidecar")?;

    let agent_commands = app.state::<Mutex<AgentCommands>>();
    {
        let mut agent_commands = agent_commands.lock().unwrap();
        agent_commands.agents.insert(agent_id.to_string(), child);
    }

    let app_handle = app.clone();
    let agent_id = agent_id.to_string();
    tauri::async_runtime::spawn(async move {
        // read events such as stdout
        while let Some(event) = rx.recv().await {
            match event {
                CommandEvent::Stdout(line_bytes) => {
                    if line_bytes.is_empty() || line_bytes[0] != b'.' {
                        log::debug!(
                            "non-command stdout from {} {}: {:.200}",
                            &agent_name,
                            &agent_id,
                            String::from_utf8_lossy(&line_bytes)
                        );
                        continue;
                    }

                    let line = String::from_utf8_lossy(&line_bytes);
                    // log::debug!("stdout from {}: {:.200}", &agent, &line);

                    let (cmd, args) = parse_stdout(&line);
                    match cmd {
                        ".CONFIG" => {
                            let value = serde_json::from_str::<Value>(args);
                            if let Err(e) = value {
                                log::error!("Failed to parse config: {}", e);
                                continue;
                            }
                            recieve_config(&app_handle, &agent_name, value.unwrap()).unwrap_or_else(
                                |e| {
                                    log::error!("Failed to receive config: {}", e);
                                },
                            )
                        }
                        ".CONFIG_SCHEMA" => {
                            let value = serde_json::from_str::<Value>(args);
                            if let Err(e) = value {
                                log::error!("Failed to parse config schema: {}", e);
                                continue;
                            }
                            recieve_config_schema(&app_handle, &agent_name, value.unwrap())
                                .unwrap_or_else(|e| {
                                    log::error!("Failed to receive config schema: {}", e);
                                })
                        }
                        ".READ" => {
                            let kind = args.to_string();
                            if kind.is_empty() {
                                log::error!("Invalid READ command: {:.40}", &line);
                                continue;
                            }
                            main_tx
                                .send(AgentMessage::Read {
                                    agent: agent_id.clone(),
                                    kind,
                                })
                                .await
                                .unwrap_or_else(|e| {
                                    log::error!("Failed to send message: {}", e);
                                });
                        }
                        ".STORE" => {
                            let kind_value = args.split_once(" ");
                            if kind_value.is_none() {
                                log::error!("Invalid STORE command: {:.40}", &line);
                                continue;
                            }
                            let (kind, value) = kind_value.unwrap();
                            let value = serde_json::from_str::<Value>(value);
                            if value.is_err() {
                                log::error!("Failed to parse value: {:.40}", &line);
                                continue;
                            }
                            main_tx
                                .send(AgentMessage::Store {
                                    agent: agent_id.clone(),
                                    kind: kind.to_string(),
                                    value: value.unwrap(),
                                })
                                .await
                                .unwrap_or_else(|e| {
                                    log::error!("Failed to send message: {}", e);
                                });
                        }
                        ".SUBSCRIBE" => {
                            let kind = args.to_string();
                            subscribe(&app_handle, &agent_id, &kind);
                        }
                        ".WRITE" => {
                            let kind_value = args.split_once(" ");
                            if kind_value.is_none() {
                                log::error!("Invalid WRITE command: {:.40}", &line);
                                continue;
                            }
                            let (kind, value) = kind_value.unwrap();
                            let value = serde_json::from_str::<Value>(value);
                            if value.is_err() {
                                log::error!("Failed to parse value: {:.40}", &line);
                                continue;
                            }
                            main_tx
                                .send(AgentMessage::Write {
                                    agent: agent_id.clone(),
                                    kind: kind.to_string(),
                                    value: value.unwrap(),
                                })
                                .await
                                .unwrap_or_else(|e| {
                                    log::error!("Failed to send message: {}", e);
                                });
                        }
                        _ => {
                            log::error!("Unknown command: {} {}", agent_id, cmd);
                        }
                    }
                }

                CommandEvent::Stderr(line_bytes) => {
                    let line = String::from_utf8_lossy(&line_bytes);
                    log::debug!("stderr from {} {}: {:.200}", agent_name, agent_id, line);
                }

                CommandEvent::Terminated(status) => {
                    log::info!(
                        "Agent exited: {} {} with status: {:?}",
                        agent_name,
                        agent_id,
                        status
                    );
                    unsubscribe_agent(&app_handle, &agent_id);
                    let agent_commands = app_handle.state::<Mutex<AgentCommands>>();
                    {
                        let mut agent_commands = agent_commands.lock().unwrap();
                        agent_commands.agents.remove(&agent_id);
                    }
                    break;
                }

                CommandEvent::Error(e) => {
                    log::error!("CommandEvent Error {} {}: {}", agent_name, agent_id, e);
                }

                _ => {
                    log::error!(
                        "Unknown CommandEvent: {} {} {:?}",
                        agent_name,
                        agent_id,
                        event
                    );
                }
            }
        }
    });
    Ok(())
}

pub fn stop_agent(app: &AppHandle, agent_id: &str) -> Result<()> {
    unsubscribe_agent(app, agent_id);

    let agent_commands = app.state::<Mutex<AgentCommands>>();
    let mut agent_commands = agent_commands.lock().unwrap();
    if let Some(child) = agent_commands.agents.get_mut(agent_id) {
        child.write(".QUIT\n".as_bytes()).unwrap_or_else(|e| {
            log::error!("Failed to write to {}: {}", agent_id, e);
        });
    }
    Ok(())
}

pub fn quit(app: &AppHandle) {
    let agent_commands = app.state::<Mutex<AgentCommands>>();
    {
        // send QUIT command to all agents
        let mut agent_commands = agent_commands.lock().unwrap();
        let agent_ids = agent_commands
            .agents
            .keys()
            .cloned()
            .collect::<vec::Vec<String>>();
        for agent_id in agent_ids {
            log::info!("Stopping agent: {}", agent_id);
            // we cannot use stop_agent here because it will also try to lock aget_commands.
            if let Some(child) = agent_commands.agents.get_mut(&agent_id) {
                child.write(".QUIT\n".as_bytes()).unwrap_or_else(|e| {
                    log::error!("Failed to write to {}: {}", agent_id, e);
                });
            }
        }
    }

    // wait for all agents to exit
    for _ in 0..20 {
        {
            let agent_commands = agent_commands.lock().unwrap();
            if agent_commands.agents.is_empty() {
                return;
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }

    {
        // kill remaining agents
        let mut agent_commands = agent_commands.lock().unwrap();
        let programs = agent_commands
            .agents
            .keys()
            .cloned()
            .collect::<vec::Vec<String>>();
        for program in programs {
            log::warn!("Killing agent: {}", program);
            if let Some(command) = agent_commands.agents.remove(&program) {
                command.kill().unwrap_or_else(|e| {
                    log::error!("Failed to kill agent: {} {}", program, e);
                });
            }
        }
    }
}

fn app_path() -> PathBuf {
    let mut path = std::env::current_exe().unwrap();
    path.pop();
    path
}

async fn user_path(app: &AppHandle) -> Vec<PathBuf> {
    let mut path_dirs = if let Ok(path) = env_path(app).await {
        env::split_paths(&path).collect()
    } else {
        vec![]
    };
    path_dirs.insert(0, app_path());
    path_dirs
}

#[cfg(target_os = "macos")]
async fn env_path(app: &AppHandle) -> Result<String> {
    let output = app
        .shell()
        .command("zsh")
        .args(vec!["-i", "-c", "echo $PATH"])
        .output()
        .await
        .unwrap();
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(anyhow::anyhow!("Failed to get PATH"))
    }
}

#[cfg(not(target_os = "macos"))]
async fn env_path(_app: &AppHandle) -> Result<String> {
    env::var("PATH").context("Failed to get PATH")
}

// is_executable and search_agents are based on cargo/main.rs

#[cfg(unix)]
fn is_executable<P: AsRef<Path>>(path: P) -> bool {
    use std::os::unix::prelude::*;
    fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

#[cfg(windows)]
fn is_executable<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().is_file()
}

async fn search_agents(app: &AppHandle) -> BTreeMap<String, PathBuf> {
    let prefix = "mnemnk-";
    let suffix = env::consts::EXE_SUFFIX;
    let mut agents = BTreeMap::new();
    for path in user_path(app).await {
        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            _ => continue,
        };
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                let Some(filename) = path.file_name().and_then(|s| s.to_str()) else {
                    continue;
                };
                let Some(name) = filename
                    .strip_prefix(prefix)
                    .and_then(|s| s.strip_suffix(suffix))
                else {
                    continue;
                };
                if validate_app_name(name) && is_executable(&path) {
                    agents.insert(name.to_string(), path);
                }
            }
        }
    }
    dbg!(&agents);
    agents
}

fn validate_app_name(name: &str) -> bool {
    static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-zA-Z][a-zA-Z0-9_-]+$").unwrap());
    const DENY_LIST: [&str; 3] = ["app", "prototype", "constructor"];
    RE.is_match(&name) && DENY_LIST.iter().all(|&x| x != name)
}

fn parse_stdout(line: &str) -> (&str, &str) {
    let (cmd, args) = line.split_once(" ").unwrap_or((line, ""));
    (cmd.trim(), args.trim())
}

fn recieve_config(app: &AppHandle, agent: &str, config: Value) -> Result<()> {
    let settings = app.state::<Mutex<MnemnkSettings>>();
    {
        if !validate_config(&config) {
            return Err(anyhow::anyhow!("Invalid config"));
        }

        let mut settings = settings.lock().unwrap();
        if let Some(agent_settings) = settings.agents.get_mut(agent) {
            agent_settings.default_config = Some(config);
        } else {
            let mut agent_settings = AgentSettings::default();
            agent_settings.default_config = Some(config);
            settings.agents.insert(agent.to_string(), agent_settings);
        }
    }
    settings::save(app)?;
    Ok(())
}

fn validate_config(value: &Value) -> bool {
    static RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[a-zA-Z][a-zA-Z0-9_-]+$").unwrap());
    const DENY_LIST: [&str; 2] = ["prototype", "constructor"];

    match value {
        Value::Object(map) => {
            for (key, value) in map {
                if !RE.is_match(&key) || DENY_LIST.iter().any(|&x| x == key) {
                    return false;
                }
                if !validate_config(value) {
                    return false;
                }
            }
        }
        Value::Array(array) => {
            for value in array {
                if !validate_config(value) {
                    return false;
                }
            }
        }
        _ => {}
    }
    true
}

fn recieve_config_schema(app: &AppHandle, agent: &str, schema: Value) -> Result<()> {
    let settings = app.state::<Mutex<MnemnkSettings>>();
    {
        let mut settings = settings.lock().unwrap();
        if let Some(agent_settings) = settings.agents.get_mut(agent) {
            agent_settings.schema = Some(schema);
        } else {
            let mut agent_settings = AgentSettings::default();
            agent_settings.schema = Some(schema);
            settings.agents.insert(agent.to_string(), agent_settings);
        }
    }
    settings::save(app)?;
    Ok(())
}

#[derive(Clone, Debug, Serialize)]
struct WriteBoardMessage {
    agent: String,
    kind: String,
    value: Value,
}

fn write_board(app: &AppHandle, agent_id: &str, kind: &str, value: &Value) -> Result<()> {
    let agent_board = app.state::<Mutex<AgentBoard>>();
    {
        let mut agent_board = agent_board.lock().unwrap();
        agent_board.board.insert(kind.to_string(), value.clone());
    }
    // remove image from the value. it's too big to send to frontend
    let mut value = value.clone();
    if value.get("image").is_some() {
        value.as_object_mut().unwrap().remove("image");
    }
    // emit the message to frontend
    let message = WriteBoardMessage {
        agent: agent_id.to_string(),
        kind: kind.to_string(),
        value,
    };
    app.emit(EMIT_PUBLISH, Some(message))?;
    Ok(())
}

fn read_board(app: &AppHandle, agent_id: &str) -> Option<Value> {
    let agent_board = app.state::<Mutex<AgentBoard>>();
    let agent_board = agent_board.lock().unwrap();
    agent_board.board.get(agent_id).cloned()
}

fn publish(app: &AppHandle, agent_id: &str, kind: &str, value: &Value) {
    let agent_board = app.state::<Mutex<AgentBoard>>();
    let subscribers;
    {
        let agent_board = agent_board.lock().unwrap();
        subscribers = agent_board.subscribers.get(kind).cloned();
    }
    if let Some(subscribers) = subscribers {
        let agent_commands = app.state::<Mutex<AgentCommands>>();
        for subscriber in subscribers {
            if subscriber == agent_id {
                continue;
            }
            let mut agent_commands = agent_commands.lock().unwrap();
            if let Some(child) = agent_commands.agents.get_mut(&subscriber) {
                child
                    .write(
                        format!(".PUBLISH {} {} {}\n", agent_id, kind, value.to_string())
                            .as_bytes(),
                    )
                    .unwrap_or_else(|e| {
                        log::error!("Failed to write to {}: {}", subscriber, e);
                    });
            }
        }
    }
}

fn subscribe(app: &AppHandle, agent_id: &str, kind: &str) {
    let agent_board = app.state::<Mutex<AgentBoard>>();
    {
        let mut agent_board = agent_board.lock().unwrap();
        if let Some(subscribers) = agent_board.subscribers.get_mut(kind) {
            subscribers.push(agent_id.to_string());
        } else {
            agent_board
                .subscribers
                .insert(kind.to_string(), vec![agent_id.to_string()]);
        }
    }
}

fn unsubscribe_agent(app: &AppHandle, agent_id: &str) {
    let agent_board = app.state::<Mutex<AgentBoard>>();
    {
        let mut agent_board = agent_board.lock().unwrap();
        for subscribers in agent_board.subscribers.values_mut() {
            subscribers.retain(|s| s != agent_id);
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AgentCatalogEntry {
    pub name: String,
    pub path: String,
}

#[tauri::command]
pub fn get_agent_catalog_cmd(
    agent_commands: State<Mutex<AgentCommands>>,
) -> Result<Vec<AgentCatalogEntry>, String> {
    let catalog;
    {
        let agent_commands = agent_commands.lock().unwrap();
        catalog = agent_commands.catalog.clone();
    }
    Ok(catalog
        .into_iter()
        .map(|(k, v)| AgentCatalogEntry {
            name: k,
            path: v.to_string_lossy().to_string(),
        })
        .collect())
}

#[tauri::command(rename_all = "snake_case")]
pub fn start_agent_cmd(app: AppHandle, agent_id: String) -> Result<(), String> {
    start_agent(&app, &agent_id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn stop_agent_cmd(app: AppHandle, agent_id: String) -> Result<(), String> {
    stop_agent(&app, &agent_id).map_err(|e| e.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_agent_enabled_cmd(
    app: AppHandle,
    settings: State<Mutex<MnemnkSettings>>,
    agent_id: &str,
    enabled: bool,
) -> Result<(), String> {
    {
        let mut settings = settings.lock().unwrap();
        if let Some(agent_node) = find_agent_node_mut(&mut settings, agent_id) {
            agent_node.enabled = enabled;
        } else {
            return Err("Agent not found".to_string());
        }
    }
    settings::save(&app).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn save_agent_config_cmd(
    app: AppHandle,
    settings: State<Mutex<MnemnkSettings>>,
    agent_id: &str,
    config: Value,
) -> Result<(), String> {
    {
        let mut settings = settings.lock().unwrap();
        if let Some(agent_node) = find_agent_node_mut(&mut settings, agent_id) {
            agent_node.config = Some(config.clone());
        } else {
            return Err("Agent not found".to_string());
        }
    }
    settings::save(&app).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn get_agent_settings_cmd(settings: State<Mutex<MnemnkSettings>>) -> Result<Value, String> {
    let settings = settings.lock().unwrap();
    let config = settings.agents.clone();
    let value = serde_json::to_value(&config).map_err(|e| e.to_string())?;
    Ok(value)
}

#[tauri::command]
pub fn get_agent_flows_cmd(settings: State<Mutex<MnemnkSettings>>) -> Result<Value, String> {
    let settings = settings.lock().unwrap();
    let config = settings.agent_flows.clone();
    let value = serde_json::to_value(&config).map_err(|e| e.to_string())?;
    Ok(value)
}

#[tauri::command(rename_all = "snake_case")]
pub fn save_agent_flow_cmd(
    app: AppHandle,
    settings: State<Mutex<MnemnkSettings>>,
    agent_flow: AgentFlow,
    idx: usize,
) -> Result<(), String> {
    {
        let mut settings = settings.lock().unwrap();
        if idx < settings.agent_flows.len() {
            settings.agent_flows[idx] = agent_flow;
        } else {
            settings.agent_flows.push(agent_flow);
        }
    }
    settings::save(&app).map_err(|e| e.to_string())?;
    Ok(())
}

fn find_agent_node<'a>(settings: &'a MnemnkSettings, agent_id: &str) -> Option<&'a AgentFlowNode> {
    for agent_flow in &settings.agent_flows {
        if let Some(agent_node) = agent_flow.nodes.iter().find(|x| x.id == agent_id) {
            return Some(agent_node);
        }
    }
    None
}

fn find_agent_node_mut<'a>(
    settings: &'a mut MnemnkSettings,
    agent_id: &str,
) -> Option<&'a mut AgentFlowNode> {
    for agent_flow in &mut settings.agent_flows {
        if let Some(agent_node) = agent_flow.nodes.iter_mut().find(|x| x.id == agent_id) {
            return Some(agent_node);
        }
    }
    return None;
}
