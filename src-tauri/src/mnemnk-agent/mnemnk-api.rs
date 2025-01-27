use std::time::Duration;

use anyhow::Result;
use axum::{routing::post, Json, Router};
use clap::Parser;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tokio::io::{stdin, AsyncBufReadExt, BufReader};
use tokio::net::TcpListener;
use tokio::signal;
use tower_http::timeout::TimeoutLayer;

const AGENT_NAME: &str = "mnemnk-api";

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct AgentConfig {
    address: String,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            address: "localhost:3296".to_string(),
        }
    }
}

#[derive(Debug, Parser)]
pub struct Args {
    #[arg(short = 'c', long = "config", help = "JSON config string")]
    config: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();

    let mut config = AgentConfig::default();
    if let Some(c) = &args.config {
        if let Value::Object(c) = serde_json::from_str(c)? {
            if let Some(address) = c.get("address") {
                config.address = address.as_str().unwrap().to_string();
            }
        }
    }
    println!("CONFIG {}", serde_json::to_string(&config)?);

    log::info!("Starting {}.", AGENT_NAME);

    let c = config.clone();
    tokio::spawn(async move {
        start_server(&c).await;
    });

    let mut reader = BufReader::new(stdin());
    let mut line = String::new();
    loop {
        tokio::select! {
            // Read from stdin
            _ = reader.read_line(&mut line) => {
                if let Err(e) = process_line(&config, &line).await {
                    log::error!("Failed to process line: {}", e);
                }
                line.clear();
            }
        }
    }
}

async fn start_server(config: &AgentConfig) {
    let app = Router::new()
        .route("/store", post(store))
        .layer((TimeoutLayer::new(Duration::from_secs(2)),));
    let listener = TcpListener::bind(&config.address)
        .await
        .expect("failed to bind address");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("failed to start server");
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct StoreRequest {
    agent: String,
    kind: String,
    value: Value,
}

async fn store(request: Json<StoreRequest>) -> Result<Json<Value>, String> {
    let json_value = serde_json::to_string(&request.value).map_err(|e| e.to_string())?;
    // TODO: store agent into metadata
    println!("STORE {} {}", request.kind, json_value);
    Ok(Json(json!({"status": "ok"})))
}

async fn process_line(config: &AgentConfig, line: &str) -> Result<()> {
    log::debug!("process_line: {}", line);

    if let Some((cmd, args)) = parse_line(line) {
        match cmd {
            "GET_CONFIG" => {
                get_config(config, args)?;
            }
            "QUIT" => {
                log::info!("QUIT {}.", AGENT_NAME);
                // TODO: send message to server
                std::process::exit(0);
            }
            _ => {
                log::error!("Unknown command: {}", cmd);
            }
        }
    }
    Ok(())
}

fn parse_line(line: &str) -> Option<(&str, &str)> {
    if line.is_empty() {
        return None;
    }

    let line = line.trim();
    if line.is_empty() {
        return None;
    }

    if let Some((cmd, args)) = line.split_once(" ") {
        Some((cmd, args))
    } else {
        Some((line, ""))
    }
}

fn get_config(config: &AgentConfig, _args: &str) -> Result<()> {
    println!("CONFIG {}", serde_json::to_string(config)?);
    Ok(())
}

// async fn execute_task(_config: &AgentConfig) -> Result<()> {
//     let win_info = check_application().await;

//     if win_info.is_none() {
//         return Ok(());
//     }

//     if let Some(win_info) = win_info {
//         // debug!("check_application: {:?}", win_info);
//         let win_info_json = serde_json::to_string(&win_info)?;
//         println!("STORE {} {}", KIND, win_info_json);
//     }

//     Ok(())
// }
