use tauri::{image::Image, AppHandle, Manager};

use crate::{
    models::{AppStatus, BreakSession, ReminderEvent, RuntimeState, Settings},
    windows::TRAY_ID,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayVisualState {
    Running,
    Reminder,
    Break,
    Paused,
    Snoozed,
    OutsideSchedule,
}

pub fn sync_tray(app: &AppHandle) {
    let Ok(payload) = app.state::<crate::state::AppContext>().snapshot() else {
        return;
    };
    let visual = derive_visual_state(
        &payload.settings,
        &payload.runtime_state,
        payload.active_reminder.as_ref(),
        payload.active_break.as_ref(),
    );
    let tooltip = tooltip_for_state(visual, payload.active_break.as_ref());

    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_tooltip(Some(tooltip));
        if payload.settings.status_icon_mode == "adaptive" {
            let _ = tray.set_icon(Some(icon_for_state(visual)));
        }
    }
}

pub fn derive_visual_state(
    _settings: &Settings,
    runtime: &RuntimeState,
    active_reminder: Option<&ReminderEvent>,
    active_break: Option<&BreakSession>,
) -> TrayVisualState {
    if active_break.is_some() || matches!(runtime.current_status, AppStatus::BreakInProgress) {
        return TrayVisualState::Break;
    }
    if active_reminder.is_some() || matches!(runtime.current_status, AppStatus::ReminderPending) {
        return TrayVisualState::Reminder;
    }
    match runtime.current_status {
        AppStatus::Paused => TrayVisualState::Paused,
        AppStatus::Snoozed => TrayVisualState::Snoozed,
        AppStatus::OutsideSchedule => TrayVisualState::OutsideSchedule,
        _ => TrayVisualState::Running,
    }
}

fn tooltip_for_state(visual: TrayVisualState, active_break: Option<&BreakSession>) -> String {
    match visual {
        TrayVisualState::Running => "GazeRest - running".into(),
        TrayVisualState::Reminder => "GazeRest - break reminder ready".into(),
        TrayVisualState::Break => {
            let seconds = active_break
                .map(|session| session.remaining_seconds.max(0))
                .unwrap_or(0);
            format!("GazeRest - on break ({seconds}s)")
        }
        TrayVisualState::Paused => "GazeRest - paused".into(),
        TrayVisualState::Snoozed => "GazeRest - snoozed".into(),
        TrayVisualState::OutsideSchedule => "GazeRest - outside schedule".into(),
    }
}

fn icon_for_state(visual: TrayVisualState) -> Image<'static> {
    let color = match visual {
        TrayVisualState::Running => [47, 153, 137, 255],
        TrayVisualState::Reminder => [224, 92, 76, 255],
        TrayVisualState::Break => [52, 117, 150, 255],
        TrayVisualState::Paused => [190, 125, 46, 255],
        TrayVisualState::Snoozed => [76, 115, 185, 255],
        TrayVisualState::OutsideSchedule => [90, 105, 120, 255],
    };

    let mut rgba = vec![0_u8; 32 * 32 * 4];
    for y in 0..32 {
        for x in 0..32 {
            let dx = x as f32 - 15.5;
            let dy = y as f32 - 15.5;
            let distance = (dx * dx + dy * dy).sqrt();
            let index = (y * 32 + x) * 4;
            if distance <= 13.5 {
                rgba[index..index + 4].copy_from_slice(&color);
            } else if distance <= 15.0 {
                rgba[index..index + 4].copy_from_slice(&[255, 255, 255, 180]);
            }
        }
    }

    match visual {
        TrayVisualState::Reminder => draw_bar(&mut rgba, 10, 8, 12, 16, [255, 255, 255, 255]),
        TrayVisualState::Break => draw_bar(&mut rgba, 9, 9, 5, 14, [255, 255, 255, 255]),
        TrayVisualState::Paused => {
            draw_bar(&mut rgba, 10, 9, 4, 14, [255, 255, 255, 255]);
            draw_bar(&mut rgba, 18, 9, 4, 14, [255, 255, 255, 255]);
        }
        TrayVisualState::Snoozed => draw_bar(&mut rgba, 8, 14, 16, 4, [255, 255, 255, 255]),
        TrayVisualState::OutsideSchedule => draw_bar(&mut rgba, 9, 14, 14, 4, [255, 255, 255, 255]),
        TrayVisualState::Running => draw_check(&mut rgba),
    }

    Image::new_owned(rgba, 32, 32)
}

fn draw_bar(rgba: &mut [u8], x: usize, y: usize, width: usize, height: usize, color: [u8; 4]) {
    for yy in y..(y + height) {
        for xx in x..(x + width) {
            let index = (yy * 32 + xx) * 4;
            rgba[index..index + 4].copy_from_slice(&color);
        }
    }
}

fn draw_check(rgba: &mut [u8]) {
    for offset in 0..4 {
        draw_bar(rgba, 9 + offset, 17 + offset, 3, 3, [255, 255, 255, 255]);
        draw_bar(rgba, 13 + offset, 20 - offset, 3, 3, [255, 255, 255, 255]);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{RuntimeState, Settings};

    #[test]
    fn active_break_wins_tray_state() {
        let settings = Settings::default();
        let runtime = RuntimeState::default();
        let break_session = BreakSession {
            id: 1,
            started_at: crate::models::utc_now(),
            ended_at: None,
            duration_seconds: 20,
            status: "running".into(),
            cancel_reason: None,
            triggered_by_reminder_event_id: None,
            created_at: crate::models::utc_now(),
            style: crate::models::TimerStyle::Minimal,
            remaining_seconds: 10,
        };

        assert_eq!(
            derive_visual_state(&settings, &runtime, None, Some(&break_session)),
            TrayVisualState::Break
        );
    }

    #[test]
    fn paused_maps_to_paused() {
        let settings = Settings::default();
        let mut runtime = RuntimeState::default();
        runtime.current_status = AppStatus::Paused;

        assert_eq!(
            derive_visual_state(&settings, &runtime, None, None),
            TrayVisualState::Paused
        );
    }
}
