use std::sync::Mutex;

use crate::db::Database;
use crate::models::{BootstrapPayload, BreakSession, ReminderEvent, RuntimeState, Settings};

pub struct VolatileState {
    pub settings: Settings,
    pub runtime: RuntimeState,
    pub active_reminder: Option<ReminderEvent>,
    pub active_break: Option<BreakSession>,
    pub exit_requested: bool,
    pub break_generation: u64,
    pub last_tick_unix: i64,
}

pub struct AppContext {
    pub db: Database,
    pub volatile: Mutex<VolatileState>,
}

impl AppContext {
    pub fn new(
        db: Database,
        settings: Settings,
        runtime: RuntimeState,
        active_reminder: Option<ReminderEvent>,
        active_break: Option<BreakSession>,
    ) -> Self {
        Self {
            db,
            volatile: Mutex::new(VolatileState {
                settings,
                runtime,
                active_reminder,
                active_break,
                exit_requested: false,
                break_generation: 0,
                last_tick_unix: chrono::Utc::now().timestamp(),
            }),
        }
    }

    pub fn snapshot(&self) -> Result<BootstrapPayload, String> {
        let mut guard = self
            .volatile
            .lock()
            .map_err(|_| "state lock poisoned".to_string())?;
        crate::runtime_service::reconcile_snapshot(&self.db, &mut guard)?;

        Ok(BootstrapPayload {
            settings: guard.settings.clone(),
            runtime_state: guard.runtime.clone(),
            today_summary: self.db.get_today_summary()?,
            active_reminder: guard.active_reminder.clone(),
            active_break: guard.active_break.clone(),
            now_iso: crate::models::utc_now(),
        })
    }
}
