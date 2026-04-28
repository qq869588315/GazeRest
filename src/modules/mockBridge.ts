import type {
  BootstrapPayload,
  BreakSession,
  DetectedDisplaySize,
  PausePreset,
  ReminderEvent,
  Settings,
} from '../types/app'

type EventName =
  | 'state-updated'
  | 'settings-updated'
  | 'reminder-issued'
  | 'break-tick'
  | 'break-finished'
  | 'close-intent'

type Listener<T> = (payload: T) => void

const listeners = new Map<EventName, Set<Listener<unknown>>>()

const now = new Date().toISOString()

let settings: Settings = {
  id: 1,
  language: 'zh-CN',
  reminderIntervalMinutes: 20,
  breakDurationSeconds: 20,
  autoCloseBreakWindow: true,
  reminderLevel: 1,
  soundEnabled: false,
  soundType: null,
  lowDistractionMode: true,
  fullscreenDelayEnabled: true,
  launchAtStartup: true,
  closeButtonBehavior: 'hide_main_window',
  workScheduleEnabled: false,
  activeDays: [1, 2, 3, 4, 5],
  workTimeStart: '09:00',
  workTimeEnd: '18:00',
  timerStyle: 'minimal',
  statusIconMode: 'adaptive',
  windowOpacity: 0.3,
  displayWidthCm: 55,
  displayHeightCm: 31,
  recommendedViewingDistanceCm: 63.13,
  hasCompletedOnboarding: true,
  createdAt: now,
  updatedAt: now,
}

let activeReminder: ReminderEvent | null = null
let activeBreak: BreakSession | null = null
let breakCountdownTimer: number | null = null

let snapshot: BootstrapPayload = {
  settings,
  runtimeState: {
    id: 1,
    currentStatus: 'running',
    activeElapsedSeconds: 488,
    nextReminderDueAt: new Date(Date.now() + 19 * 60 * 1000 + 52 * 1000).toISOString(),
    pausedUntil: null,
    deferredReminderPending: false,
    pendingReminderEventId: null,
    pendingReminderLevel: null,
    lastFullscreenDetectedAt: null,
    lastIdleDetectedAt: null,
    lastActivityAt: now,
    updatedAt: now,
  },
  todaySummary: {
    completedBreakCount: 4,
    skippedCount: 1,
    snoozedCount: 2,
    maxActiveStreakSeconds: 1512,
    completionRate: 67,
  },
  activeReminder,
  activeBreak,
  nowIso: now,
}

export async function bootstrapAppMock() {
  return snapshot
}

export async function saveSettingsMock(nextSettings: Settings) {
  settings = {
    ...nextSettings,
    updatedAt: new Date().toISOString(),
  }
  snapshot = {
    ...snapshot,
    settings,
  }
  emit('settings-updated', settings)
  emit('state-updated', snapshot)
  return settings
}

export async function startBreakMock(triggeredByReminderEventId: number | null) {
  clearBreakCountdownMock()
  activeBreak = {
    id: Date.now(),
    startedAt: new Date().toISOString(),
    endedAt: null,
    durationSeconds: settings.breakDurationSeconds,
    status: 'running',
    cancelReason: null,
    triggeredByReminderEventId,
    createdAt: new Date().toISOString(),
    style: settings.timerStyle,
    remainingSeconds: settings.breakDurationSeconds,
  }
  activeReminder = null
  snapshot = {
    ...snapshot,
    activeBreak,
    activeReminder,
    runtimeState: {
      ...snapshot.runtimeState,
      currentStatus: 'break_in_progress',
      pendingReminderEventId: null,
      pendingReminderLevel: null,
    },
  }
  emit('state-updated', snapshot)
  emit('break-tick', activeBreak)
  if (typeof window !== 'undefined') {
    breakCountdownTimer = window.setInterval(tickBreakCountdownMock, 1000)
  }
  return activeBreak
}

export async function cancelBreakMock() {
  clearBreakCountdownMock()
  const finishedBreak = activeBreak
    ? {
        ...activeBreak,
        status: activeBreak.remainingSeconds <= 0 ? 'completed' : 'canceled',
        cancelReason: activeBreak.remainingSeconds <= 0 ? null : 'user_cancelled',
        endedAt: new Date().toISOString(),
      }
    : null

  activeBreak = null

  snapshot = {
    ...snapshot,
    activeBreak: null,
    runtimeState: {
      ...snapshot.runtimeState,
      currentStatus: 'running',
      activeElapsedSeconds: 0,
      nextReminderDueAt: new Date(
        Date.now() + settings.reminderIntervalMinutes * 60 * 1000,
      ).toISOString(),
    },
  }
  emit('break-finished', finishedBreak)
  emit('state-updated', snapshot)
}

