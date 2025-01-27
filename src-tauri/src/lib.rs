use tauri_plugin_window_state::{AppHandleExt, StateFlags};

mod mnemnk;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                // .level(log::LevelFilter::Debug)
                .build(),
        )
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            log::info!("show main window");
            mnemnk::window::show_main(app).unwrap_or_else(|e| {
                log::error!("Failed to show main window: {}", e);
            });
        }))
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                mnemnk::settings::init(&app_handle);
                mnemnk::tray::init(&app_handle).unwrap_or_else(|e| {
                    log::error!("Failed to initialize tray: {}", e);
                });
                mnemnk::store::init(&app_handle)
                    .await
                    .unwrap_or_else(|e| log::error!("Failed to initialize store: {}", e));
                mnemnk::agent::init(&app_handle);
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
            mnemnk::settings::get_settings_json,
            mnemnk::settings::set_settings_json,
            mnemnk::store::index_year,
            mnemnk::store::find_events_by_ymd,
            mnemnk::store::search,
        ])
        .register_uri_scheme_protocol("mimg", |ctx, request| {
            mnemnk::store::handle_mimg_protocol(ctx.app_handle(), request)
        })
        .on_window_event(|window, event| match event {
            tauri::WindowEvent::CloseRequested { api, .. } => {
                window.hide().unwrap();
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
