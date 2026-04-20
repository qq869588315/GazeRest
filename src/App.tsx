import { startTransition, useEffect, useEffectEvent, useState } from 'react'
import { useTranslation } from 'react-i18next'
import styles from './App.module.css'
import {
  bootstrapApp,
  cancelBreak,
  detectDisplaySize,
  minimizeMainWindow,
  pauseApp,
  quitApp,
  resumeApp,
  saveSettings,
  skipReminder,
  snoozeReminder,
  startBreak,
  subscribeEvent,
} from './modules/bridge'
import { getWindowView, syncMainWindowLayout } from './modules/window'
import type {
  BootstrapPayload,
  BreakSession,
  DetectedDisplaySize,
  ReminderEvent,
  RuntimeState,
  Settings,
  TodaySummary,
} from './types/app'
import { MainWindow } from './ui/windows/MainWindow'
import { BreakWindow, ReminderWindow } from './ui/windows/Overlays'

type MainScreen = 'panel' | 'settings' | 'distance'
type SaveStatus = 'idle' | 'saving' | 'saved' | 'error'
type CloseDecision = 'minimize' | 'quit'

const CLOSE_PROMPT_STORAGE_KEY = 'gazerest.closePromptSeen'
const windowView = getWindowView()

function hasSeenClosePrompt() {
  if (typeof window === 'undefined') {
    return false
  }

  return window.localStorage.getItem(CLOSE_PROMPT_STORAGE_KEY) === '1'
}

function markClosePromptSeen() {
  if (typeof window === 'undefined') {
    return
  }

  window.localStorage.setItem(CLOSE_PROMPT_STORAGE_KEY, '1')
}

