use chrono::{DateTime, Datelike, Local, Timelike, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AppStatus {
    Running,
    Paused,
    Snoozed,
    BreakInProgress,
    OutsideSchedule,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TimerStyle {
    Minimal,
    Breathing,
    Guided,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PausePreset {
    #[serde(rename = "minutes_30")]
    Minutes30,
    #[serde(rename = "hour_1")]
    Hour1,
    Today,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReminderAction {
    StartedBreak,
    Skipped,
    Snoozed,
    Ignored,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub id: i64,
    pub language: String,
    pub reminder_interval_minutes: i64,
    pub break_duration_seconds: i64,
    pub auto_close_break_window: bool,
    pub reminder_level: i64,
    pub sound_enabled: bool,
    pub sound_type: Option<String>,
    pub low_distraction_mode: bool,
    pub fullscreen_delay_enabled: bool,
    pub launch_at_startup: bool,
    pub close_button_behavior: String,
    pub work_schedule_enabled: bool,
    pub active_days: Vec<u32>,
    pub work_time_start: Option<String>,
    pub work_time_end: Option<String>,
    pub timer_style: TimerStyle,
    pub status_icon_mode: String,
    pub window_opacity: f64,
    pub display_width_cm: Option<f64>,
    pub display_height_cm: Option<f64>,
    pub recommended_viewing_distance_cm: Option<f64>,
    pub has_completed_onboarding: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeState {
    pub id: i64,
    pub current_status: AppStatus,
    pub active_elapsed_seconds: i64,
    pub next_reminder_due_at: Option<String>,
    pub paused_until: Option<String>,
    pub deferred_reminder_pending: bool,
    pub pending_reminder_event_id: Option<i64>,
    pub pending_reminder_level: Option<i64>,
    pub last_fullscreen_detected_at: Option<String>,
    pub last_idle_detected_at: Option<String>,
    pub last_activity_at: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderEvent {
    pub id: i64,
    pub triggered_at: String,
    pub trigger_reason: String,
    pub reminder_level: i64,
    pub was_fullscreen_delayed: bool,
    pub delivery_type: String,
    pub user_action: Option<ReminderAction>,
    pub action_at: Option<String>,
    pub deferred_minutes: Option<i64>,
    pub active_elapsed_seconds: i64,
    pub created_at: String,
    pub display_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BreakSession {
    pub id: i64,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_seconds: i64,
    pub status: String,
    pub cancel_reason: Option<String>,
    pub triggered_by_reminder_event_id: Option<i64>,
    pub created_at: String,
    pub style: TimerStyle,
    pub remaining_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodaySummary {
    pub completed_break_count: i64,
    pub skipped_count: i64,
    pub snoozed_count: i64,
    pub max_active_streak_seconds: i64,
    pub completion_rate: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapPayload {
    pub settings: Settings,
    pub runtime_state: RuntimeState,
    pub today_summary: TodaySummary,
    pub active_reminder: Option<ReminderEvent>,
    pub active_break: Option<BreakSession>,
    pub now_iso: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetectedDisplaySize {
    pub display_width_cm: f64,
    pub display_height_cm: f64,
}

impl Settings {
    pub fn default() -> Self {
        let now = utc_now();
        let display_width_cm = Some(55.0);
        let display_height_cm = Some(31.0);
        Self {
            id: 1,
            language: "zh-CN".into(),
            reminder_interval_minutes: 20,
            break_duration_seconds: 20,
            auto_close_break_window: true,
            reminder_level: 1,
            sound_enabled: false,
            sound_type: None,
            low_distraction_mode: true,
            fullscreen_delay_enabled: true,
            launch_at_startup: true,
            close_button_behavior: default_close_button_behavior().into(),
            work_schedule_enabled: false,
            active_days: vec![1, 2, 3, 4, 5],
            work_time_start: Some("09:00".into()),
            work_time_end: Some("18:00".into()),
            timer_style: TimerStyle::Minimal,
            status_icon_mode: "adaptive".into(),
            window_opacity: 0.3,
            display_width_cm,
            display_height_cm,
            recommended_viewing_distance_cm: display_width_cm
                .zip(display_height_cm)
                .and_then(|(width, height)| recommended_viewing_distance(width, height)),
            has_completed_onboarding: false,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

pub fn default_close_button_behavior() -> &'static str {
    "hide_main_window"
}

pub fn normalize_close_button_behavior(value: &str) -> &'static str {
    match value {
        "quit_app" => "quit_app",
        _ => default_close_button_behavior(),
    }
}

impl RuntimeState {
    pub fn default() -> Self {
        Self {
            id: 1,
            current_status: AppStatus::Running,
            active_elapsed_seconds: 0,
            next_reminder_due_at: None,
            paused_until: None,
            deferred_reminder_pending: false,
            pending_reminder_event_id: None,
            pending_reminder_level: None,
            last_fullscreen_detected_at: None,
            last_idle_detected_at: None,
            last_activity_at: None,
            updated_at: utc_now(),
        }
    }
}

pub fn utc_now() -> String {
    Utc::now().to_rfc3339()
}

pub fn end_of_today_utc() -> DateTime<Utc> {
    let local = Local::now()
        .with_hour(23)
        .and_then(|time| time.with_minute(59))
        .and_then(|time| time.with_second(59))
        .unwrap_or_else(Local::now);
    local.with_timezone(&Utc)
}

pub fn recommended_viewing_distance(width_cm: f64, height_cm: f64) -> Option<f64> {
    if width_cm <= 0.0 || height_cm <= 0.0 {
        return None;
    }

    Some(width_cm.hypot(height_cm))
}

pub fn is_active_day(days: &[u32]) -> bool {
    let day = Local::now().weekday().number_from_monday();
    days.contains(&day)
}
