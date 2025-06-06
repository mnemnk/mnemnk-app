#[cfg(feature = "api")]
mod implementation {
    use anyhow::{Context as _, Result};
    use axum::{extract::State, routing::post, Json, Router};
    use axum_auth::AuthBearer;
    use serde::{Deserialize, Serialize};
    use serde_json::{json, Value};
    use std::sync::{Arc, Mutex};
    use tauri::{AppHandle, Manager};
    use tokio::net::TcpListener;
    use tokio::time::Duration;
    use tower_http::timeout::TimeoutLayer;

    use crate::mnemnk::agent::{Agent, AgentConfig, AgentContext, AgentData, AsAgent, AsAgentData};

    pub struct ApiAgent {
        data: AsAgentData,
        server_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
    }

    impl AsAgent for ApiAgent {
        fn new(
            app: AppHandle,
            id: String,
            def_name: String,
            config: Option<AgentConfig>,
        ) -> Result<Self> {
            Ok(Self {
                data: AsAgentData::new(app, id, def_name, config),
                server_handle: Arc::new(Mutex::new(None)),
            })
        }

        fn data(&self) -> &AsAgentData {
            &self.data
        }

        fn mut_data(&mut self) -> &mut AsAgentData {
            &mut self.data
        }

        fn start(&mut self) -> Result<()> {
            self.start_server()?;
            Ok(())
        }

        fn stop(&mut self) -> Result<()> {
            self.stop_server()?;
            Ok(())
        }
    }

    #[derive(Debug, Deserialize, Serialize)]
    struct OutData {
        ch: String,
        kind: String,
        value: Value,
    }

    impl ApiAgent {
        fn start_server(&mut self) -> Result<()> {
            let app_handle = self.app().clone();
            let agent_id = self.id().to_string();

            let global_config = self.global_config().context("no global config")?;
            let address = global_config.get_string_or("address", super::ADDRESS_DEFAULT);
            let api_key = global_config.get_string("api_key");

            let server_handle = self.server_handle.clone();

            let handle = tokio::spawn(async move {
                log::info!("Starting API server on {}", address);

                let app_state = AppState {
                    app_handle,
                    agent_id,
                    api_key,
                };

                let app = Router::new()
                    .route("/out", post(handle_out).with_state(app_state))
                    .layer((TimeoutLayer::new(Duration::from_secs(10)),));

                if let Ok(listener) = TcpListener::bind(&address).await {
                    axum::serve(listener, app).await.unwrap_or_else(|e| {
                        log::error!("API server error: {}", e);
                    });
                } else {
                    log::error!("Failed to bind to address: {}", address);
                }

                log::info!("API server stopped");
            });

            *server_handle.lock().unwrap() = Some(handle);

            Ok(())
        }

        fn stop_server(&mut self) -> Result<()> {
            if let Some(handle) = self.server_handle.lock().unwrap().take() {
                handle.abort();
                log::info!("API server stopping...");
            }

            Ok(())
        }
    }

    #[derive(Clone)]
    struct AppState {
        app_handle: AppHandle,
        agent_id: String,
        api_key: Option<String>,
    }

    async fn handle_out(
        AuthBearer(token): AuthBearer,
        State(state): State<AppState>,
        Json(out_data): Json<OutData>,
    ) -> Result<Json<Value>, String> {
        // Check API key if configured
        if let Some(key) = &state.api_key {
            if token != *key {
                return Err("Unauthorized".to_string());
            }
        }

        // Validate the request
        if out_data.ch.is_empty() {
            return Err("Channel is empty".to_string());
        }
        if out_data.kind.is_empty() {
            return Err("Kind is empty".to_string());
        }

        let agent_data = AgentData::from_json_data(out_data.kind, out_data.value)
            .map_err(|e| format!("Failed to create AgentData: {}", e))?;

        // Get the environment and try to send the output
        let env = state.app_handle.state::<crate::mnemnk::agent::AgentEnv>();
        if let Err(e) = env.try_send_agent_out(
            state.agent_id,
            AgentContext::new_with_ch(out_data.ch),
            agent_data,
        ) {
            log::error!("Failed to send agent out: {}", e);
            return Err(format!("Failed to process request: {}", e));
        }

        Ok(Json(json!({"status": "ok"})))
    }
}

// Dummy API Agent implementation when the feature is not enabled
#[cfg(not(feature = "api"))]
mod implementation {
    use anyhow::Result;
    use tauri::AppHandle;

    use crate::mnemnk::agent::{AgentConfig, AgentData, AsAgent, AsAgentData};

    pub struct ApiAgent {
        data: AsAgentData,
    }

    impl AsAgent for ApiAgent {
        fn new(
            app: AppHandle,
            id: String,
            def_name: String,
            config: Option<AgentConfig>,
        ) -> Result<Self> {
            Ok(Self {
                data: AsAgentData::new(app, id, def_name, config),
            })
        }

        fn data(&self) -> &AsAgentData {
            &self.data
        }

        fn mut_data(&mut self) -> &mut AsAgentData {
            &mut self.data
        }

        fn process(&mut self, _ch: String, _data: AgentData) -> Result<()> {
            log::warn!("API Agent is disabled because the 'builtin-api' feature is not enabled");
            Ok(())
        }
    }
}

pub use implementation::ApiAgent;

use crate::mnemnk::agent::agent::new_boxed;
use crate::mnemnk::agent::definition::AGENT_KIND_BUILTIN;
use crate::mnemnk::agent::{AgentConfigEntry, AgentDefinition, AgentDefinitions, AgentValue};

static CONFIG_ADDRESS: &str = "address";
static CONFIG_API_KEY: &str = "api_key";

static ADDRESS_DEFAULT: &str = "localhost:3296";

pub fn init_agent_defs(defs: &mut AgentDefinitions) {
    // API Agent
    defs.insert(
        "$api".into(),
        AgentDefinition::new(AGENT_KIND_BUILTIN, "$api", Some(new_boxed::<ApiAgent>))
            .with_title("API")
            .with_category("Core")
            .with_outputs(vec!["*"])
            .with_global_config(vec![
                (
                    CONFIG_ADDRESS.into(),
                    AgentConfigEntry::new(AgentValue::new_string(ADDRESS_DEFAULT), "string")
                        .with_title("Address")
                        .with_description("API server address (host:port)"),
                ),
                (
                    CONFIG_API_KEY.into(),
                    AgentConfigEntry::new(AgentValue::new_string(""), "string")
                        .with_title("API Key")
                        .with_description("API key for authentication"),
                ),
            ]),
    );
}
