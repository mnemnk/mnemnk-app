use anyhow::Result;
use tauri::{AppHandle, Manager};

fn show_window(app: &AppHandle, label: &str) -> Result<()> {
    if let Some(window) = app.get_webview_window(label) {
        let mut url = window.url()?;
        url.set_path("/");
        window.navigate(url)?;
        if window.is_minimized()? {
            window.unminimize()?;
        }
        window.show()?;
        window.set_focus()?;
    } else {
        log::error!("window not found: {}", label);
    }
    Ok(())
}

pub fn show_main(app: &AppHandle) -> Result<()> {
    show_window(app, "main")
}

pub fn hide_main(app: &AppHandle) -> Result<()> {
    if let Some(window) = app.get_webview_window("main") {
        window.hide()?;
    }
    Ok(())
}
