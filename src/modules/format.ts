import type { PausePreset } from '../types/app'

export function formatDuration(totalSeconds: number) {
  const safeSeconds = Math.max(0, Math.floor(totalSeconds))
  const hours = Math.floor(safeSeconds / 3600)
  const minutes = Math.floor((safeSeconds % 3600) / 60)
  const seconds = safeSeconds % 60

  if (hours > 0) {
    return `${hours}h ${minutes.toString().padStart(2, '0')}m`
  }

  return `${minutes}m ${seconds.toString().padStart(2, '0')}s`
}

export function formatRemaining(isoValue: string | null) {
  if (!isoValue) {
    return '--'
  }

  const remainingSeconds = secondsUntil(isoValue)
  if (remainingSeconds <= 0) {
    return 'now'
  }

  return formatDuration(remainingSeconds)
}

export function formatShortTime(isoValue: string) {
  return new Intl.DateTimeFormat(undefined, {
    hour: '2-digit',
    minute: '2-digit',
  }).format(new Date(isoValue))
}

export function formatCountdown(totalSeconds: number) {
  const safeSeconds = Math.max(0, Math.floor(totalSeconds))
  const hours = Math.floor(safeSeconds / 3600)
  const minutes = Math.floor((safeSeconds % 3600) / 60)
  const seconds = safeSeconds % 60

  if (hours > 0) {
    return `${hours}:${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`
  }

  return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`
}

export function formatMetricTime(totalSeconds: number) {
  const safeSeconds = Math.max(0, Math.floor(totalSeconds))
  const hours = Math.floor(safeSeconds / 3600)
  const minutes = Math.floor((safeSeconds % 3600) / 60)
  const seconds = safeSeconds % 60

  if (hours > 0) {
    return `${hours.toString().padStart(2, '0')}:${minutes
      .toString()
      .padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`
  }

  return `${minutes.toString().padStart(2, '0')}:${seconds.toString().padStart(2, '0')}`
}

export function secondsUntil(isoValue: string | null) {
  if (!isoValue) {
    return 0
  }

  return Math.max(0, Math.round((new Date(isoValue).getTime() - Date.now()) / 1000))
}

export function localizePausePreset(
  t: (key: string, options?: Record<string, unknown>) => string,
  preset: PausePreset,
) {
  if (preset === 'minutes_30') {
    return t('common.pause30')
  }
  if (preset === 'hour_1') {
    return t('common.pause60')
  }
  return t('common.pauseToday')
}
