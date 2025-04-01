use tauri::AppHandle;
use tauri_plugin_window_state::{AppHandleExt, StateFlags};

mod mnemnk;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(if cfg!(debug_assertions) {
                    log::LevelFilter::Debug
                } else {
                    log::LevelFilter::Info
                })
                .build(),
        )
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            log::info!("show main window");
            mnemnk::window::show_main(app).unwrap_or_else(|e| {
                log::error!("Failed to show main window: {}", e);
            });
        }))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                mnemnk::settings::init(&app_handle).unwrap_or_else(|e| {
                    log::error!("Failed to initialize settings: {}", e);
                    app_handle.exit(1);
                });
                mnemnk::store::init(&app_handle).await.unwrap_or_else(|e| {
                    log::error!("Failed to initialize store: {}", e);
                    app_handle.exit(1);
                });
                mnemnk::tray::init(&app_handle).unwrap_or_else(|e| {
                    log::error!("Failed to initialize tray: {}", e);
                });
                mnemnk::agent::init(&app_handle).await.unwrap_or_else(|e| {
                    log::error!("Failed to initialize agent: {}", e);
                });
                mnemnk::autostart::init(&app_handle).unwrap_or_else(|e| {
                    log::error!("Failed to initialize autostart: {}", e);
                });
                mnemnk::shortcut::init(&app_handle).unwrap_or_else(|e| {
                    log::error!("Failed to initialize shortcut: {}", e);
                });

                ctrlc::set_handler(move || {
                    app_handle.exit(0);
                })
                .unwrap_or_else(|e| {
                    log::error!("Failed to set ctrl-c handler: {}", e);
                });
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            exit_app_cmd,
            mnemnk::agent::env::get_agent_defs_cmd,
            mnemnk::agent::flow::get_agent_flows_cmd,
            mnemnk::agent::flow::new_agent_flow_cmd,
            mnemnk::agent::flow::add_agent_flow_edge_cmd,
            mnemnk::agent::flow::remove_agent_flow_edge_cmd,
            mnemnk::agent::flow::add_agent_flow_node_cmd,
            mnemnk::agent::flow::remove_agent_flow_node_cmd,
            mnemnk::agent::flow::import_agent_flow_cmd,
            mnemnk::agent::flow::save_agent_flow_cmd,
            mnemnk::settings::get_core_settings_cmd,
            mnemnk::settings::set_core_settings_cmd,
            mnemnk::settings::get_agent_configs_cmd,
            mnemnk::settings::set_agent_config_cmd,
            mnemnk::store::daily_stats_cmd,
            mnemnk::store::find_events_by_ymd_cmd,
            mnemnk::store::reindex_ymd_cmd,
            mnemnk::store::search_events_cmd,
            mnemnk::store::reindex_text_cmd,
        ])
        .register_uri_scheme_protocol("mimg", |ctx, request| {
            mnemnk::store::handle_mimg_protocol(ctx.app_handle(), request)
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                #[cfg(not(target_os = "macos"))]
                {
                    window.hide().unwrap();
                }
                #[cfg(target_os = "macos")]
                {
                    use tauri::Manager;
                    tauri::AppHandle::hide(window.app_handle()).unwrap();
                }
                api.prevent_close();
            }
            _ => {}
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let tauri::RunEvent::Exit = event {
                mnemnk::window::hide_main(app).unwrap_or_else(|e| {
                    log::error!("Failed to hide main window: {}", e);
                });
                app.save_window_state(StateFlags::all())
                    .unwrap_or_else(|e| {
                        log::error!("Failed to save window state: {}", e);
                    });
                mnemnk::agent::quit(app);
                mnemnk::store::quit(app);
                mnemnk::settings::quit(app);
            }
        });
}

#[tauri::command]
fn exit_app_cmd(app: AppHandle) -> Result<(), String> {
    // The application will not exit immediately;
    // the Exit event processing above will be executed.
    app.exit(0);
    Ok(())
}
