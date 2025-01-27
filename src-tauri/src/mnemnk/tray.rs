use anyhow::Result;
use tauri::AppHandle;
use tauri::{
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
};

use crate::mnemnk;

pub fn init(app: &AppHandle) -> Result<()> {
    let quit = MenuItemBuilder::with_id("quit", "Quit").build(app)?;
    let show = MenuItemBuilder::with_id("show", "Show").build(app)?;

    let menu = MenuBuilder::new(app).items(&[&show, &quit]).build()?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .icon_as_template(true)
        .tooltip(&app.package_info().name)
        .menu(&menu)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "quit" => {
                app.exit(0);
            }
            "show" => mnemnk::window::show_main(app).unwrap_or_else(|e| {
                log::error!("Failed to show main window: {}", e);
            }),
            _ => {}
        })
        .build(app)?;

    Ok(())
}
