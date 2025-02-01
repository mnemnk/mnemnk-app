use anyhow::Result;
use tauri::{AppHandle, Manager, Url};

fn show_window(app: &AppHandle, label: &str) -> Result<()> {
    if let Some(mut window) = app.get_webview_window(label) {
        // window.navigate(window.url()?)?;
        window.navigate(Url::parse(
            window.url()?.origin().ascii_serialization().as_str(),
        )?)?;
        if window.is_minimized()? {
            window.unminimize()?;
        }
        if window.is_visible()? {
            window.set_focus()?;
        }
        window.show()?;
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
