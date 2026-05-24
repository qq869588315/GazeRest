use tauri::{AppHandle, LogicalSize, Manager, Size, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_positioner::{Position, WindowExt};

pub const MAIN_WINDOW_LABEL: &str = "main";
pub const REMINDER_WINDOW_LABEL: &str = "reminder";
pub const BREAK_WINDOW_LABEL: &str = "break";
pub const TRAY_ID: &str = "main-tray";

pub fn toggle_main_window(app: &AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
        return Ok(());
    };

    if window.is_minimized().map_err(|error| error.to_string())? {
        return show_main_window(app);
    }

    let is_visible = window.is_visible().map_err(|error| error.to_string())?;
    if !is_visible {
        return show_main_window(app);
    }

    let is_focused = window.is_focused().map_err(|error| error.to_string())?;
    if is_focused {
        window.hide().map_err(|error| error.to_string())?;
        return Ok(());
    }

    window.set_focus().map_err(|error| error.to_string())
}

pub fn show_main_window(app: &AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
        return Ok(());
    };

    if window.is_minimized().map_err(|error| error.to_string())? {
        window.unminimize().map_err(|error| error.to_string())?;
    }

    if !window.is_visible().map_err(|error| error.to_string())? {
        window.show().map_err(|error| error.to_string())?;
    }

    window.set_focus().map_err(|error| error.to_string())
}

pub fn minimize_main_window(app: &AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
        return Ok(());
    };

    if !window.is_visible().map_err(|error| error.to_string())? {
        window.show().map_err(|error| error.to_string())?;
    }

    window.minimize().map_err(|error| error.to_string())
}

pub fn hide_main_window(app: &AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window(MAIN_WINDOW_LABEL) else {
        return Ok(());
    };

    window.hide().map_err(|error| error.to_string())
}

pub fn show_reminder(app: &AppHandle, level: i64) -> Result<(), String> {
    close_window(app, REMINDER_WINDOW_LABEL);

    if level <= 0 {
        return Ok(());
    }

    let (width, height, position, fullscreen, always_on_top) = match level {
        1 => (372.0, 196.0, Some(Position::BottomRight), false, true),
        2 => (560.0, 380.0, Some(Position::Center), false, true),
        3 => (900.0, 620.0, None, true, true),
        _ => (380.0, 188.0, Some(Position::BottomRight), false, true),
    };
    let transparent = level == 3;

    let window = WebviewWindowBuilder::new(
        app,
        REMINDER_WINDOW_LABEL,
        WebviewUrl::App(format!("index.html?view=reminder&nonce={}", chrono::Utc::now().timestamp_millis()).into()),
    )
    .title("GazeRest Reminder")
    .inner_size(width, height)
    .visible(false)
    .resizable(false)
    .decorations(false)
    .transparent(transparent)
    .always_on_top(always_on_top)
    .skip_taskbar(true)
    .focused(level == 3)
    .build()
    .map_err(|error| error.to_string())?;

    if fullscreen {
        window
            .set_fullscreen(true)
            .map_err(|error| error.to_string())?;
        window.set_focus().map_err(|error| error.to_string())?;
    } else if let Some(position) = position {
        let _ = window.move_window(position);
    }

    window.show().map_err(|error| error.to_string())
}

pub fn hide_reminder(app: &AppHandle) -> Result<(), String> {
    close_window(app, REMINDER_WINDOW_LABEL);
    Ok(())
}

pub fn show_break(app: &AppHandle) -> Result<(), String> {
    close_window(app, BREAK_WINDOW_LABEL);

    let window = WebviewWindowBuilder::new(
        app,
        BREAK_WINDOW_LABEL,
        WebviewUrl::App(format!("index.html?view=break&nonce={}", chrono::Utc::now().timestamp_millis()).into()),
    )
    .title("GazeRest Break")
    .inner_size(360.0, 450.0)
    .visible(false)
    .resizable(false)
    .decorations(false)
    .transparent(false)
    .always_on_top(true)
    .skip_taskbar(true)
    .focused(true)
    .build()
    .map_err(|error| error.to_string())?;

    window
        .set_fullscreen(false)
        .map_err(|error| error.to_string())?;
    window
        .set_size(Size::Logical(LogicalSize::new(360.0, 450.0)))
        .map_err(|error| error.to_string())?;
    let _ = window.move_window(Position::Center);
    window.show().map_err(|error| error.to_string())?;
    window.set_focus().map_err(|error| error.to_string())
}

pub fn hide_break(app: &AppHandle) -> Result<(), String> {
    close_window(app, BREAK_WINDOW_LABEL);
    Ok(())
}

fn close_window(app: &AppHandle, label: &str) {
    if let Some(window) = app.get_webview_window(label) {
        let _ = window.close();
    }
}
