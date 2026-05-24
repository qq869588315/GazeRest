use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use tauri::{AppHandle, Manager};

use crate::state::AppContext;

pub fn play_reminder_sound(app: &AppHandle) {
    let enabled = app
        .state::<AppContext>()
        .volatile
        .lock()
        .map(|guard| guard.settings.sound_enabled)
        .unwrap_or(false);

    if !enabled {
        return;
    }

    #[cfg(target_os = "windows")]
    {
        std::thread::spawn(|| {
            let script = "[console]::beep(880,160); Start-Sleep -Milliseconds 60; [console]::beep(988,180)";
            let mut command = Command::new("powershell");
            command
                .creation_flags(0x08000000)
                .args(["-NoProfile", "-WindowStyle", "Hidden", "-Command", script]);

            if let Err(error) = command.spawn() {
                log::warn!("failed to play reminder sound: {error}");
            }
        });
    }

    #[cfg(not(target_os = "windows"))]
    {
        let _ = app;
    }
}
