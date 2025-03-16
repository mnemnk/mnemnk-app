use std::collections::{BTreeMap, HashMap, HashSet};
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

#[derive(Clone, Debug)]
enum AgentMessage {
    Board {
        kind: String,
        value: Value,
    },
    Write {
        agent: String,
        kind: String,
        value: Value,
    },
}

pub struct AgentCommands {
    // node id -> child process
    agents: HashMap<String, CommandChild>,

    // agent name -> path
    catalog: BTreeMap<String, PathBuf>,

    // message sender
    tx: mpsc::Sender<AgentMessage>,

    // enabled node ids
    enabled_nodes: HashSet<String>,

    // node id -> node ids
    edges: HashMap<String, Vec<String>>,
}

struct AgentBoards {
    // node id -> board name
    board_names: HashMap<String, String>,

    // board name -> value
    board_values: HashMap<String, Value>,

    // board name -> subscribers (node ids)
    subscribers: HashMap<String, Vec<String>>,
}

pub async fn init(app: &AppHandle) {
    let (tx, rx) = mpsc::channel(128);

    let agent_commands = AgentCommands {
        agents: HashMap::new(),
        catalog: search_agents(app).await,
        tx,
        enabled_nodes: HashSet::new(),
        edges: HashMap::new(),
    };
    app.manage(Mutex::new(agent_commands));

    let agent_boards = AgentBoards {
        board_names: HashMap::new(),
        board_values: HashMap::new(),
        subscribers: HashMap::new(),
    };
    app.manage(Mutex::new(agent_boards));

    sync_agent_flows(app);
    spawn_main_loop(app, rx);
}