export async function pauseAppMock(preset: PausePreset) {
  const minutes = preset === 'minutes_30' ? 30 : preset === 'hour_1' ? 60 : 600
  snapshot = {
    ...snapshot,
    runtimeState: {
      ...snapshot.runtimeState,
      currentStatus: 'paused',
      pausedUntil: new Date(Date.now() + minutes * 60 * 1000).toISOString(),
    },
  }
  emit('state-updated', snapshot)
}

export async function resumeAppMock() {
  snapshot = {
    ...snapshot,
    runtimeState: {
      ...snapshot.runtimeState,
      currentStatus: 'running',
      pausedUntil: null,
    },
  }
  emit('state-updated', snapshot)
}

export async function minimizeMainWindowMock() {
  return
}

export async function hideMainWindowMock() {
  return
}

export async function closeAppMock() {
  return
}

export async function detectDisplaySizeMock(): Promise<DetectedDisplaySize> {
  return {
    displayWidthCm: 53.1,
    displayHeightCm: 29.9,
  }
}

export async function snoozeReminderMock() {
  activeReminder = null
  snapshot = {
    ...snapshot,
    activeReminder,
    runtimeState: {
      ...snapshot.runtimeState,
      currentStatus: 'snoozed',
      nextReminderDueAt: new Date(Date.now() + 5 * 60 * 1000).toISOString(),
      pendingReminderEventId: null,
      pendingReminderLevel: null,
    },
  }
  emit('state-updated', snapshot)
  emit('reminder-issued', activeReminder)
}

export async function skipReminderMock() {
  activeReminder = null
  snapshot = {
    ...snapshot,
    activeReminder,
    runtimeState: {
      ...snapshot.runtimeState,
      currentStatus: 'running',
      activeElapsedSeconds: 0,
      nextReminderDueAt: new Date(
        Date.now() + settings.reminderIntervalMinutes * 60 * 1000,
      ).toISOString(),
      pendingReminderEventId: null,
      pendingReminderLevel: null,
    },
  }
  emit('state-updated', snapshot)
  emit('reminder-issued', activeReminder)
}

export async function listenMock<T>(
  event: EventName,
  handler: Listener<T>,
) {
  const bucket = listeners.get(event) ?? new Set<Listener<unknown>>()
  bucket.add(handler as Listener<unknown>)
  listeners.set(event, bucket)

  return () => {
    bucket.delete(handler as Listener<unknown>)
  }
}

function emit<T>(event: EventName, payload: T) {
  const bucket = listeners.get(event)
  if (!bucket) {
    return
  }

  for (const handler of bucket) {
    handler(payload)
  }
}

function clearBreakCountdownMock() {
  if (breakCountdownTimer !== null && typeof window !== 'undefined') {
    window.clearInterval(breakCountdownTimer)
  }
  breakCountdownTimer = null
}

function tickBreakCountdownMock() {
  if (!activeBreak) {
    clearBreakCountdownMock()
    return
  }

  activeBreak = {
    ...activeBreak,
    remainingSeconds: Math.max(0, activeBreak.remainingSeconds - 1),
  }

  snapshot = {
    ...snapshot,
    activeBreak,
  }
  emit('break-tick', activeBreak)
  emit('state-updated', snapshot)

  if (activeBreak.remainingSeconds > 0) {
    return
  }

  clearBreakCountdownMock()
  if (!settings.autoCloseBreakWindow) {
    return
  }

  const finishedBreak = {
    ...activeBreak,
    status: 'completed',
    endedAt: new Date().toISOString(),
  }
  activeBreak = null
  snapshot = {
    ...snapshot,
    activeBreak: null,
    runtimeState: {
      ...snapshot.runtimeState,
      currentStatus: 'running',
      activeElapsedSeconds: 0,
      nextReminderDueAt: new Date(
        Date.now() + settings.reminderIntervalMinutes * 60 * 1000,
      ).toISOString(),
    },
  }
  emit('break-finished', finishedBreak)
  emit('state-updated', snapshot)
}
