import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import type {
  BootstrapPayload,
  BreakSession,
  DetectedDisplaySize,
  PausePreset,
  Settings,
} from '../types/app'
import {
  bootstrapAppMock,
  cancelBreakMock,
  closeAppMock,
  detectDisplaySizeMock,
  listenMock,
  hideMainWindowMock,
  minimizeMainWindowMock,
  pauseAppMock,
  resumeAppMock,
  saveSettingsMock,
  skipReminderMock,
  snoozeReminderMock,
  startBreakMock,
} from './mockBridge'

const isTauriRuntime =
  typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

export async function bootstrapApp() {
  if (!isTauriRuntime) {
    return bootstrapAppMock()
  }
  return invoke<BootstrapPayload>('bootstrap_app')
}

export async function saveSettings(settings: Settings) {
  if (!isTauriRuntime) {
    return saveSettingsMock(settings)
  }
  return invoke<Settings>('save_settings', { settings })
}

export async function startBreak(triggeredByReminderEventId: number | null) {
  if (!isTauriRuntime) {
    return startBreakMock(triggeredByReminderEventId)
  }
  return invoke<BreakSession>('start_break', { triggeredByReminderEventId })
}

export async function cancelBreak() {
  if (!isTauriRuntime) {
    return cancelBreakMock()
  }
  return invoke<void>('cancel_break')
}

export async function snoozeReminder() {
  if (!isTauriRuntime) {
    return snoozeReminderMock()
  }
  return invoke<void>('snooze_reminder')
}

export async function skipReminder() {
  if (!isTauriRuntime) {
    return skipReminderMock()
  }
  return invoke<void>('skip_reminder')
}

export async function pauseApp(preset: PausePreset) {
  if (!isTauriRuntime) {
    return pauseAppMock(preset)
  }
  return invoke<void>('pause_app', { preset })
}

export async function resumeApp() {
  if (!isTauriRuntime) {
    return resumeAppMock()
  }
  return invoke<void>('resume_app')
}

export async function minimizeMainWindow() {
  if (!isTauriRuntime) {
    return minimizeMainWindowMock()
  }
  return invoke<void>('minimize_main_window')
}

export async function hideMainWindow() {
  if (!isTauriRuntime) {
    return hideMainWindowMock()
  }
  return invoke<void>('hide_main_window')
}

export async function quitApp() {
  if (!isTauriRuntime) {
    return closeAppMock()
  }
  return invoke<void>('quit_app')
}

export async function detectDisplaySize() {
  if (!isTauriRuntime) {
    return detectDisplaySizeMock()
  }

  return invoke<DetectedDisplaySize>('detect_display_size')
}

export async function subscribeEvent<T>(
  event:
    | 'state-updated'
    | 'settings-updated'
    | 'reminder-issued'
    | 'break-tick'
    | 'break-finished'
    | 'close-intent',
  handler: (payload: T) => void,
) {
  if (!isTauriRuntime) {
    return listenMock(event, handler)
  }

  const unlisten = await listen<T>(event, (payload) => handler(payload.payload))
  return () => void unlisten()
}
