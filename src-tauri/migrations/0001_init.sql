CREATE TABLE IF NOT EXISTS settings (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  language TEXT NOT NULL,
  reminder_interval_minutes INTEGER NOT NULL,
  break_duration_seconds INTEGER NOT NULL,
  reminder_level INTEGER NOT NULL,
  sound_enabled INTEGER NOT NULL DEFAULT 0,
  sound_type TEXT NULL,
  low_distraction_mode INTEGER NOT NULL DEFAULT 1,
  fullscreen_delay_enabled INTEGER NOT NULL DEFAULT 1,
  launch_at_startup INTEGER NOT NULL DEFAULT 1,
  close_button_behavior TEXT NOT NULL DEFAULT 'hide_main_window',
  work_schedule_enabled INTEGER NOT NULL DEFAULT 0,
  active_days_json TEXT NOT NULL,
  work_time_start TEXT NULL,
  work_time_end TEXT NULL,
  timer_style TEXT NOT NULL,
  status_icon_mode TEXT NOT NULL,
  window_opacity REAL NOT NULL DEFAULT 0.3,
  display_width_cm REAL NULL,
  display_height_cm REAL NULL,
  recommended_viewing_distance_cm REAL NULL,
  has_completed_onboarding INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS runtime_state (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  current_status TEXT NOT NULL,
  active_elapsed_seconds INTEGER NOT NULL DEFAULT 0,
  next_reminder_due_at TEXT NULL,
  paused_until TEXT NULL,
  deferred_reminder_pending INTEGER NOT NULL DEFAULT 0,
  pending_reminder_event_id INTEGER NULL,
  pending_reminder_level INTEGER NULL,
  last_fullscreen_detected_at TEXT NULL,
  last_idle_detected_at TEXT NULL,
  last_activity_at TEXT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS reminder_events (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  triggered_at TEXT NOT NULL,
  trigger_reason TEXT NOT NULL,
  reminder_level INTEGER NOT NULL,
  was_fullscreen_delayed INTEGER NOT NULL DEFAULT 0,
  delivery_type TEXT NOT NULL,
  user_action TEXT NULL,
  action_at TEXT NULL,
  deferred_minutes INTEGER NULL,
  active_elapsed_seconds INTEGER NOT NULL DEFAULT 0,
  created_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_reminder_events_triggered_at
  ON reminder_events (triggered_at);

CREATE INDEX IF NOT EXISTS idx_reminder_events_user_action
  ON reminder_events (user_action);

CREATE TABLE IF NOT EXISTS break_sessions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  started_at TEXT NOT NULL,
  ended_at TEXT NULL,
  duration_seconds INTEGER NOT NULL,
  status TEXT NOT NULL,
  cancel_reason TEXT NULL,
  triggered_by_reminder_event_id INTEGER NULL,
  timer_style TEXT NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY (triggered_by_reminder_event_id) REFERENCES reminder_events(id)
);

CREATE INDEX IF NOT EXISTS idx_break_sessions_started_at
  ON break_sessions (started_at);

CREATE INDEX IF NOT EXISTS idx_break_sessions_status
  ON break_sessions (status);
