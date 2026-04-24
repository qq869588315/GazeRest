use std::{thread, time::Duration};

use chrono::{DateTime, Utc};
use tauri::{AppHandle, Emitter, Manager};

use crate::{
    commands,
    models::{utc_now, AppStatus, ReminderEvent},
    state::AppContext,
};

const IDLE_PAUSE_SECONDS: u64 = 60;
const SLEEP_RESET_SECONDS: i64 = 15 * 60;

pub fn spawn_monitor(app: &AppHandle) {
    let app_handle = app.clone();
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(1));
        if let Err(error) = tick(&app_handle) {
            log::error!("后台轮询失败: {error}");
        }
    });
}

pub fn spawn_break_countdown(app: AppHandle, generation: u64) {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(1));
        let state = app.state::<AppContext>();
        let mut guard = match state.volatile.lock() {
            Ok(guard) => guard,
            Err(_) => return,
        };

        if guard.break_generation != generation {
            return;
        }

        let Some(active_break) = guard.active_break.as_mut() else {
            return;
        };

        if active_break.remaining_seconds <= 0 {
            return;
        }

        active_break.remaining_seconds -= 1;
        let tick_payload = active_break.clone();

        if active_break.remaining_seconds > 0 {
            drop(guard);
            let _ = app.emit("break-tick", Some(tick_payload));
            let _ = commands::emit_snapshot(&app);
            continue;
        }

        let mut finished = tick_payload.clone();
        finished.status = "completed".into();
        finished.ended_at = Some(utc_now());
        let session_id = finished.id;

        guard.active_break = None;
        guard.runtime.current_status = AppStatus::Paused;
        guard.runtime.active_elapsed_seconds = 0;
        guard.runtime.paused_until = None;
        guard.runtime.updated_at = utc_now();
        guard.runtime.next_reminder_due_at = None;
        if let Err(error) = state.db.finish_break_session(session_id, "completed", None) {
            log::error!("更新休息会话失败: {error}");
        }
        if let Err(error) = state.db.save_runtime(&guard.runtime) {
            log::error!("保存运行状态失败: {error}");
        }
        drop(guard);

        let _ = crate::windows::hide_break(&app);
        let _ = app.emit("break-finished", Some(finished));
        let _ = commands::emit_snapshot(&app);
        return;
    });
}

