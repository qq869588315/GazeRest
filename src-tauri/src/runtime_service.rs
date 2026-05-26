use chrono::{DateTime, Duration, Utc};

use crate::{
    commands,
    models::{
        local_date_key, utc_now, AppStatus, BreakSession, ReminderAction, ReminderEvent, RuntimeState,
        Settings,
    },
    state::VolatileState,
};

const IDLE_PAUSE_SECONDS: u64 = 60;
const SLEEP_RESET_SECONDS: i64 = 15 * 60;

#[derive(Debug, Clone)]
pub enum RuntimeEffect {
    ShowReminder(i64),
    HideReminder,
    ShowBreak,
    HideBreak,
    PlayReminderSound,
    EmitReminder(Option<ReminderEvent>),
    EmitBreakTick(Option<BreakSession>),
    EmitBreakFinished(Option<BreakSession>),
    AutoHideReminder { reminder_id: i64 },
}

#[derive(Debug, Clone)]
pub struct TickInput {
    pub idle_seconds: u64,
    pub is_fullscreen: bool,
    pub now_timestamp: i64,
}

pub fn reconcile_startup(
    db: &crate::db::Database,
    settings: &Settings,
    runtime: &mut RuntimeState,
    active_break: &mut Option<BreakSession>,
) -> Result<(), String> {
    db.finish_all_running_break_sessions("interrupted", Some("app_restarted"))?;
    if let Some(reminder_id) = runtime.pending_reminder_event_id {
        db.update_reminder_action(reminder_id, ReminderAction::Ignored, None)?;
    }

    *active_break = None;
    reset_runtime_for_process_start(settings, runtime);
    db.save_runtime(runtime)
}

pub fn reconcile_snapshot(db: &crate::db::Database, guard: &mut VolatileState) -> Result<(), String> {
    rollover_today(&mut guard.runtime);

    if let Some(active_break) = guard.active_break.as_mut() {
        refresh_break_remaining(active_break);
        if active_break.remaining_seconds <= 0 {
            let finished = finish_active_break(db, guard, None)?;
            guard.runtime.current_status = AppStatus::BreakCompleted;
            guard.runtime.updated_at = utc_now();
            db.save_runtime(&guard.runtime)?;
            log::info!("break session completed during snapshot: {}", finished.id);
        }
    }

    if guard.active_break.is_none() && matches!(guard.runtime.current_status, AppStatus::BreakInProgress) {
        reset_runtime_for_next_round(&guard.settings, &mut guard.runtime);
        db.save_runtime(&guard.runtime)?;
    }

    if guard.active_reminder.is_none() && matches!(guard.runtime.current_status, AppStatus::ReminderPending) {
        guard.runtime.current_status = AppStatus::Running;
        guard.runtime.next_reminder_due_at = commands::next_due_at(&guard.settings, &guard.runtime);
        guard.runtime.updated_at = utc_now();
        db.save_runtime(&guard.runtime)?;
    }

    Ok(())
}

