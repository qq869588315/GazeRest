use std::{fs, path::PathBuf};

use chrono::{Duration, Local, TimeZone, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use tauri::Manager;

use crate::models::{
    normalize_close_button_behavior, utc_now, AppStatus, BreakSession, ReminderAction,
    ReminderEvent, RuntimeState, Settings, TimerStyle, TodaySummary,
};

pub struct Database {
    path: PathBuf,
}

impl Database {
    pub fn new(app: &tauri::AppHandle) -> Result<Self, String> {
        let base_dir = app
            .path()
            .app_data_dir()
            .map_err(|error| error.to_string())?;
        fs::create_dir_all(&base_dir).map_err(|error| error.to_string())?;
        let database = Self {
            path: base_dir.join("gazerest.sqlite"),
        };
        database.migrate()?;
        Ok(database)
    }

    fn connect(&self) -> Result<Connection, String> {
        let connection = Connection::open(&self.path).map_err(|error| error.to_string())?;
        connection
            .execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|error| error.to_string())?;
        Ok(connection)
    }

    fn migrate(&self) -> Result<(), String> {
        let connection = self.connect()?;
        connection
            .execute_batch(include_str!("../migrations/0001_init.sql"))
            .map_err(|error| error.to_string())?;
        ensure_settings_column(
            &connection,
            "low_distraction_mode",
            "INTEGER NOT NULL DEFAULT 1",
        )?;
        ensure_settings_column(
            &connection,
            "close_button_behavior",
            "TEXT NOT NULL DEFAULT 'hide_main_window'",
        )?;
        ensure_settings_column(&connection, "window_opacity", "REAL NOT NULL DEFAULT 0.3")?;
        ensure_settings_column(&connection, "display_width_cm", "REAL NULL")?;
        ensure_settings_column(&connection, "display_height_cm", "REAL NULL")?;
        ensure_settings_column(&connection, "recommended_viewing_distance_cm", "REAL NULL")?;
        Ok(())
    }

    pub fn load_or_init_settings(&self) -> Result<Settings, String> {
        let connection = self.connect()?;
        let maybe_row = connection
      .query_row(
        "SELECT id, language, reminder_interval_minutes, break_duration_seconds, reminder_level,
                sound_enabled, sound_type, low_distraction_mode, fullscreen_delay_enabled, launch_at_startup,
                close_button_behavior, work_schedule_enabled, active_days_json, work_time_start, work_time_end,
                timer_style, status_icon_mode, window_opacity, display_width_cm, display_height_cm,
                recommended_viewing_distance_cm, has_completed_onboarding, created_at, updated_at
         FROM settings WHERE id = 1",
        [],
        map_settings,
      )
      .optional()
      .map_err(|error| error.to_string())?;

        if let Some(settings) = maybe_row {
            return Ok(settings);
        }

        let settings = Settings::default();
        self.save_settings(&settings)?;
        Ok(settings)
    }

    pub fn save_settings(&self, settings: &Settings) -> Result<(), String> {
        let connection = self.connect()?;
        connection
            .execute(
                "INSERT OR REPLACE INTO settings (
          id, language, reminder_interval_minutes, break_duration_seconds, reminder_level,
          sound_enabled, sound_type, low_distraction_mode, fullscreen_delay_enabled, launch_at_startup,
          close_button_behavior, work_schedule_enabled, active_days_json, work_time_start, work_time_end,
          timer_style, status_icon_mode, window_opacity, display_width_cm, display_height_cm,
          recommended_viewing_distance_cm, has_completed_onboarding, created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24)",
                params![
                    settings.id,
                    settings.language,
                    settings.reminder_interval_minutes,
                    settings.break_duration_seconds,
                    settings.reminder_level,
                    bool_to_i64(settings.sound_enabled),
                    settings.sound_type,
                    bool_to_i64(settings.low_distraction_mode),
                    bool_to_i64(settings.fullscreen_delay_enabled),
                    bool_to_i64(settings.launch_at_startup),
                    normalize_close_button_behavior(&settings.close_button_behavior),
                    bool_to_i64(settings.work_schedule_enabled),
                    serde_json::to_string(&settings.active_days)
                        .map_err(|error| error.to_string())?,
                    settings.work_time_start,
                    settings.work_time_end,
                    timer_style_to_str(settings.timer_style),
                    settings.status_icon_mode,
                    settings.window_opacity,
                    settings.display_width_cm,
                    settings.display_height_cm,
                    settings.recommended_viewing_distance_cm,
                    bool_to_i64(settings.has_completed_onboarding),
                    settings.created_at,
                    settings.updated_at,
                ],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn load_or_init_runtime(&self) -> Result<RuntimeState, String> {
        let connection = self.connect()?;
        let maybe_row = connection
            .query_row(
                "SELECT id, current_status, active_elapsed_seconds, next_reminder_due_at,
                paused_until, deferred_reminder_pending, pending_reminder_event_id,
                pending_reminder_level, last_fullscreen_detected_at, last_idle_detected_at,
                last_activity_at, updated_at
         FROM runtime_state WHERE id = 1",
                [],
                map_runtime,
            )
            .optional()
            .map_err(|error| error.to_string())?;

        if let Some(runtime) = maybe_row {
            return Ok(runtime);
        }

        let runtime = RuntimeState::default();
        self.save_runtime(&runtime)?;
        Ok(runtime)
    }

    pub fn save_runtime(&self, runtime: &RuntimeState) -> Result<(), String> {
        let connection = self.connect()?;
        connection
            .execute(
                "INSERT OR REPLACE INTO runtime_state (
          id, current_status, active_elapsed_seconds, next_reminder_due_at, paused_until,
          deferred_reminder_pending, pending_reminder_event_id, pending_reminder_level,
          last_fullscreen_detected_at, last_idle_detected_at, last_activity_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
                params![
                    runtime.id,
                    app_status_to_str(&runtime.current_status),
                    runtime.active_elapsed_seconds,
                    runtime.next_reminder_due_at,
                    runtime.paused_until,
                    bool_to_i64(runtime.deferred_reminder_pending),
                    runtime.pending_reminder_event_id,
                    runtime.pending_reminder_level,
                    runtime.last_fullscreen_detected_at,
                    runtime.last_idle_detected_at,
                    runtime.last_activity_at,
                    runtime.updated_at,
                ],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn insert_reminder_event(&self, reminder: &ReminderEvent) -> Result<i64, String> {
        let connection = self.connect()?;
        connection
            .execute(
                "INSERT INTO reminder_events (
          triggered_at, trigger_reason, reminder_level, was_fullscreen_delayed, delivery_type,
          user_action, action_at, deferred_minutes, active_elapsed_seconds, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                params![
                    reminder.triggered_at,
                    reminder.trigger_reason,
                    reminder.reminder_level,
                    bool_to_i64(reminder.was_fullscreen_delayed),
                    reminder.delivery_type,
                    reminder.user_action.as_ref().map(reminder_action_to_str),
                    reminder.action_at,
                    reminder.deferred_minutes,
                    reminder.active_elapsed_seconds,
                    reminder.created_at,
                ],
            )
            .map_err(|error| error.to_string())?;
        Ok(connection.last_insert_rowid())
    }

    pub fn update_reminder_action(
        &self,
        reminder_id: i64,
        action: ReminderAction,
        deferred_minutes: Option<i64>,
    ) -> Result<(), String> {
        let connection = self.connect()?;
        connection
            .execute(
                "UPDATE reminder_events
         SET user_action = ?1, action_at = ?2, deferred_minutes = ?3
         WHERE id = ?4",
                params![
                    reminder_action_to_str(&action),
                    utc_now(),
                    deferred_minutes,
                    reminder_id,
                ],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn insert_break_session(&self, session: &BreakSession) -> Result<i64, String> {
        let connection = self.connect()?;
        connection
            .execute(
                "INSERT INTO break_sessions (
          started_at, ended_at, duration_seconds, status, cancel_reason,
          triggered_by_reminder_event_id, timer_style, created_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    session.started_at,
                    session.ended_at,
                    session.duration_seconds,
                    session.status,
                    session.cancel_reason,
                    session.triggered_by_reminder_event_id,
                    timer_style_to_str(session.style),
                    session.created_at,
                ],
            )
            .map_err(|error| error.to_string())?;
        Ok(connection.last_insert_rowid())
    }

    pub fn finish_break_session(
        &self,
        session_id: i64,
        status: &str,
        cancel_reason: Option<&str>,
    ) -> Result<(), String> {
        let connection = self.connect()?;
        connection
            .execute(
                "UPDATE break_sessions
         SET ended_at = ?1, status = ?2, cancel_reason = ?3
         WHERE id = ?4",
                params![utc_now(), status, cancel_reason, session_id],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn get_today_summary(&self) -> Result<TodaySummary, String> {
        let connection = self.connect()?;
        let local_now = Local::now();
        let start_local = local_now
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .ok_or_else(|| "无法计算当天开始时间".to_string())?;
        let start_utc = Local
            .from_local_datetime(&start_local)
            .single()
            .ok_or_else(|| "无法转换本地开始时间".to_string())?
            .with_timezone(&Utc)
            .to_rfc3339();
        let end_utc = (Utc::now() + Duration::days(1)).to_rfc3339();

        let completed_break_count = connection
      .query_row(
        "SELECT COUNT(*) FROM break_sessions WHERE status = 'completed' AND started_at >= ?1 AND started_at < ?2",
        params![start_utc, end_utc],
        |row| row.get::<_, i64>(0),
      )
      .map_err(|error| error.to_string())?;

        let skipped_count = count_by_action(&connection, &start_utc, &end_utc, "skipped")?;
        let snoozed_count = count_by_action(&connection, &start_utc, &end_utc, "snoozed")?;
        let max_active_streak_seconds = connection
      .query_row(
        "SELECT COALESCE(MAX(active_elapsed_seconds), 0) FROM reminder_events WHERE triggered_at >= ?1 AND triggered_at < ?2",
        params![start_utc, end_utc],
        |row| row.get::<_, i64>(0),
      )
      .map_err(|error| error.to_string())?;
        let total_prompts = completed_break_count + skipped_count + snoozed_count;
        let completion_rate = if total_prompts == 0 {
            0
        } else {
            ((completed_break_count as f64 / total_prompts as f64) * 100.0).round() as i64
        };

        Ok(TodaySummary {
            completed_break_count,
            skipped_count,
            snoozed_count,
            max_active_streak_seconds,
            completion_rate,
        })
    }
}

fn count_by_action(
    connection: &Connection,
    start_utc: &str,
    end_utc: &str,
    action: &str,
) -> Result<i64, String> {
    connection
    .query_row(
      "SELECT COUNT(*) FROM reminder_events WHERE user_action = ?1 AND triggered_at >= ?2 AND triggered_at < ?3",
      params![action, start_utc, end_utc],
      |row| row.get::<_, i64>(0),
    )
    .map_err(|error| error.to_string())
}

fn map_settings(row: &rusqlite::Row<'_>) -> rusqlite::Result<Settings> {
    Ok(Settings {
        id: row.get(0)?,
        language: row.get(1)?,
        reminder_interval_minutes: row.get(2)?,
        break_duration_seconds: row.get(3)?,
        reminder_level: row.get(4)?,
        sound_enabled: row.get::<_, i64>(5)? == 1,
        sound_type: row.get(6)?,
        low_distraction_mode: row.get::<_, i64>(7)? == 1,
        fullscreen_delay_enabled: row.get::<_, i64>(8)? == 1,
        launch_at_startup: row.get::<_, i64>(9)? == 1,
        close_button_behavior: normalize_close_button_behavior(&row.get::<_, String>(10)?).into(),
        work_schedule_enabled: row.get::<_, i64>(11)? == 1,
        active_days: serde_json::from_str::<Vec<u32>>(&row.get::<_, String>(12)?)
            .unwrap_or_else(|_| vec![1, 2, 3, 4, 5]),
        work_time_start: row.get(13)?,
        work_time_end: row.get(14)?,
        timer_style: timer_style_from_str(&row.get::<_, String>(15)?),
        status_icon_mode: row.get(16)?,
        window_opacity: row.get(17)?,
        display_width_cm: row.get(18)?,
        display_height_cm: row.get(19)?,
        recommended_viewing_distance_cm: row.get(20)?,
        has_completed_onboarding: row.get::<_, i64>(21)? == 1,
        created_at: row.get(22)?,
        updated_at: row.get(23)?,
    })
}

fn ensure_settings_column(
    connection: &Connection,
    column_name: &str,
    definition: &str,
) -> Result<(), String> {
    let mut statement = connection
        .prepare("PRAGMA table_info(settings)")
        .map_err(|error| error.to_string())?;
    let columns = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|error| error.to_string())?;

    for column in columns {
        if column.map_err(|error| error.to_string())? == column_name {
            return Ok(());
        }
    }

    connection
        .execute_batch(&format!(
            "ALTER TABLE settings ADD COLUMN {column_name} {definition};"
        ))
        .map_err(|error| error.to_string())
}

fn map_runtime(row: &rusqlite::Row<'_>) -> rusqlite::Result<RuntimeState> {
    Ok(RuntimeState {
        id: row.get(0)?,
        current_status: app_status_from_str(&row.get::<_, String>(1)?),
        active_elapsed_seconds: row.get(2)?,
        next_reminder_due_at: row.get(3)?,
        paused_until: row.get(4)?,
        deferred_reminder_pending: row.get::<_, i64>(5)? == 1,
        pending_reminder_event_id: row.get(6)?,
        pending_reminder_level: row.get(7)?,
        last_fullscreen_detected_at: row.get(8)?,
        last_idle_detected_at: row.get(9)?,
        last_activity_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

pub fn bool_to_i64(value: bool) -> i64 {
    if value {
        1
    } else {
        0
    }
}

pub fn app_status_to_str(status: &AppStatus) -> &'static str {
    match status {
        AppStatus::Running => "running",
        AppStatus::Paused => "paused",
        AppStatus::Snoozed => "snoozed",
        AppStatus::BreakInProgress => "break_in_progress",
        AppStatus::OutsideSchedule => "outside_schedule",
    }
}

pub fn app_status_from_str(value: &str) -> AppStatus {
    match value {
        "paused" => AppStatus::Paused,
        "snoozed" => AppStatus::Snoozed,
        "break_in_progress" => AppStatus::BreakInProgress,
        "outside_schedule" => AppStatus::OutsideSchedule,
        _ => AppStatus::Running,
    }
}

pub fn timer_style_to_str(style: TimerStyle) -> &'static str {
    match style {
        TimerStyle::Minimal => "minimal",
        TimerStyle::Breathing => "breathing",
        TimerStyle::Guided => "guided",
    }
}

pub fn timer_style_from_str(value: &str) -> TimerStyle {
    match value {
        "breathing" => TimerStyle::Breathing,
        "guided" => TimerStyle::Guided,
        _ => TimerStyle::Minimal,
    }
}

pub fn reminder_action_to_str(action: &ReminderAction) -> &'static str {
    match action {
        ReminderAction::StartedBreak => "started_break",
        ReminderAction::Skipped => "skipped",
        ReminderAction::Snoozed => "snoozed",
        ReminderAction::Ignored => "ignored",
    }
}
