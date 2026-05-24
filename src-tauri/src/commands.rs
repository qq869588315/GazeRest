use chrono::{Duration, Utc};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::models::{
    end_of_today_utc, normalize_close_button_behavior, normalize_settings, utc_now, AppStatus,
    BreakSession, DetectedDisplaySize, PausePreset, RuntimeState, Settings,
};
use crate::runtime_service::RuntimeEffect;
use crate::state::AppContext;

pub type CommandResult<T> = Result<T, String>;

#[tauri::command]
pub fn bootstrap_app(
    state: State<'_, AppContext>,
) -> CommandResult<crate::models::BootstrapPayload> {
    state.snapshot()
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppContext>) -> CommandResult<Settings> {
    let guard = state
        .volatile
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;
    Ok(guard.settings.clone())
}

#[tauri::command]
pub fn save_settings(
    app: AppHandle,
    state: State<'_, AppContext>,
    mut settings: Settings,
) -> CommandResult<Settings> {
    settings.updated_at = utc_now();
    save_settings_inner(&app, &state, settings)
}

#[tauri::command]
pub fn get_runtime_state(state: State<'_, AppContext>) -> CommandResult<RuntimeState> {
    let guard = state
        .volatile
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;
    Ok(guard.runtime.clone())
}

#[tauri::command]
pub fn get_today_summary(
    state: State<'_, AppContext>,
) -> CommandResult<crate::models::TodaySummary> {
    state.db.get_today_summary()
}

#[tauri::command]
pub fn start_break(
    app: AppHandle,
    state: State<'_, AppContext>,
    triggered_by_reminder_event_id: Option<i64>,
) -> CommandResult<BreakSession> {
    start_break_inner(&app, &state, triggered_by_reminder_event_id)
}

#[tauri::command]
pub fn cancel_break(app: AppHandle, state: State<'_, AppContext>) -> CommandResult<()> {
    cancel_break_inner(&app, &state)
}

#[tauri::command]
pub fn snooze_reminder(app: AppHandle, state: State<'_, AppContext>) -> CommandResult<()> {
    snooze_reminder_inner(&app, &state)
}

#[tauri::command]
pub fn skip_reminder(app: AppHandle, state: State<'_, AppContext>) -> CommandResult<()> {
    skip_reminder_inner(&app, &state)
}

#[tauri::command]
pub fn pause_app(
    app: AppHandle,
    state: State<'_, AppContext>,
    preset: PausePreset,
) -> CommandResult<()> {
    pause_app_inner(&app, &state, preset)
}

#[tauri::command]
pub fn resume_app(app: AppHandle, state: State<'_, AppContext>) -> CommandResult<()> {
    resume_app_inner(&app, &state)
}

#[tauri::command]
pub fn minimize_main_window(app: AppHandle) -> CommandResult<()> {
    minimize_main_window_inner(&app)
}

#[tauri::command]
pub fn hide_main_window(app: AppHandle) -> CommandResult<()> {
    hide_main_window_inner(&app)
}

#[tauri::command]
pub fn quit_app(app: AppHandle, state: State<'_, AppContext>) -> CommandResult<()> {
    quit_app_inner(&app, &state)
}

#[tauri::command]
pub fn detect_display_size() -> CommandResult<DetectedDisplaySize> {
    crate::platform::detect_display_size()
}

pub fn emit_snapshot(app: &AppHandle) -> CommandResult<()> {
    let payload = app.state::<AppContext>().snapshot()?;
    app.emit("state-updated", payload)
        .map_err(|error| error.to_string())?;
    crate::tray_service::sync_tray(app);
    Ok(())
}

pub fn apply_effects(app: &AppHandle, effects: Vec<RuntimeEffect>) -> CommandResult<()> {
    for effect in effects {
        match effect {
            RuntimeEffect::ShowReminder(level) => crate::windows::show_reminder(app, level)?,
            RuntimeEffect::HideReminder => defer_window_action(app, |app| {
                let _ = crate::windows::hide_reminder(&app);
            }),
            RuntimeEffect::ShowBreak => defer_window_action(app, |app| {
                let _ = crate::windows::show_break(&app);
            }),
            RuntimeEffect::HideBreak => defer_window_action(app, |app| {
                let _ = crate::windows::hide_break(&app);
            }),
            RuntimeEffect::PlayReminderSound => crate::sound_service::play_reminder_sound(app),
            RuntimeEffect::EmitReminder(reminder) => app
                .emit("reminder-issued", reminder)
                .map_err(|error| error.to_string())?,
            RuntimeEffect::EmitBreakTick(session) => app
                .emit("break-tick", session)
                .map_err(|error| error.to_string())?,
            RuntimeEffect::EmitBreakFinished(session) => app
                .emit("break-finished", session)
                .map_err(|error| error.to_string())?,
            RuntimeEffect::AutoHideReminder { reminder_id } => {
                crate::scheduler::schedule_auto_hide(app.clone(), reminder_id)
            }
        }
    }
    crate::tray_service::sync_tray(app);
    Ok(())
}

fn defer_window_action<F>(app: &AppHandle, action: F)
where
    F: FnOnce(AppHandle) + Send + 'static,
{
    let app = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(120));
        action(app);
    });
}