pub fn tick(
    db: &crate::db::Database,
    guard: &mut VolatileState,
    input: TickInput,
) -> Result<Vec<RuntimeEffect>, String> {
    let mut effects = Vec::new();
    rollover_today(&mut guard.runtime);
    let delta = (input.now_timestamp - guard.last_tick_unix).max(1);
    guard.last_tick_unix = input.now_timestamp;

    if let Some(active_break) = guard.active_break.as_mut() {
        refresh_break_remaining(active_break);
        if active_break.remaining_seconds <= 0 {
            let auto_close = guard.settings.auto_close_break_window;
            let finished = finish_active_break(db, guard, None)?;
            effects.push(RuntimeEffect::EmitBreakFinished(Some(finished)));
            if auto_close {
                effects.push(RuntimeEffect::HideBreak);
            } else {
                guard.runtime.current_status = AppStatus::BreakCompleted;
                guard.runtime.updated_at = utc_now();
                db.save_runtime(&guard.runtime)?;
            }
        } else {
            effects.push(RuntimeEffect::EmitBreakTick(guard.active_break.clone()));
        }
        return Ok(effects);
    }

    if let Some(paused_until) = guard
        .runtime
        .paused_until
        .as_ref()
        .and_then(|value| parse_utc(value))
    {
        if paused_until > Utc::now() {
            guard.runtime.current_status = AppStatus::Paused;
            guard.runtime.updated_at = utc_now();
            db.save_runtime(&guard.runtime)?;
            return Ok(effects);
        }

        guard.runtime.paused_until = None;
    }

    if matches!(guard.runtime.current_status, AppStatus::Paused)
        && guard.runtime.paused_until.is_none()
    {
        guard.runtime.next_reminder_due_at = None;
        guard.runtime.updated_at = utc_now();
        db.save_runtime(&guard.runtime)?;
        return Ok(effects);
    }

    if !crate::platform::within_schedule(&guard.settings) {
        guard.runtime.current_status = AppStatus::OutsideSchedule;
        guard.runtime.next_reminder_due_at = None;
        guard.runtime.updated_at = utc_now();
        db.save_runtime(&guard.runtime)?;
        return Ok(effects);
    }

    if matches!(guard.runtime.current_status, AppStatus::OutsideSchedule | AppStatus::Paused) {
        guard.runtime.current_status = AppStatus::Running;
    }

    if delta >= SLEEP_RESET_SECONDS {
        guard.runtime.active_elapsed_seconds = 0;
        guard.runtime.last_idle_detected_at = Some(utc_now());
    } else if input.idle_seconds >= IDLE_PAUSE_SECONDS {
        guard.runtime.last_idle_detected_at = Some(utc_now());
    } else if guard.active_reminder.is_none()
        && !matches!(guard.runtime.current_status, AppStatus::Snoozed | AppStatus::ReminderPending)
    {
        guard.runtime.active_elapsed_seconds += delta;
        guard.runtime.today_active_elapsed_seconds += delta;
        guard.runtime.today_max_active_streak_seconds = guard
            .runtime
            .today_max_active_streak_seconds
            .max(guard.runtime.active_elapsed_seconds);
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
    } else if guard.runtime.deferred_reminder_pending && !input.is_fullscreen {
        should_issue = true;
        trigger_reason = "fullscreen_release".into();
        fullscreen_delayed = true;
    } else if guard.active_reminder.is_none()
        && guard.runtime.active_elapsed_seconds >= guard.settings.reminder_interval_minutes * 60
    {
        if input.is_fullscreen && guard.settings.fullscreen_delay_enabled {
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
        reminder.id = db.insert_reminder_event(&reminder)?;
        guard.active_reminder = Some(reminder.clone());
        guard.runtime.pending_reminder_event_id = Some(reminder.id);
        guard.runtime.pending_reminder_level = Some(reminder.reminder_level);
        guard.runtime.deferred_reminder_pending = false;
        guard.runtime.current_status = AppStatus::ReminderPending;
        guard.runtime.next_reminder_due_at = None;
        effects.push(RuntimeEffect::PlayReminderSound);
        effects.push(RuntimeEffect::EmitReminder(Some(reminder.clone())));
        if reminder.reminder_level > 0 {
            effects.push(RuntimeEffect::ShowReminder(reminder.reminder_level));
        }
        if reminder.reminder_level == 1 {
            effects.push(RuntimeEffect::AutoHideReminder {
                reminder_id: reminder.id,
            });
        }
    } else if guard.active_reminder.is_none() {
        guard.runtime.next_reminder_due_at = commands::next_due_at(&guard.settings, &guard.runtime);
    }

    guard.runtime.updated_at = utc_now();
    db.save_runtime(&guard.runtime)?;
    Ok(effects)
}

pub fn start_break(
    db: &crate::db::Database,
    guard: &mut VolatileState,
    triggered_by_reminder_event_id: Option<i64>,
) -> Result<(BreakSession, Vec<RuntimeEffect>), String> {
    db.finish_all_running_break_sessions("interrupted", Some("new_break_started"))?;

    let reminder_id = triggered_by_reminder_event_id
        .or_else(|| guard.active_reminder.as_ref().map(|item| item.id));
    if let Some(id) = reminder_id {
        db.update_reminder_action(id, ReminderAction::StartedBreak, None)?;
    }

    guard.break_generation += 1;
    let now = utc_now();
    let mut session = BreakSession {
        id: 0,
        started_at: now.clone(),
        ended_at: None,
        duration_seconds: guard.settings.break_duration_seconds.max(1),
        status: "running".into(),
        cancel_reason: None,
        triggered_by_reminder_event_id: reminder_id,
        created_at: now.clone(),
        style: guard.settings.timer_style,
        remaining_seconds: guard.settings.break_duration_seconds.max(1),
    };
    session.id = db.insert_break_session(&session)?;

    commands::clear_pending_reminder(&mut guard.runtime);
    guard.active_reminder = None;
    guard.active_break = Some(session.clone());
    guard.runtime.current_status = AppStatus::BreakInProgress;
    guard.runtime.next_reminder_due_at = None;
    guard.runtime.updated_at = now;
    db.save_runtime(&guard.runtime)?;

    Ok((
        session.clone(),
        vec![
            RuntimeEffect::HideReminder,
            RuntimeEffect::ShowBreak,
            RuntimeEffect::EmitReminder(None),
            RuntimeEffect::EmitBreakTick(Some(session)),
        ],
    ))
}

pub fn cancel_break(
    db: &crate::db::Database,
    guard: &mut VolatileState,
) -> Result<Vec<RuntimeEffect>, String> {
    guard.break_generation += 1;
    let mut effects = vec![RuntimeEffect::HideBreak];
    let Some(mut session) = guard.active_break.take() else {
        reset_runtime_for_next_round(&guard.settings, &mut guard.runtime);
        db.save_runtime(&guard.runtime)?;
        effects.push(RuntimeEffect::EmitBreakFinished(None));
        return Ok(effects);
    };

    refresh_break_remaining(&mut session);
    let completed = session.remaining_seconds <= 0;
    let status = if completed { "completed" } else { "canceled" };
    let cancel_reason = if completed {
        None
    } else {
        Some("user_cancelled")
    };
    db.finish_break_session(session.id, status, cancel_reason)?;
    session.status = status.into();
    session.cancel_reason = cancel_reason.map(str::to_string);
    session.ended_at = Some(utc_now());
    session.remaining_seconds = if completed { 0 } else { session.remaining_seconds };
    reset_runtime_for_next_round(&guard.settings, &mut guard.runtime);
    db.save_runtime(&guard.runtime)?;
    effects.push(RuntimeEffect::EmitBreakFinished(Some(session)));
    Ok(effects)
}

pub fn snooze_reminder(
    db: &crate::db::Database,
    guard: &mut VolatileState,
) -> Result<Vec<RuntimeEffect>, String> {
    if let Some(reminder) = guard.active_reminder.as_ref() {
        db.update_reminder_action(reminder.id, ReminderAction::Snoozed, Some(5))?;
    }

    guard.active_reminder = None;
    guard.runtime.current_status = AppStatus::Snoozed;
    guard.runtime.next_reminder_due_at = Some((Utc::now() + Duration::minutes(5)).to_rfc3339());
    guard.runtime.updated_at = utc_now();
    commands::clear_pending_reminder(&mut guard.runtime);
    db.save_runtime(&guard.runtime)?;
    Ok(vec![
        RuntimeEffect::HideReminder,
        RuntimeEffect::EmitReminder(None),
    ])
}

pub fn skip_reminder(
    db: &crate::db::Database,
    guard: &mut VolatileState,
) -> Result<Vec<RuntimeEffect>, String> {
    if let Some(reminder) = guard.active_reminder.as_ref() {
        db.update_reminder_action(reminder.id, ReminderAction::Skipped, None)?;
    }

    guard.active_reminder = None;
    reset_runtime_for_next_round(&guard.settings, &mut guard.runtime);
    commands::clear_pending_reminder(&mut guard.runtime);
    db.save_runtime(&guard.runtime)?;
    Ok(vec![
        RuntimeEffect::HideReminder,
        RuntimeEffect::EmitReminder(None),
    ])
}

pub fn pause(
    db: &crate::db::Database,
    guard: &mut VolatileState,
    paused_until: String,
) -> Result<Vec<RuntimeEffect>, String> {
    if let Some(reminder) = guard.active_reminder.as_ref() {
        let _ = db.update_reminder_action(reminder.id, ReminderAction::Ignored, None);
    }

    guard.active_reminder = None;
    guard.runtime.current_status = AppStatus::Paused;
    guard.runtime.paused_until = Some(paused_until);
    guard.runtime.updated_at = utc_now();
    commands::clear_pending_reminder(&mut guard.runtime);
    db.save_runtime(&guard.runtime)?;
    Ok(vec![
        RuntimeEffect::HideReminder,
        RuntimeEffect::EmitReminder(None),
    ])
}

pub fn resume(db: &crate::db::Database, guard: &mut VolatileState) -> Result<(), String> {
    guard.runtime.current_status = if crate::platform::within_schedule(&guard.settings) {
        AppStatus::Running
    } else {
        AppStatus::OutsideSchedule
    };
    guard.runtime.paused_until = None;
    guard.runtime.active_elapsed_seconds = 0;
    guard.runtime.updated_at = utc_now();
    guard.runtime.next_reminder_due_at = commands::next_due_at(&guard.settings, &guard.runtime);
    db.save_runtime(&guard.runtime)
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
        commands::next_due_at(settings, runtime)
    } else {
        None
    };
    commands::clear_pending_reminder(runtime);
}

