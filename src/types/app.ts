export type AppStatus =
  | 'running'
  | 'paused'
  | 'snoozed'
  | 'break_in_progress'
  | 'outside_schedule'

export type ReminderLevel = 0 | 1 | 2 | 3
export type TimerStyle = 'minimal' | 'breathing' | 'guided'
export type PausePreset = 'minutes_30' | 'hour_1' | 'today'
export type ReminderAction = 'started_break' | 'skipped' | 'snoozed' | 'ignored'
export type WindowView = 'main' | 'reminder' | 'break'
export type CloseButtonBehavior = 'hide_main_window' | 'quit_app'

export interface Settings {
  id: number
  language: 'zh-CN' | 'en-US'
  reminderIntervalMinutes: number
  breakDurationSeconds: number
  reminderLevel: ReminderLevel
  soundEnabled: boolean
  soundType: string | null
  lowDistractionMode: boolean
  fullscreenDelayEnabled: boolean
  launchAtStartup: boolean
  closeButtonBehavior: CloseButtonBehavior
  workScheduleEnabled: boolean
  activeDays: number[]
  workTimeStart: string | null
  workTimeEnd: string | null
  timerStyle: TimerStyle
  statusIconMode: 'adaptive' | 'simple'
  windowOpacity: number
  displayWidthCm: number | null
  displayHeightCm: number | null
  recommendedViewingDistanceCm: number | null
  hasCompletedOnboarding: boolean
  createdAt: string
  updatedAt: string
}

export interface DetectedDisplaySize {
  displayWidthCm: number
  displayHeightCm: number
}

export interface RuntimeState {
  id: number
  currentStatus: AppStatus
  activeElapsedSeconds: number
  nextReminderDueAt: string | null
  pausedUntil: string | null
  deferredReminderPending: boolean
  pendingReminderEventId: number | null
  pendingReminderLevel: ReminderLevel | null
  lastFullscreenDetectedAt: string | null
  lastIdleDetectedAt: string | null
  lastActivityAt: string | null
  updatedAt: string
}

export interface ReminderEvent {
  id: number
  triggeredAt: string
  triggerReason: string
  reminderLevel: ReminderLevel
  wasFullscreenDelayed: boolean
  deliveryType: 'status' | 'card' | 'immersive'
  userAction: ReminderAction | null
  actionAt: string | null
  deferredMinutes: number | null
  activeElapsedSeconds: number
  createdAt: string
  displayMode: 'status' | 'card' | 'immersive'
}

export interface BreakSession {
  id: number
  startedAt: string
  endedAt: string | null
  durationSeconds: number
  status: 'running' | 'completed' | 'canceled' | 'interrupted'
  cancelReason: string | null
  triggeredByReminderEventId: number | null
  createdAt: string
  style: TimerStyle
  remainingSeconds: number
}

export interface TodaySummary {
  completedBreakCount: number
  skippedCount: number
  snoozedCount: number
  maxActiveStreakSeconds: number
  completionRate: number
}

export interface BootstrapPayload {
  settings: Settings
  runtimeState: RuntimeState
  todaySummary: TodaySummary
  activeReminder: ReminderEvent | null
  activeBreak: BreakSession | null
  nowIso: string
}
