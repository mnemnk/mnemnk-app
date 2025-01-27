use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, path::PathBuf, sync::Mutex};
use tauri::{AppHandle, Manager};

#[derive(Debug, Serialize, Deserialize)]
pub struct CoreSettings {
    pub autostart: Option<bool>,
    pub data_dir: Option<String>,
    pub shortcut_key: Option<String>,
    pub thumbnail_width: Option<u32>,
    pub thumbnail_height: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentSettings {
    pub enabled: Option<bool>,
    pub path: Option<String>,
    pub config: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MnemnkSettings {
    pub core: CoreSettings,
    pub agents: HashMap<String, AgentSettings>,
}

impl Default for CoreSettings {
    fn default() -> Self {
        let autostart = Some(false);
        let data_dir = None;
        let shortcut_key = Some("Alt+Shift+KeyM".to_string());

        CoreSettings {
            autostart,
            data_dir,
            shortcut_key,
            thumbnail_width: None,
            thumbnail_height: None,
        }
    }
}

impl Default for AgentSettings {
    fn default() -> Self {
        AgentSettings {
            enabled: Some(false),
            path: None,
            config: None,
        }
    }
}

impl Default for MnemnkSettings {
    fn default() -> Self {
        MnemnkSettings {
            core: CoreSettings::default(),
            agents: HashMap::from([
                (
                    "mnemnk-api".to_string(),
                    AgentSettings {
                        enabled: Some(true),
                        ..Default::default()
                    },
                ),
                (
                    "mnemnk-application".to_string(),
                    AgentSettings {
                        enabled: Some(true),
                        ..Default::default()
                    },
                ),
                (
                    "mnemnk-screen".to_string(),
                    AgentSettings {
                        enabled: Some(true),
                        ..Default::default()
                    },
                ),
            ]),
        }
    }
}

pub fn init(app: &AppHandle) {
    let settings: MnemnkSettings =
        confy::load(app_name(app), None).expect("Failed to load settings");
    dbg!(&settings);
    app.manage(Mutex::new(settings));
}

pub fn save(app: &AppHandle) {
    let settings = app.state::<Mutex<MnemnkSettings>>();
    {
        let settings = settings.lock().unwrap();
        confy::store(app_name(app), None, &*settings).expect("Failed to save settings");
    }
}

pub fn data_dir(app: &AppHandle) -> Option<String> {
    let mut data_dir;
    let settings = app.state::<Mutex<MnemnkSettings>>();
    {
        let settings = settings.lock().unwrap();
        data_dir = settings.core.data_dir.clone();
    }
    if let Some(dir) = &data_dir {
        let dir = PathBuf::from(dir);
        if !dir.exists() {
            std::fs::create_dir(dir).expect("Failed to create data directory");
        }
    } else {
        let app_local_data_dir = app
            .path()
            .app_local_data_dir()
            .expect("Failed to get app local data directory");
        if !app_local_data_dir.exists() {
            std::fs::create_dir(&app_local_data_dir).expect("Failed to create data directory");
        }
        data_dir = Some(app_local_data_dir.to_string_lossy().to_string());
    }
    data_dir
}

pub fn quit(app: &AppHandle) {
    save(app);
}

fn app_name(app: &AppHandle) -> &str {
    app.config().identifier.as_str()
}

#[tauri::command]
pub fn get_settings_json(app: AppHandle) -> Result<String, String> {
    let settings = app.state::<Mutex<MnemnkSettings>>();
    let settings = settings.lock().unwrap();
    let json = serde_json::to_string_pretty(&*settings).map_err(|e| e.to_string())?;
    Ok(json)
}

#[tauri::command]
pub fn set_settings_json(app: AppHandle, json_str: String) -> Result<(), String> {
    {
        let settings = app.state::<Mutex<MnemnkSettings>>();
        let mut settings = settings.lock().unwrap();
        *settings = serde_json::from_str(&json_str).map_err(|e| e.to_string())?;
    }
    save(&app);
    Ok(())
}
