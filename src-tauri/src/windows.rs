use tauri::{AppHandle, LogicalSize, Manager, Size, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_positioner::{Position, WindowExt};

pub const MAIN_WINDOW_LABEL: &str = "main";
pub const REMINDER_WINDOW_LABEL: &str = "reminder";
pub const BREAK_WINDOW_LABEL: &str = "break";
pub const TRAY_ID: &str = "main-tray";

pub fn ensure_aux_windows(app: &AppHandle) -> Result<(), String> {
    if app.get_webview_window(REMINDER_WINDOW_LABEL).is_none() {
        WebviewWindowBuilder::new(
            app,
            REMINDER_WINDOW_LABEL,
            WebviewUrl::App("index.html?view=reminder".into()),
        )
        .title("GazeRest Reminder")
        .inner_size(460.0, 300.0)
        .visible(false)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(false)
        .build()
        .map_err(|error| error.to_string())?;
    }

    if app.get_webview_window(BREAK_WINDOW_LABEL).is_none() {
        WebviewWindowBuilder::new(
            app,
            BREAK_WINDOW_LABEL,
            WebviewUrl::App("index.html?view=break".into()),
        )
        .title("GazeRest Break")
        .inner_size(360.0, 450.0)
        .visible(false)
        .resizable(false)
        .decorations(false)
        .transparent(true)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(false)
        .build()
        .map_err(|error| error.to_string())?;
    }

    Ok(())
}

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

pub fn show_reminder(
    app: &AppHandle,
    level: i64,
    low_distraction_mode: bool,
) -> Result<(), String> {
    let Some(window) = app.get_webview_window(REMINDER_WINDOW_LABEL) else {
        return Ok(());
    };

    window
        .set_fullscreen(false)
        .map_err(|error| error.to_string())?;

    match level {
        1 => {
            window
                .set_size(Size::Logical(LogicalSize::new(420.0, 240.0)))
                .map_err(|error| error.to_string())?;
            let _ = window.move_window(Position::BottomRight);
        }
        2 => {
            window
                .set_size(Size::Logical(LogicalSize::new(660.0, 460.0)))
                .map_err(|error| error.to_string())?;
            let _ = window.move_window(Position::Center);
        }
        3 => {}
        _ => {}
    }

    window.show().map_err(|error| error.to_string())?;
    if level == 3 {
        window
            .set_fullscreen(true)
            .map_err(|error| error.to_string())?;
    } else if !low_distraction_mode && level >= 2 {
        window.set_focus().map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub fn hide_reminder(app: &AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window(REMINDER_WINDOW_LABEL) else {
        return Ok(());
    };
    window.hide().map_err(|error| error.to_string())
}

pub fn show_break(app: &AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window(BREAK_WINDOW_LABEL) else {
        return Ok(());
    };
    window
        .set_fullscreen(false)
        .map_err(|error| error.to_string())?;
    window
        .set_size(Size::Logical(LogicalSize::new(360.0, 450.0)))
        .map_err(|error| error.to_string())?;
    window.show().map_err(|error| error.to_string())?;
    let _ = window.move_window(Position::Center);
    Ok(())
}

pub fn hide_break(app: &AppHandle) -> Result<(), String> {
    let Some(window) = app.get_webview_window(BREAK_WINDOW_LABEL) else {
        return Ok(());
    };
    window.hide().map_err(|error| error.to_string())
}
