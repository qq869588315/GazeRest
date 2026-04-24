import { LogicalSize, getCurrentWindow } from '@tauri-apps/api/window'
import type { WindowView } from '../types/app'

export type MainLayout = 'onboarding' | 'panel' | 'settings' | 'distance'

const isTauriRuntime =
  typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window

const layoutSizes: Record<MainLayout, { width: number; height: number }> = {
  onboarding: { width: 670, height: 400 },
  panel: { width: 670, height: 400 },
  settings: { width: 980, height: 680 },
  distance: { width: 520, height: 620 },
}

let lastSyncedLayout: MainLayout | null = null

export function getWindowView(): WindowView {
  if (typeof window === 'undefined') {
    return 'main'
  }

  const value = new URLSearchParams(window.location.search).get('view')

  if (value === 'reminder' || value === 'break') {
    return value
  }

  return 'main'
}

export async function syncMainWindowLayout(layout: MainLayout) {
  if (!isTauriRuntime || getWindowView() !== 'main') {
    return
  }

  if (lastSyncedLayout === layout) {
    return
  }

  if (lastSyncedLayout === null) {
    lastSyncedLayout = layout
    return
  }

  const currentWindow = getCurrentWindow()
  const nextSize = layoutSizes[layout]
  await currentWindow.setSize(new LogicalSize(nextSize.width, nextSize.height))
  await currentWindow.center()
  lastSyncedLayout = layout
}

export async function startWindowDragging() {
  if (!isTauriRuntime || getWindowView() !== 'main') {
    return
  }

  await getCurrentWindow().startDragging()
}