fn sync_agent_flows(app: &AppHandle) {
    let agent_flows;
    let settings = app.state::<Mutex<MnemnkSettings>>();
    {
        let settings = settings.lock().unwrap();
        agent_flows = settings.agent_flows.clone();
    }

    let mut enabled_nodes = HashSet::new();
    let mut edges = HashMap::<String, Vec<String>>::new();
    let mut board_names = HashMap::<String, String>::new();
    let mut subscribers = HashMap::<String, Vec<String>>::new();
    for agent_flow in &agent_flows {
        for node in &agent_flow.nodes {
            if !node.enabled {
                continue;
            } else if node.name == "$board" {
                if let Some(board_name) = node
                    .config
                    .as_ref()
                    .and_then(|x| x.get("board_name").cloned())
                {
                    if let Some(board_name_str) = board_name.as_str() {
                        board_names.insert(node.id.clone(), board_name_str.to_string());
                    } else {
                        log::error!("Invalid board_name for node: {}", node.id);
                    }
                }
            } else if node.name == "$database" {
                // nothing
            } else if node.name.starts_with("$") {
                log::error!("Unknown node: {}", node.name);
                continue;
            } else {
                if let Err(e) = start_agent(app, &node.id) {
                    log::error!("Failed to start agent: {}", e);
                    continue;
                };
            }
            enabled_nodes.insert(node.id.clone());
        }
    }
    for agent_flow in &agent_flows {
        for edge in &agent_flow.edges {
            if !enabled_nodes.contains(&edge.source) || !enabled_nodes.contains(&edge.target) {
                continue;
            }
            if edge.source.starts_with("$board_") {
                if let Some(board_name) = board_names.get(&edge.source) {
                    if board_name == "" || board_name == "*" {
                        continue;
                    }
                    if let Some(subs) = subscribers.get_mut(board_name) {
                        subs.push(edge.target.clone());
                    } else {
                        subscribers.insert(board_name.clone(), vec![edge.target.clone()]);
                    }
                }
                continue;
            }

            if let Some(targets) = edges.get_mut(&edge.source) {
                targets.push(edge.target.clone());
            } else {
                edges.insert(edge.source.clone(), vec![edge.target.clone()]);
            }
        }
    }
    let agent_commands = app.state::<Mutex<AgentCommands>>();
    {
        let mut agent_commands = agent_commands.lock().unwrap();
        agent_commands.enabled_nodes = enabled_nodes;
        agent_commands.edges = edges;
    }
    let agent_boards = app.state::<Mutex<AgentBoards>>();
    {
        let mut agent_boards = agent_boards.lock().unwrap();
        agent_boards.board_names = board_names;
        agent_boards.subscribers = subscribers;
    }
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
                        ".SUBSCRIBE" => {
                            // let kind = args.to_string();
                            // subscribe(&app_handle, &agent_id, &kind);
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
                    // unsubscribe_agent(&app_handle, &agent_id);
                    let agent_commands = app_handle.state::<Mutex<AgentCommands>>();
                    {
                        let mut agent_commands = agent_commands.lock().unwrap();
                        agent_commands.agents.remove(&agent_id);
                        agent_commands.enabled_nodes.remove(&agent_id);
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

fn spawn_main_loop(app: &AppHandle, rx: mpsc::Receiver<AgentMessage>) {
    // TODO: each message should have an origin field to prevent infinite loop
    let app_handle = app.clone();
    let mut rx = rx;
    tauri::async_runtime::spawn(async move {
        while let Some(message) = rx.recv().await {
            use AgentMessage::*;

            match message {
                Board { kind, value } => {
                    board_message(&app_handle, kind, value).await;
                }
                Write { agent, kind, value } => {
                    write_message(&app_handle, agent, kind, value).await;
                }
            }
        }
    });
}

pub fn stop_agent(app: &AppHandle, agent_id: &str) -> Result<()> {
    let agent_commands = app.state::<Mutex<AgentCommands>>();
    let mut agent_commands = agent_commands.lock().unwrap();
    agent_commands.enabled_nodes.remove(agent_id);
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

async fn board_message(app: &AppHandle, kind: String, value: Value) {
    let subscribers;
    let agent_boards = app.state::<Mutex<AgentBoards>>();
    {
        let agent_board = agent_boards.lock().unwrap();
        subscribers = agent_board.subscribers.get(&kind).cloned(); // TODO: use board_name instead of kind
    }
    if let Some(subscribers) = subscribers {
        let enabled_nodes;
        let agent_commands = app.state::<Mutex<AgentCommands>>();
        {
            let agent_commands = agent_commands.lock().unwrap();
            enabled_nodes = agent_commands.enabled_nodes.clone();
        }
        for subscriber in subscribers {
            if !enabled_nodes.contains(&subscriber) {
                continue;
            }
            if subscriber.starts_with("$") {
                if subscriber.starts_with("$board_") {
                    if let Err(e) = write_message_to_board(
                        &app,
                        subscriber.clone(),
                        kind.clone(),
                        value.clone(),
                    )
                    .await
                    {
                        log::error!("Failed to write board: {}", e);
                    };
                } else if subscriber.starts_with("$database_") {
                    if let Err(e) =
                        store::store(&app, kind.clone(), kind.clone(), value.clone()).await
                    {
                        log::error!("Failed to store: {}", e);
                    }
                } else {
                    log::error!("Unknown subscriber: {}", subscriber);
                }
            } else {
                write_message_to_agent(&agent_commands, &kind, &subscriber, &kind, &value);
            }
        }
    }
}

async fn write_message(app: &AppHandle, agent_id: String, kind: String, value: Value) {
    let targets;
    let enabled_nodes;
    let agent_commands = app.state::<Mutex<AgentCommands>>();
    {
        let agent_commands = agent_commands.lock().unwrap();
        targets = agent_commands.edges.get(&agent_id).cloned();
        enabled_nodes = agent_commands.enabled_nodes.clone();
    }
    if let Some(targets) = targets {
        for target in targets {
            if !enabled_nodes.contains(&target) {
                continue;
            }
            if target.starts_with("$") {
                if target.starts_with("$board_") {
                    if let Err(e) =
                        write_message_to_board(&app, target.clone(), kind.clone(), value.clone())
                            .await
                    {
                        log::error!("Failed to write board: {}", e);
                    };
                } else if target.starts_with("$database_") {
                    if let Err(e) =
                        store::store(&app, agent_id.clone(), kind.clone(), value.clone()).await
                    {
                        log::error!("Failed to store: {}", e);
                    }
                } else {
                    log::error!("Unknown target: {}", target);
                }
            } else {
                write_message_to_agent(&agent_commands, &agent_id, &target, &kind, &value);
            }
        }
    }
}

fn write_message_to_agent(
    agent_commands: &State<Mutex<AgentCommands>>,
    agent_id: &str,
    target: &str,
    kind: &str,
    value: &Value,
) {
    let mut agent_commands = agent_commands.lock().unwrap();
    if let Some(command) = agent_commands.agents.get_mut(target) {
        command
            .write(format!(".PUBLISH {} {} {}\n", agent_id, kind, value.to_string()).as_bytes())
            .unwrap_or_else(|e| {
                log::error!("Failed to write to {}: {}", target, e);
            });
    } else {
        log::error!("Agent not found: {}", target);
    }
}

#[derive(Clone, Debug, Serialize)]
struct WriteBoardMessage {
    agent: String,
    kind: String,
    value: Value,
}

async fn write_message_to_board(
    app: &AppHandle,
    board_id: String,
    kind: String,
    value: Value,
) -> Result<()> {
    // update board value
    let board_name;
    let agent_boards = app.state::<Mutex<AgentBoards>>();
    {
        let mut agent_boards = agent_boards.lock().unwrap();
        if let Some(bn) = agent_boards.board_names.get(&board_id) {
            board_name = if bn == "" || bn == "*" {
                kind.clone()
            } else {
                bn.clone()
            };
        } else {
            board_name = kind.clone();
        }
        agent_boards
            .board_values
            .insert(board_name.clone(), value.clone());
    }

    send_board(&app, board_name.clone(), value.clone()).await;

    // remove image from the value. it's too big to send to frontend
    let mut value = value;
    if value.get("image").is_some() {
        value.as_object_mut().unwrap().remove("image");
    }

    // emit the message to frontend
    let message = WriteBoardMessage {
        agent: board_id,
        kind: board_name,
        value,
    };
    app.emit(EMIT_PUBLISH, Some(message))?;

    Ok(())
}

async fn send_board(app: &AppHandle, kind: String, value: Value) {
    let agent_commands = app.state::<Mutex<AgentCommands>>();
    let main_tx;
    {
        let agent_commands = agent_commands.lock().unwrap();
        main_tx = agent_commands.tx.clone();
    }
    main_tx
        .send(AgentMessage::Board { kind, value })
        .await
        .unwrap_or_else(|e| {
            log::error!("Failed to send message: {}", e);
        });
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