fn tick(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<AppContext>();
    let idle_seconds = crate::platform::idle_seconds();
    let is_fullscreen = crate::platform::is_fullscreen(app);
    let now_timestamp = Utc::now().timestamp();

    let mut reminder_to_emit: Option<ReminderEvent> = None;
    let mut auto_hide = false;

    {
        let mut guard = state
            .volatile
            .lock()
            .map_err(|_| "状态锁已损坏".to_string())?;
        let delta = (now_timestamp - guard.last_tick_unix).max(1) as i64;
        guard.last_tick_unix = now_timestamp;

        let mut pause_expired = false;

        if let Some(paused_until) = guard
            .runtime
            .paused_until
            .as_ref()
            .and_then(|value| parse_utc(value))
        {
            if paused_until > Utc::now() {
                guard.runtime.current_status = AppStatus::Paused;
                guard.runtime.updated_at = utc_now();
                state.db.save_runtime(&guard.runtime)?;
                drop(guard);
                return commands::emit_snapshot(app);
            }

            guard.runtime.paused_until = None;
            pause_expired = true;
        }

        if guard.active_break.is_some() {
            drop(guard);
            return commands::emit_snapshot(app);
        }

        if matches!(guard.runtime.current_status, AppStatus::Paused)
            && guard.runtime.paused_until.is_none()
            && !pause_expired
        {
            guard.runtime.next_reminder_due_at = None;
            guard.runtime.updated_at = utc_now();
            state.db.save_runtime(&guard.runtime)?;
            drop(guard);
            return commands::emit_snapshot(app);
        }

        if !crate::platform::within_schedule(&guard.settings) {
            guard.runtime.current_status = AppStatus::OutsideSchedule;
            guard.runtime.next_reminder_due_at = None;
            guard.runtime.updated_at = utc_now();
            state.db.save_runtime(&guard.runtime)?;
            drop(guard);
            return commands::emit_snapshot(app);
        }

        if matches!(
            guard.runtime.current_status,
            AppStatus::OutsideSchedule | AppStatus::Paused
        ) {
            guard.runtime.current_status = AppStatus::Running;
        }

        if delta >= SLEEP_RESET_SECONDS {
            guard.runtime.active_elapsed_seconds = 0;
            guard.runtime.last_idle_detected_at = Some(utc_now());
        } else if idle_seconds >= IDLE_PAUSE_SECONDS {
            guard.runtime.last_idle_detected_at = Some(utc_now());
        } else if guard.active_reminder.is_none()
            && !matches!(guard.runtime.current_status, AppStatus::Snoozed)
        {
            guard.runtime.active_elapsed_seconds += delta;
            guard.runtime.last_activity_at = Some(utc_now());
        }

        let mut should_issue = false;
        let mut trigger_reason = "interval_due".to_string();
        let mut fullscreen_delayed = false;

        if matches!(guard.runtime.current_status, AppStatus::Snoozed) {
            if let Some(next_due) = guard
                .runtime
                .next_reminder_due_at
                .as_ref()
                .and_then(|value| parse_utc(value))
            {
                if next_due <= Utc::now() && guard.active_reminder.is_none() {
                    should_issue = true;
                    trigger_reason = "snooze_due".into();
                }
            }
        } else if guard.runtime.deferred_reminder_pending && !is_fullscreen {
            should_issue = true;
            trigger_reason = "fullscreen_release".into();
            fullscreen_delayed = true;
        } else if guard.active_reminder.is_none()
            && guard.runtime.active_elapsed_seconds >= guard.settings.reminder_interval_minutes * 60
        {
            if is_fullscreen && guard.settings.fullscreen_delay_enabled {
                guard.runtime.deferred_reminder_pending = true;
                guard.runtime.last_fullscreen_detected_at = Some(utc_now());
            } else {
                should_issue = true;
            }
        }

        if should_issue {
            let display_mode = if guard.settings.reminder_level == 0 {
                "status"
            } else if guard.settings.reminder_level == 3 {
                "immersive"
            } else {
                "card"
            };

            let mut reminder = ReminderEvent {
                id: 0,
                triggered_at: utc_now(),
                trigger_reason,
                reminder_level: guard.settings.reminder_level,
                was_fullscreen_delayed: fullscreen_delayed,
                delivery_type: display_mode.into(),
                user_action: None,
                action_at: None,
                deferred_minutes: None,
                active_elapsed_seconds: guard.runtime.active_elapsed_seconds,
                created_at: utc_now(),
                display_mode: display_mode.into(),
            };
            let reminder_id = state.db.insert_reminder_event(&reminder)?;
            reminder.id = reminder_id;
            guard.active_reminder = Some(reminder.clone());
            guard.runtime.pending_reminder_event_id = Some(reminder_id);
            guard.runtime.pending_reminder_level = Some(reminder.reminder_level);
            guard.runtime.deferred_reminder_pending = false;
            guard.runtime.current_status = AppStatus::Running;
            guard.runtime.next_reminder_due_at = None;
            reminder_to_emit = Some(reminder);
            auto_hide = guard.settings.reminder_level == 1;
        } else if guard.active_reminder.is_none() {
            guard.runtime.next_reminder_due_at =
                commands::next_due_at(&guard.settings, &guard.runtime);
        }

        guard.runtime.updated_at = utc_now();
        state.db.save_runtime(&guard.runtime)?;
    }

    if let Some(reminder) = reminder_to_emit.clone() {
        if reminder.reminder_level > 0 {
            let low_distraction_mode = {
                let guard = state
                    .volatile
                    .lock()
                    .map_err(|_| "state lock poisoned".to_string())?;
                guard.settings.low_distraction_mode
            };
            crate::windows::show_reminder(app, reminder.reminder_level, low_distraction_mode)?;
        }
        app.emit("reminder-issued", Some(reminder.clone()))
            .map_err(|error| error.to_string())?;

        if auto_hide {
            schedule_auto_hide(app.clone(), reminder.id);
        }
    }

    commands::emit_snapshot(app)
}

fn schedule_auto_hide(app: AppHandle, reminder_id: i64) {
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(8));
        let state = app.state::<AppContext>();
        let Ok(guard) = state.volatile.lock() else {
            return;
        };
        let should_hide = guard
            .active_reminder
            .as_ref()
            .map(|item| item.id == reminder_id && item.reminder_level == 1)
            .unwrap_or(false);
        drop(guard);

        if should_hide {
            let _ = crate::windows::hide_reminder(&app);
        }
    });
}

fn parse_utc(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|parsed| parsed.with_timezone(&Utc))
}