pub fn reset_runtime_for_process_start(settings: &Settings, runtime: &mut RuntimeState) {
    rollover_today(runtime);
    runtime.active_elapsed_seconds = 0;
    runtime.paused_until = None;
    runtime.last_fullscreen_detected_at = None;
    runtime.last_idle_detected_at = None;
    runtime.last_activity_at = None;
    runtime.current_status = if crate::platform::within_schedule(settings) {
        AppStatus::Running
    } else {
        AppStatus::OutsideSchedule
    };
    runtime.next_reminder_due_at = if matches!(runtime.current_status, AppStatus::Running) {
        commands::next_due_at(settings, runtime)
    } else {
        None
    };
    runtime.updated_at = utc_now();
    commands::clear_pending_reminder(runtime);
}

pub fn refresh_break_remaining(session: &mut BreakSession) {
    let Some(started_at) = parse_utc(&session.started_at) else {
        session.remaining_seconds = session.remaining_seconds.max(0);
        return;
    };
    let elapsed = (Utc::now() - started_at).num_seconds();
    session.remaining_seconds = (session.duration_seconds - elapsed).max(0);
}

pub fn rollover_today(runtime: &mut RuntimeState) {
    let today = local_date_key();
    if runtime.today_active_date == today {
        return;
    }
    runtime.today_active_date = today;
    runtime.today_active_elapsed_seconds = 0;
    runtime.today_max_active_streak_seconds = 0;
}

