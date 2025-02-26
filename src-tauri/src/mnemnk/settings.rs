use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{LazyLock, Mutex},
};
use tauri::{AppHandle, Manager, State};

const CONFIG_FILENAME: &str = "config.yml";

#[derive(Debug, Serialize, Deserialize)]
pub struct MnemnkSettings {
    pub core: CoreSettings,
    pub agents: HashMap<String, AgentSettings>,
}

impl Default for MnemnkSettings {
    fn default() -> Self {
        MnemnkSettings {
            core: CoreSettings::default(),
            agents: HashMap::default(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CoreSettings {
    pub autostart: Option<bool>,
    pub data_dir: Option<String>,
    pub shortcut_key: Option<String>, // global shortcut key. Will be merged into shortcut_keys
    pub shortcut_keys: Option<HashMap<String, String>>,
    pub thumbnail_width: Option<u32>,
    pub thumbnail_height: Option<u32>,
}

impl Default for CoreSettings {
    fn default() -> Self {
        static SHORTCUT_KEYS: LazyLock<HashMap<String, String>> = LazyLock::new(|| {
            let mut map = HashMap::new();
            #[cfg(target_os = "macos")]
            {
                map.insert("global_shortcut".into(), "Command+Shift+M".into());
                map.insert("fullscreen".into(), "".into()); // macOS has its own fullscreen shortcut (Cmd+Ctrl+F)
            }
            #[cfg(not(target_os = "macos"))]
            {
                map.insert("global_shortcut".into(), "Alt+Shift+M".into());
                map.insert("fullscreen".into(), "F11".into());
            }
            map.insert("screenshot_only".into(), " ".into());
            map.insert("search".into(), "Ctrl+K, Command+K".into());
            map
        });

        CoreSettings {
            autostart: Some(false),
            data_dir: None,
            shortcut_key: Some("Alt+Shift+KeyM".into()),
            shortcut_keys: Some(SHORTCUT_KEYS.clone()),
            thumbnail_width: None,
            thumbnail_height: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgentSettings {
    pub enabled: Option<bool>,
    pub config: Option<Value>,
    pub schema: Option<Value>,
}

impl Default for AgentSettings {
    fn default() -> Self {
        AgentSettings {
            enabled: Some(false),
            config: None,
            schema: None,
        }
    }
}

pub fn init(app: &AppHandle) {
    let cf = config_file(app);
    log::info!("Config file: {}", cf);
    let mut settings: MnemnkSettings =
        confy::load_path(cf).unwrap_or_else(|_| MnemnkSettings::default());
    if let Some(ref mut shortcut_keys) = settings.core.shortcut_keys {
        let default_core_settings = CoreSettings::default();
        for (k, v) in default_core_settings.shortcut_keys.unwrap().iter() {
            if !shortcut_keys.contains_key(k) {
                shortcut_keys.insert(k.clone(), v.clone());
            }
        }
    } else {
        settings.core.shortcut_keys = CoreSettings::default().shortcut_keys;
    }
    dbg!(&settings);
    app.manage(Mutex::new(settings));
}

pub fn save(app: &AppHandle) {
    let settings = app.state::<Mutex<MnemnkSettings>>();
    {
        let settings = settings.lock().unwrap();
        confy::store_path(config_file(app), &*settings).unwrap_or_else(|e| {
            log::error!("Failed to save settings: {}", e);
        });
    }
}

pub fn config_file(app: &AppHandle) -> String {
    let app_config_dir = app
        .path()
        .app_config_dir()
        .expect("Failed to get app config directory");
    if !app_config_dir.exists() {
        std::fs::create_dir(&app_config_dir).expect("Failed to create config directory");
    }
    app_config_dir
        .join(CONFIG_FILENAME)
        .to_string_lossy()
        .to_string()
}

pub fn data_dir(app: &AppHandle) -> Option<String> {
    let mut data_dir;
    let settings = app.state::<Mutex<MnemnkSettings>>();
    {
        let settings = settings.lock().unwrap();
        data_dir = settings.core.data_dir.clone();
    }
    if data_dir.is_some() && !data_dir.as_ref().unwrap().is_empty() {
        let dir = PathBuf::from(data_dir.as_ref().unwrap());
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

pub fn quit(_app: &AppHandle) {
    // save(app);
}

#[tauri::command]
pub fn get_core_settings_cmd(settings: State<Mutex<MnemnkSettings>>) -> Result<Value, String> {
    let settings = settings.lock().unwrap();
    let json = serde_json::to_value(&(*settings).core).map_err(|e| e.to_string())?;
    Ok(json)
}

#[tauri::command(rename_all = "snake_case")]
pub fn set_core_settings_cmd(
    app: AppHandle,
    settings: State<Mutex<MnemnkSettings>>,
    new_settings: Value,
) -> Result<(), String> {
    let new_settings: CoreSettings =
        serde_json::from_value(new_settings).map_err(|e| e.to_string())?;
    {
        let mut settings = settings.lock().unwrap();
        (*settings).core = new_settings;
    }
    save(&app);
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
pub fn get_settings_filepath_cmd(app: AppHandle) -> Result<String, String> {
    let cf = config_file(&app);
    Ok(cf)
}
