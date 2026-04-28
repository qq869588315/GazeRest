use chrono::{Duration, Utc};
use tauri::{AppHandle, Emitter, Manager, State};

use crate::models::{
    end_of_today_utc, normalize_close_button_behavior, utc_now, AppStatus, BreakSession,
    DetectedDisplaySize, PausePreset, ReminderAction, ReminderEvent, RuntimeState, Settings,
};
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
        .map_err(|error| error.to_string())
}

pub fn save_settings_inner(
    app: &AppHandle,
    state: &AppContext,
    mut settings: Settings,
) -> CommandResult<Settings> {
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
    let mut guard = state
        .volatile
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;
    if let Some(active_break) = guard.active_break.clone() {
        return Ok(active_break);
    }

    let reminder_id = triggered_by_reminder_event_id
        .or_else(|| guard.active_reminder.as_ref().map(|item| item.id));
    if let Some(id) = reminder_id {
        state
            .db
            .update_reminder_action(id, ReminderAction::StartedBreak, None)?;
    }

    guard.break_generation += 1;
    let generation = guard.break_generation;
    let now = utc_now();
    let mut session = BreakSession {
        id: 0,
        started_at: now.clone(),
        ended_at: None,
        duration_seconds: guard.settings.break_duration_seconds,
        status: "running".into(),
        cancel_reason: None,
        triggered_by_reminder_event_id: reminder_id,
        created_at: now.clone(),
        style: guard.settings.timer_style,
        remaining_seconds: guard.settings.break_duration_seconds,
    };
    let session_id = state.db.insert_break_session(&session)?;
    session.id = session_id;

    clear_pending_reminder(&mut guard.runtime);
    guard.active_reminder = None;
    guard.active_break = Some(session.clone());
    guard.runtime.current_status = AppStatus::BreakInProgress;
    guard.runtime.updated_at = now;
    state.db.save_runtime(&guard.runtime)?;
    drop(guard);

    crate::windows::hide_reminder(app)?;
    crate::windows::show_break(app)?;
    app.emit("reminder-issued", Option::<ReminderEvent>::None)
        .map_err(|error| error.to_string())?;
    app.emit("break-tick", session.clone())
        .map_err(|error| error.to_string())?;
    emit_snapshot(app)?;
    crate::scheduler::spawn_break_countdown(app.clone(), generation);
    Ok(session)
}

pub fn cancel_break_inner(app: &AppHandle, state: &AppContext) -> CommandResult<()> {
    let mut guard = state
        .volatile
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;
    let Some(mut session) = guard.active_break.take() else {
        return Ok(());
    };

    guard.break_generation += 1;
    let completed = session.remaining_seconds <= 0;
    let status = if completed { "completed" } else { "canceled" };
    let cancel_reason = if completed {
        None
    } else {
        Some("user_cancelled")
    };
    session.status = status.into();
    session.cancel_reason = cancel_reason.map(str::to_string);
    session.ended_at = Some(utc_now());
    state
        .db
        .finish_break_session(session.id, status, cancel_reason)?;
    let settings = guard.settings.clone();
    reset_runtime_for_next_round(&settings, &mut guard.runtime);
    state.db.save_runtime(&guard.runtime)?;
    drop(guard);

    crate::windows::hide_break(app)?;
    app.emit("break-finished", Some(session))
        .map_err(|error| error.to_string())?;
    emit_snapshot(app)
}

pub fn snooze_reminder_inner(app: &AppHandle, state: &AppContext) -> CommandResult<()> {
    let mut guard = state
        .volatile
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;
    if let Some(reminder) = guard.active_reminder.as_ref() {
        state
            .db
            .update_reminder_action(reminder.id, ReminderAction::Snoozed, Some(5))?;
    }

    guard.active_reminder = None;
    guard.runtime.current_status = AppStatus::Snoozed;
    guard.runtime.next_reminder_due_at = Some((Utc::now() + Duration::minutes(5)).to_rfc3339());
    guard.runtime.updated_at = utc_now();
    clear_pending_reminder(&mut guard.runtime);
    state.db.save_runtime(&guard.runtime)?;
    drop(guard);

    crate::windows::hide_reminder(app)?;
    app.emit("reminder-issued", Option::<ReminderEvent>::None)
        .map_err(|error| error.to_string())?;
    emit_snapshot(app)
}

pub fn skip_reminder_inner(app: &AppHandle, state: &AppContext) -> CommandResult<()> {
    let mut guard = state
        .volatile
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;
    if let Some(reminder) = guard.active_reminder.as_ref() {
        state
            .db
            .update_reminder_action(reminder.id, ReminderAction::Skipped, None)?;
    }

    guard.active_reminder = None;
    guard.runtime.current_status = AppStatus::Running;
    guard.runtime.active_elapsed_seconds = 0;
    guard.runtime.updated_at = utc_now();
    clear_pending_reminder(&mut guard.runtime);
    guard.runtime.next_reminder_due_at = next_due_at(&guard.settings, &guard.runtime);
    state.db.save_runtime(&guard.runtime)?;
    drop(guard);

    crate::windows::hide_reminder(app)?;
    app.emit("reminder-issued", Option::<ReminderEvent>::None)
        .map_err(|error| error.to_string())?;
    emit_snapshot(app)
}

pub fn pause_app_inner(
    app: &AppHandle,
    state: &AppContext,
    preset: PausePreset,
) -> CommandResult<()> {
    let mut guard = state
        .volatile
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;
    if let Some(reminder) = guard.active_reminder.as_ref() {
        let _ = state
            .db
            .update_reminder_action(reminder.id, ReminderAction::Ignored, None);
    }

    guard.active_reminder = None;
    guard.runtime.current_status = AppStatus::Paused;
    guard.runtime.paused_until = Some(match preset {
        PausePreset::Minutes30 => (Utc::now() + Duration::minutes(30)).to_rfc3339(),
        PausePreset::Hour1 => (Utc::now() + Duration::hours(1)).to_rfc3339(),
        PausePreset::Today => end_of_today_utc().to_rfc3339(),
    });
    guard.runtime.updated_at = utc_now();
    clear_pending_reminder(&mut guard.runtime);
    state.db.save_runtime(&guard.runtime)?;
    drop(guard);

    crate::windows::hide_reminder(app)?;
    emit_snapshot(app)
}

pub fn resume_app_inner(app: &AppHandle, state: &AppContext) -> CommandResult<()> {
    let mut guard = state
        .volatile
        .lock()
        .map_err(|_| "state lock poisoned".to_string())?;
    guard.runtime.current_status = if crate::platform::within_schedule(&guard.settings) {
        AppStatus::Running
    } else {
        AppStatus::OutsideSchedule
    };
    guard.runtime.paused_until = None;
    guard.runtime.active_elapsed_seconds = 0;
    guard.runtime.updated_at = utc_now();
    guard.runtime.next_reminder_due_at = next_due_at(&guard.settings, &guard.runtime);
    state.db.save_runtime(&guard.runtime)?;
    drop(guard);

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

pub fn reset_runtime_for_next_round(settings: &Settings, runtime: &mut RuntimeState) {
    runtime.current_status = if crate::platform::within_schedule(settings) {
        AppStatus::Running
    } else {
        AppStatus::OutsideSchedule
    };
    runtime.active_elapsed_seconds = 0;
    runtime.paused_until = None;
    runtime.updated_at = utc_now();
    runtime.next_reminder_due_at = if matches!(runtime.current_status, AppStatus::Running) {
        next_due_at(settings, runtime)
    } else {
        None
    };
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
