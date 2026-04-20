mod commands;
mod db;
mod models;
mod platform;
mod scheduler;
mod state;
mod windows;

use tauri::{
    menu::MenuBuilder,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, WindowEvent,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = windows::show_main_window(app);
        }))
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            None::<Vec<&str>>,
        ))
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .setup(|app| {
            let database = db::Database::new(app.handle())?;
            let settings = database.load_or_init_settings()?;
            let runtime = database.load_or_init_runtime()?;
            app.manage(state::AppContext::new(database, settings, runtime));
            windows::ensure_aux_windows(app.handle())?;
            build_tray(app.handle())?;
            scheduler::spawn_monitor(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::bootstrap_app,
            commands::get_settings,
            commands::save_settings,
            commands::get_runtime_state,
            commands::get_today_summary,
            commands::start_break,
            commands::cancel_break,
            commands::snooze_reminder,
            commands::skip_reminder,
            commands::pause_app,
            commands::resume_app,
            commands::minimize_main_window,
            commands::quit_app,
            commands::detect_display_size,
        ])
        .on_window_event(|window, event| {
            if window.label() != windows::MAIN_WINDOW_LABEL {
                return;
            }

            if let WindowEvent::CloseRequested { api, .. } = event {
                let exit_requested = window
                    .app_handle()
                    .state::<state::AppContext>()
                    .volatile
                    .lock()
                    .map(|guard| guard.exit_requested)
                    .unwrap_or(false);

                if !exit_requested {
                    api.prevent_close();
                    let _ = window.emit("close-intent", ());
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn build_tray(app: &AppHandle) -> Result<(), String> {
    let tray_menu = MenuBuilder::new(app)
        .text("show", "显示面板")
        .text("start_break", "开始 20 秒休息")
        .separator()
        .text("pause_30", "暂停 30 分钟")
        .text("pause_60", "暂停 1 小时")
        .text("pause_today", "今天不再提醒")
        .text("resume", "立即恢复")
        .separator()
        .text("quit", "退出 GazeRest")
        .build()
        .map_err(|error| error.to_string())?;

    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| "missing default app icon".to_string())?;

    TrayIconBuilder::with_id(windows::TRAY_ID)
        .icon(icon)
        .tooltip("GazeRest")
        .menu(&tray_menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, event| match event.id().as_ref() {
            "show" => {
                let _ = windows::show_main_window(app);
            }
            "start_break" => {
                let state = app.state::<state::AppContext>();
                let _ = commands::start_break_inner(app, &state, None);
            }
            "pause_30" => {
                let state = app.state::<state::AppContext>();
                let _ = commands::pause_app_inner(app, &state, models::PausePreset::Minutes30);
            }
            "pause_60" => {
                let state = app.state::<state::AppContext>();
                let _ = commands::pause_app_inner(app, &state, models::PausePreset::Hour1);
            }
            "pause_today" => {
                let state = app.state::<state::AppContext>();
                let _ = commands::pause_app_inner(app, &state, models::PausePreset::Today);
            }
            "resume" => {
                let state = app.state::<state::AppContext>();
                let _ = commands::resume_app_inner(app, &state);
            }
            "quit" => {
                let state = app.state::<state::AppContext>();
                let _ = commands::quit_app_inner(app, &state);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                let _ = windows::toggle_main_window(&tray.app_handle());
            }
        })
        .build(app)
        .map_err(|error| error.to_string())?;

    Ok(())
}
