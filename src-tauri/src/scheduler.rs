use std::{thread, time::Duration};

use chrono::Utc;
use tauri::{AppHandle, Manager};

use crate::{
    commands,
    runtime_service::{self, TickInput},
    state::AppContext,
};

pub fn spawn_monitor(app: &AppHandle) {
    let app_handle = app.clone();
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(1));
        if let Err(error) = tick(&app_handle) {
            log::error!("background monitor failed: {error}");
        }
    });
}

fn tick(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<AppContext>();
    let effects = {
        let mut guard = state
            .volatile
            .lock()
            .map_err(|_| "state lock poisoned".to_string())?;
        runtime_service::tick(
            &state.db,
            &mut guard,
            TickInput {
                idle_seconds: crate::platform::idle_seconds(),
                is_fullscreen: crate::platform::is_fullscreen(app),
                now_timestamp: Utc::now().timestamp(),
            },
        )?
    };

    commands::apply_effects(app, effects)?;
    commands::emit_snapshot(app)
}

pub fn schedule_auto_hide(app: AppHandle, reminder_id: i64) {
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(8));
        let state = app.state::<AppContext>();
        let Ok(mut guard) = state.volatile.lock() else {
            return;
        };
        let should_hide = guard
            .active_reminder
            .as_ref()
            .map(|item| item.id == reminder_id && item.reminder_level == 1)
            .unwrap_or(false);
        if should_hide {
            guard.active_reminder = None;
            if matches!(
                guard.runtime.current_status,
                crate::models::AppStatus::ReminderPending
            ) {
                let settings = guard.settings.clone();
                runtime_service::reset_runtime_for_next_round(&settings, &mut guard.runtime);
                let _ = state.db.save_runtime(&guard.runtime);
            }
        }
        drop(guard);

        if should_hide {
            let _ = crate::windows::hide_reminder(&app);
            let _ = commands::emit_snapshot(&app);
        }
    });
}