fn finish_active_break(
    db: &crate::db::Database,
    guard: &mut VolatileState,
    cancel_reason: Option<&str>,
) -> Result<BreakSession, String> {
    let mut session = guard
        .active_break
        .take()
        .ok_or_else(|| "no active break session".to_string())?;
    refresh_break_remaining(&mut session);
    let status = if cancel_reason.is_some() {
        "canceled"
    } else {
        "completed"
    };
    db.finish_break_session(session.id, status, cancel_reason)?;
    session.status = status.into();
    session.cancel_reason = cancel_reason.map(str::to_string);
    session.ended_at = Some(utc_now());
    session.remaining_seconds = 0;
    guard.break_generation += 1;
    reset_runtime_for_next_round(&guard.settings, &mut guard.runtime);
    db.save_runtime(&guard.runtime)?;
    Ok(session)
}

fn parse_utc(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|parsed| parsed.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{RuntimeState, Settings};

    #[test]
    fn rollover_resets_today_fields_on_date_change() {
        let mut runtime = RuntimeState::default();
        runtime.today_active_date = "2000-01-01".into();
        runtime.today_active_elapsed_seconds = 90;
        runtime.today_max_active_streak_seconds = 80;

        rollover_today(&mut runtime);

        assert_eq!(runtime.today_active_elapsed_seconds, 0);
        assert_eq!(runtime.today_max_active_streak_seconds, 0);
        assert_eq!(runtime.today_active_date, local_date_key());
    }

    #[test]
    fn reset_next_round_keeps_today_usage() {
        let settings = Settings::default();
        let mut runtime = RuntimeState::default();
        runtime.active_elapsed_seconds = 120;
        runtime.today_active_elapsed_seconds = 600;

        reset_runtime_for_next_round(&settings, &mut runtime);

        assert_eq!(runtime.active_elapsed_seconds, 0);
        assert_eq!(runtime.today_active_elapsed_seconds, 600);
    }

    #[test]
    fn process_start_resets_session_state_but_keeps_today_usage() {
        let settings = Settings::default();
        let mut runtime = RuntimeState::default();
        runtime.current_status = AppStatus::Snoozed;
        runtime.active_elapsed_seconds = 900;
        runtime.today_active_elapsed_seconds = 2_400;
        runtime.today_max_active_streak_seconds = 1_200;
        runtime.next_reminder_due_at = Some((Utc::now() + Duration::minutes(5)).to_rfc3339());
        runtime.paused_until = Some((Utc::now() + Duration::minutes(30)).to_rfc3339());
        runtime.deferred_reminder_pending = true;
        runtime.pending_reminder_event_id = Some(42);
        runtime.pending_reminder_level = Some(2);
        runtime.last_fullscreen_detected_at = Some(utc_now());
        runtime.last_idle_detected_at = Some(utc_now());
        runtime.last_activity_at = Some(utc_now());

        reset_runtime_for_process_start(&settings, &mut runtime);

        assert_eq!(runtime.current_status, AppStatus::Running);
        assert_eq!(runtime.active_elapsed_seconds, 0);
        assert_eq!(runtime.today_active_elapsed_seconds, 2_400);
        assert_eq!(runtime.today_max_active_streak_seconds, 1_200);
        assert!(runtime.next_reminder_due_at.is_some());
        assert!(runtime.paused_until.is_none());
        assert!(!runtime.deferred_reminder_pending);
        assert!(runtime.pending_reminder_event_id.is_none());
        assert!(runtime.pending_reminder_level.is_none());
        assert!(runtime.last_fullscreen_detected_at.is_none());
        assert!(runtime.last_idle_detected_at.is_none());
        assert!(runtime.last_activity_at.is_none());
    }

    #[test]
    fn process_start_rolls_over_today_usage_on_new_local_date() {
        let settings = Settings::default();
        let mut runtime = RuntimeState::default();
        runtime.today_active_date = "2000-01-01".into();
        runtime.active_elapsed_seconds = 900;
        runtime.today_active_elapsed_seconds = 2_400;
        runtime.today_max_active_streak_seconds = 1_200;

        reset_runtime_for_process_start(&settings, &mut runtime);

        assert_eq!(runtime.active_elapsed_seconds, 0);
        assert_eq!(runtime.today_active_elapsed_seconds, 0);
        assert_eq!(runtime.today_max_active_streak_seconds, 0);
        assert_eq!(runtime.today_active_date, local_date_key());
    }

    #[test]
    fn remaining_seconds_comes_from_started_at_and_duration() {
        let mut session = BreakSession {
            id: 1,
            started_at: (Utc::now() - Duration::seconds(5)).to_rfc3339(),
            ended_at: None,
            duration_seconds: 20,
            status: "running".into(),
            cancel_reason: None,
            triggered_by_reminder_event_id: None,
            created_at: utc_now(),
            style: crate::models::TimerStyle::Minimal,
            remaining_seconds: 20,
        };

        refresh_break_remaining(&mut session);

        assert!(session.remaining_seconds <= 15);
        assert!(session.remaining_seconds > 0);
    }
}