pub fn save_settings_inner(
    app: &AppHandle,
    state: &AppContext,
    mut settings: Settings,
) -> CommandResult<Settings> {
    normalize_settings(&mut settings);
    settings.window_opacity = settings.window_opacity.clamp(0.0, 1.0);
    settings.close_button_behavior =
        normalize_close_button_behavior(&settings.close_button_behavior).into();
    state.db.save_settings(&settings)?;

    {
        let mut guard = state
            .volatile
            .lock()
            .map_err(|_| "state lock poisoned".to_string())?;
        guard.settings = settings.clone();
        crate::runtime_service::rollover_today(&mut guard.runtime);
        if !crate::platform::within_schedule(&guard.settings) {
            guard.runtime.current_status = AppStatus::OutsideSchedule;
        } else if matches!(guard.runtime.current_status, AppStatus::OutsideSchedule) {
            guard.runtime.current_status = AppStatus::Running;
        }
        guard.runtime.updated_at = utc_now();
        guard.runtime.next_reminder_due_at = next_due_at(&guard.settings, &guard.runtime);
        state.db.save_runtime(&guard.runtime)?;
    }

    sync_autostart(app, settings.launch_at_startup);
    app.emit("settings-updated", settings.clone())
        .map_err(|error| error.to_string())?;
    emit_snapshot(app)?;
    Ok(settings)
}

pub fn start_break_inner(
    app: &AppHandle,
    state: &AppContext,
    triggered_by_reminder_event_id: Option<i64>,
) -> CommandResult<BreakSession> {
    let (session, effects) = {
        let mut guard = state
            .volatile
            .lock()
            .map_err(|_| "state lock poisoned".to_string())?;
        crate::runtime_service::start_break(&state.db, &mut guard, triggered_by_reminder_event_id)?
    };

    apply_effects(app, effects)?;
    emit_snapshot(app)?;
    Ok(session)
}

pub fn cancel_break_inner(app: &AppHandle, state: &AppContext) -> CommandResult<()> {
    let effects = {
        let mut guard = state
            .volatile
            .lock()
            .map_err(|_| "state lock poisoned".to_string())?;
        crate::runtime_service::cancel_break(&state.db, &mut guard)?
    };
    apply_effects(app, effects)?;
    emit_snapshot(app)
}

pub fn snooze_reminder_inner(app: &AppHandle, state: &AppContext) -> CommandResult<()> {
    let effects = {
        let mut guard = state
            .volatile
            .lock()
            .map_err(|_| "state lock poisoned".to_string())?;
        crate::runtime_service::snooze_reminder(&state.db, &mut guard)?
    };
    apply_effects(app, effects)?;
    emit_snapshot(app)
}

pub fn skip_reminder_inner(app: &AppHandle, state: &AppContext) -> CommandResult<()> {
    let effects = {
        let mut guard = state
            .volatile
            .lock()
            .map_err(|_| "state lock poisoned".to_string())?;
        crate::runtime_service::skip_reminder(&state.db, &mut guard)?
    };
    apply_effects(app, effects)?;
    emit_snapshot(app)
}

pub fn pause_app_inner(
    app: &AppHandle,
    state: &AppContext,
    preset: PausePreset,
) -> CommandResult<()> {
    let paused_until = match preset {
        PausePreset::Minutes30 => (Utc::now() + Duration::minutes(30)).to_rfc3339(),
        PausePreset::Hour1 => (Utc::now() + Duration::hours(1)).to_rfc3339(),
        PausePreset::Today => end_of_today_utc().to_rfc3339(),
    };
    let effects = {
        let mut guard = state
            .volatile
            .lock()
            .map_err(|_| "state lock poisoned".to_string())?;
        crate::runtime_service::pause(&state.db, &mut guard, paused_until)?
    };

    apply_effects(app, effects)?;
    emit_snapshot(app)
}

pub fn resume_app_inner(app: &AppHandle, state: &AppContext) -> CommandResult<()> {
    {
        let mut guard = state
            .volatile
            .lock()
            .map_err(|_| "state lock poisoned".to_string())?;
        crate::runtime_service::resume(&state.db, &mut guard)?;
    }

    emit_snapshot(app)
}

pub fn minimize_main_window_inner(app: &AppHandle) -> CommandResult<()> {
    crate::windows::minimize_main_window(app)
}

pub fn hide_main_window_inner(app: &AppHandle) -> CommandResult<()> {
    crate::windows::hide_main_window(app)
}

pub fn quit_app_inner(app: &AppHandle, state: &AppContext) -> CommandResult<()> {
    let mut guard = state
        .volatile
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;
    guard.exit_requested = true;
    drop(guard);
    app.exit(0);
    Ok(())
}

pub fn clear_pending_reminder(runtime: &mut RuntimeState) {
    runtime.pending_reminder_event_id = None;
    runtime.pending_reminder_level = None;
    runtime.deferred_reminder_pending = false;
}

pub fn next_due_at(settings: &Settings, runtime: &RuntimeState) -> Option<String> {
    let interval_seconds = settings.reminder_interval_minutes * 60;
    let remaining = (interval_seconds - runtime.active_elapsed_seconds).max(0);
    Some((Utc::now() + Duration::seconds(remaining)).to_rfc3339())
}

fn sync_autostart(app: &AppHandle, enabled: bool) {
    #[cfg(desktop)]
    {
        use tauri_plugin_autostart::ManagerExt;

        let manager = app.autolaunch();
        let outcome = if enabled {
            manager.enable()
        } else {
            manager.disable()
        };

        if let Err(error) = outcome {
            log::warn!("failed to sync autostart setting: {error}");
        }
    }
}