function App() {
  const { t, i18n } = useTranslation()
  const [loading, setLoading] = useState(true)
  const [languageReady, setLanguageReady] = useState(false)
  const [screen, setScreen] = useState<MainScreen>('panel')
  const [savedSettings, setSavedSettings] = useState<Settings | null>(null)
  const [draftSettings, setDraftSettings] = useState<Settings | null>(null)
  const [runtimeState, setRuntimeState] = useState<RuntimeState | null>(null)
  const [todaySummary, setTodaySummary] = useState<TodaySummary | null>(null)
  const [activeReminder, setActiveReminder] = useState<ReminderEvent | null>(null)
  const [activeBreak, setActiveBreak] = useState<BreakSession | null>(null)
  const [saveStatus, setSaveStatus] = useState<SaveStatus>('idle')
  const [closePromptOpen, setClosePromptOpen] = useState(false)
  const [error, setError] = useState<string | null>(null)

  const hasUnsavedChanges =
    savedSettings !== null &&
    draftSettings !== null &&
    JSON.stringify(savedSettings) !== JSON.stringify(draftSettings)

  const firstRun = savedSettings !== null && !savedSettings.hasCompletedOnboarding
  const activeLayout = firstRun ? 'onboarding' : screen

  const reportError = useEffectEvent((value: unknown) => {
    setError(normalizeError(value, t))
  })

  const applySnapshot = useEffectEvent(
    async (payload: BootstrapPayload, preserveDraft: boolean) => {
      setSavedSettings((currentSavedSettings) => {
        const nextSettings = payload.settings

        setDraftSettings((currentDraftSettings) => {
          if (
            !preserveDraft ||
            currentDraftSettings === null ||
            currentSavedSettings === null ||
            JSON.stringify(currentSavedSettings) === JSON.stringify(currentDraftSettings)
          ) {
            return nextSettings
          }

          return currentDraftSettings
        })

        return nextSettings
      })

      setRuntimeState(payload.runtimeState)
      setTodaySummary(payload.todaySummary)
      setActiveReminder(payload.activeReminder)
      setActiveBreak(payload.activeBreak)
      await i18n.changeLanguage(payload.settings.language)
      setLanguageReady(true)
    },
  )

  useEffect(() => {
    void bootstrapApp()
      .then((payload) => applySnapshot(payload, false))
      .catch((value: unknown) => reportError(value))
      .finally(() => setLoading(false))
  }, [i18n])

  useEffect(() => {
    const disposers: Array<() => void> = []

    const bind = async () => {
      disposers.push(
        await subscribeEvent<BootstrapPayload>('state-updated', (payload) =>
          applySnapshot(payload, true),
        ),
      )
      disposers.push(
        await subscribeEvent<Settings>('settings-updated', async (payload) => {
          await i18n.changeLanguage(payload.language)
          setSavedSettings(payload)
          setDraftSettings(payload)
          setLanguageReady(true)
        }),
      )
      disposers.push(
        await subscribeEvent<ReminderEvent | null>('reminder-issued', (payload) => {
          setActiveReminder(payload)
        }),
      )
      disposers.push(
        await subscribeEvent<BreakSession | null>('break-tick', (payload) => {
          setActiveBreak(payload)
        }),
      )
      disposers.push(
        await subscribeEvent<BreakSession | null>('break-finished', (payload) => {
          setActiveBreak(payload)
        }),
      )
      disposers.push(
        await subscribeEvent<null>('close-intent', () => {
          if (hasSeenClosePrompt()) {
            void minimizeMainWindow().catch((value: unknown) => reportError(value))
            return
          }

          setClosePromptOpen(true)
        }),
      )
    }

    void bind()

    return () => {
      for (const dispose of disposers) {
        dispose()
      }
    }
  }, [i18n])

  useEffect(() => {
    if (loading || windowView !== 'main') {
      return
    }

    void syncMainWindowLayout(activeLayout).catch((value: unknown) => reportError(value))
  }, [activeLayout, loading, reportError])

  async function persistSettings(nextSettings: Settings, mode: 'explicit' | 'immediate') {
    if (mode === 'explicit') {
      setSaveStatus('saving')
    }

    setError(null)

    try {
      const saved = await saveSettings(nextSettings)
      setSavedSettings(saved)
      setDraftSettings(saved)
      await i18n.changeLanguage(saved.language)

      if (mode === 'explicit') {
        setSaveStatus('saved')
        window.setTimeout(() => setSaveStatus('idle'), 1600)
      }

      return saved
    } catch (value) {
      if (mode === 'explicit') {
        setSaveStatus('error')
      }
      setError(normalizeError(value, t))
      return null
    }
  }

  async function handleStartBreak() {
    try {
      await startBreak(activeReminder?.id ?? null)
    } catch (value) {
      setError(normalizeError(value, t))
    }
  }

  async function handleAction(action: () => Promise<void>) {
    try {
      await action()
    } catch (value) {
      setError(normalizeError(value, t))
    }
  }

  async function handleCloseDecision(decision: CloseDecision) {
    setClosePromptOpen(false)
    markClosePromptSeen()

    try {
      if (decision === 'quit') {
        await quitApp()
        return
      }

      await minimizeMainWindow()
    } catch (value) {
      setError(normalizeError(value, t))
    }
  }

  async function requestClose() {
    if (hasSeenClosePrompt()) {
      await handleCloseDecision('minimize')
      return
    }

    setClosePromptOpen(true)
  }

  if (
    loading ||
    !languageReady ||
    savedSettings === null ||
    draftSettings === null ||
    runtimeState === null ||
    todaySummary === null
  ) {
    return (
      <main className={styles.viewport}>
        <section className={styles.loadingState}>
          <div className={styles.loadingOrb} />
          <p>{t('common.loading')}</p>
        </section>
      </main>
    )
  }

  if (windowView === 'reminder') {
    return (
      <ReminderWindow
        language={savedSettings.language}
        reminder={activeReminder}
        windowOpacity={savedSettings.windowOpacity}
        onStartBreak={() => void handleStartBreak()}
        onSnooze={() => void handleAction(snoozeReminder)}
        onSkip={() => void handleAction(skipReminder)}
      />
    )
  }

  if (windowView === 'break') {
    return (
      <BreakWindow
        language={savedSettings.language}
        breakSession={activeBreak}
        windowOpacity={savedSettings.windowOpacity}
        onCancel={() => void handleAction(cancelBreak)}
      />
    )
  }

  return (
    <MainWindow
      firstRun={firstRun}
      screen={screen}
      savedSettings={savedSettings}
      draftSettings={draftSettings}
      runtimeState={runtimeState}
      todaySummary={todaySummary}
      activeReminder={activeReminder}
      saveStatus={saveStatus}
      hasUnsavedChanges={hasUnsavedChanges}
      error={error}
      closePromptOpen={closePromptOpen}
      onOpenSettings={() => startTransition(() => setScreen('settings'))}
      onBack={() => startTransition(() => setScreen('panel'))}
      onOpenDistance={() => startTransition(() => setScreen('distance'))}
      onStartBreak={() => void handleStartBreak()}
      onToggleLanguage={() => {
        const nextLanguage = savedSettings.language === 'zh-CN' ? 'en-US' : 'zh-CN'
        void persistSettings(
          {
            ...savedSettings,
            language: nextLanguage,
          },
          'immediate',
        )
      }}
      onMinimize={() => void minimizeMainWindow().catch((value: unknown) => setError(normalizeError(value, t)))}
      onCloseRequest={() => void requestClose()}
      onCloseDecision={(decision) => void handleCloseDecision(decision)}
      onDraftChange={setDraftSettings}
      onSettingsSave={() => void persistSettings(draftSettings, 'explicit')}
      onPause={(preset) => void handleAction(() => pauseApp(preset))}
      onResume={() => void handleAction(resumeApp)}
      onOnboardingApply={() =>
        void persistSettings(
          {
            ...draftSettings,
            hasCompletedOnboarding: true,
          },
          'explicit',
        )
      }
      onAutoDetectDistance={async (): Promise<DetectedDisplaySize | null> => {
        try {
          return await detectDisplaySize()
        } catch (value) {
          setError(normalizeError(value, t))
          return null
        }
      }}
      onDistanceApply={(displayWidthCm, displayHeightCm, recommendedViewingDistanceCm) => {
        void persistSettings(
          {
            ...savedSettings,
            displayWidthCm,
            displayHeightCm,
            recommendedViewingDistanceCm,
          },
          'immediate',
        ).then((saved) => {
          if (saved) {
            startTransition(() => setScreen('panel'))
          }
        })
      }}
    />
  )
}

function normalizeError(
  value: unknown,
  t: (key: string) => string,
) {
  if (value instanceof Error) {
    return value.message
  }
  if (typeof value === 'string') {
    return value
  }
  return t('errors.unknown')
}

export default App
