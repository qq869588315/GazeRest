use chrono::{Local, Timelike};
use tauri::Manager;

use crate::models::{DetectedDisplaySize, Settings};

pub fn idle_seconds() -> u64 {
    system_idle_time::get_idle_time()
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

pub fn is_fullscreen(app: &tauri::AppHandle) -> bool {
    let Some(window) = app.get_webview_window(crate::windows::MAIN_WINDOW_LABEL) else {
        return false;
    };

    let Ok(monitors) = window.available_monitors() else {
        return false;
    };
    let Ok(active_window) = active_win_pos_rs::get_active_window() else {
        return false;
    };

    let left = active_window.position.x as i32;
    let top = active_window.position.y as i32;
    let width = active_window.position.width as i32;
    let height = active_window.position.height as i32;

    monitors.into_iter().any(|monitor| {
        let monitor_position = monitor.position();
        let monitor_size = monitor.size();
        let close_to_edges =
            (left - monitor_position.x).abs() <= 8 && (top - monitor_position.y).abs() <= 8;
        let covers_monitor = width >= (monitor_size.width as f32 * 0.96) as i32
            && height >= (monitor_size.height as f32 * 0.96) as i32;
        close_to_edges && covers_monitor
    })
}

pub fn within_schedule(settings: &Settings) -> bool {
    if !settings.work_schedule_enabled {
        return true;
    }

    if !crate::models::is_active_day(&settings.active_days) {
        return false;
    }

    let now = Local::now();
    let Some(start) = settings.work_time_start.as_ref() else {
        return true;
    };
    let Some(end) = settings.work_time_end.as_ref() else {
        return true;
    };

    let Ok(start_minutes) = parse_minutes(start) else {
        return true;
    };
    let Ok(end_minutes) = parse_minutes(end) else {
        return true;
    };
    let now_minutes = now.hour() * 60 + now.minute();

    now_minutes >= start_minutes && now_minutes <= end_minutes
}

fn parse_minutes(value: &str) -> Result<u32, ()> {
    let mut parts = value.split(':');
    let Some(hours) = parts.next() else {
        return Err(());
    };
    let Some(minutes) = parts.next() else {
        return Err(());
    };
    let hours = hours.parse::<u32>().map_err(|_| ())?;
    let minutes = minutes.parse::<u32>().map_err(|_| ())?;
    Ok(hours * 60 + minutes)
}

pub fn detect_display_size() -> Result<DetectedDisplaySize, String> {
    #[cfg(target_os = "windows")]
    {
        return detect_display_size_windows();
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Automatic monitor size detection is currently only available on Windows.".into())
    }
}

#[cfg(target_os = "windows")]
fn detect_display_size_windows() -> Result<DetectedDisplaySize, String> {
    use std::process::Command;

    #[derive(serde::Deserialize)]
    struct MonitorSizePayload {
        #[serde(rename = "MaxHorizontalImageSize")]
        max_horizontal_image_size: f64,
        #[serde(rename = "MaxVerticalImageSize")]
        max_vertical_image_size: f64,
    }

    let script = "$monitor = Get-CimInstance -Namespace root\\wmi -ClassName WmiMonitorBasicDisplayParams | Where-Object { $_.Active -eq $true -and $_.MaxHorizontalImageSize -gt 0 -and $_.MaxVerticalImageSize -gt 0 } | Select-Object -First 1 -Property MaxHorizontalImageSize, MaxVerticalImageSize; if ($null -eq $monitor) { '' } else { $monitor | ConvertTo-Json -Compress }";
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .output()
        .map_err(|error| format!("Failed to read monitor size: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            "Could not detect the monitor size automatically. Please enter it manually.".into()
        } else {
            stderr
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if stdout.is_empty() || stdout == "null" {
        return Err("Could not detect the monitor size automatically. Please enter it manually.".into());
    }

    let payload: MonitorSizePayload = serde_json::from_str(&stdout)
        .map_err(|error| format!("Failed to parse monitor size: {error}"))?;

    if payload.max_horizontal_image_size <= 0.0 || payload.max_vertical_image_size <= 0.0 {
        return Err("Could not detect the monitor size automatically. Please enter it manually.".into());
    }

    Ok(DetectedDisplaySize {
        display_width_cm: round_one_decimal(payload.max_horizontal_image_size),
        display_height_cm: round_one_decimal(payload.max_vertical_image_size),
    })
}

#[cfg(target_os = "windows")]
fn round_one_decimal(value: f64) -> f64 {
    (value * 10.0).round() / 10.0
}
