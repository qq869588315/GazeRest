import type { CSSProperties, PointerEvent, ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import { startWindowDragging } from '../../modules/window'
import { ArrowLeftIcon, CloseIcon, EyeIcon, GearIcon, MinusIcon } from '../icons'
import { ClosePrompt } from './Overlays'
import { DistanceView, OnboardingView, PanelView } from './MainScreens'
import { SettingsView } from './SettingsScreen'
import type {
  PausePreset,
  ReminderEvent,
  RuntimeState,
  Settings,
  TodaySummary,
} from '../../types/app'
import styles from './MainWindow.module.css'

type SaveStatus = 'idle' | 'saving' | 'saved' | 'error'
type MainScreen = 'panel' | 'settings' | 'distance'
type CloseDecision = 'minimize' | 'quit'

type MainWindowProps = {
  firstRun: boolean
  screen: MainScreen
  savedSettings: Settings
  draftSettings: Settings
  runtimeState: RuntimeState
  todaySummary: TodaySummary
  activeReminder: ReminderEvent | null
  saveStatus: SaveStatus
  hasUnsavedChanges: boolean
  error: string | null
  closePromptOpen: boolean
  onOpenSettings: () => void
  onBack: () => void
  onOpenDistance: () => void
  onStartBreak: () => void
  onToggleLanguage: () => void
  onMinimize: () => void
  onCloseRequest: () => void
  onCloseDecision: (decision: CloseDecision) => void
  onDraftChange: (settings: Settings) => void
  onSettingsSave: () => void
  onPause: (preset: PausePreset) => void
  onResume: () => void
  onOnboardingApply: () => void
  onAutoDetectDistance: () => Promise<{ displayWidthCm: number; displayHeightCm: number } | null>
  onDistanceApply: (displayWidthCm: number, displayHeightCm: number, recommendedViewingDistanceCm: number) => void
}

export function MainWindow({
  firstRun,
  screen,
  savedSettings,
  draftSettings,
  runtimeState,
  todaySummary,
  activeReminder,
  saveStatus,
  hasUnsavedChanges,
  error,
  closePromptOpen,
  onOpenSettings,
  onBack,
  onOpenDistance,
  onStartBreak,
  onToggleLanguage,
  onMinimize,
  onCloseRequest,
  onCloseDecision,
  onDraftChange,
  onSettingsSave,
  onPause,
  onResume,
  onOnboardingApply,
  onAutoDetectDistance,
  onDistanceApply,
}: MainWindowProps) {
  const { t } = useTranslation()
  const title =
    firstRun
      ? t('brand.name')
      : screen === 'settings'
        ? t('settings.title')
        : screen === 'distance'
          ? t('distance.title')
          : t('brand.name')
  const eyebrow =
    firstRun
      ? t('onboarding.eyebrow')
      : screen === 'settings'
        ? t('settings.subtitle')
      : screen === 'distance'
          ? t('distance.subtitle')
          : t('brand.eyebrow')
  const effectiveWindowOpacity =
    screen === 'settings'
      ? draftSettings.windowOpacity
      : savedSettings.windowOpacity
  const windowStyle = {
    ['--window-shell-opacity' as string]: `${Math.min(1, Math.max(0, effectiveWindowOpacity))}`,
  } as CSSProperties

  function handleDragPointerDown(event: PointerEvent<HTMLElement>) {
    if (event.button !== 0) {
      return
    }

    const target = event.target as HTMLElement
    if (target.closest('button, input, select, textarea, a')) {
      return
    }

    event.preventDefault()
    void startWindowDragging()
  }

  return (
    <>
      <main className={styles.viewport} style={windowStyle}>
        <section className={styles.windowShell} onPointerDown={handleDragPointerDown}>
          <header className={styles.windowHeader}>
            <div className={styles.brandArea}>
              <div className={styles.logoMark}>
                <EyeIcon />
                <span className={styles.logoDot} />
              </div>
              <div className={styles.brandText}>
                <h1 className={styles.windowTitle}>{title}</h1>
                <p className={styles.windowEyebrow}>{eyebrow}</p>
              </div>
            </div>
            <div className={styles.headerDragZone} aria-hidden="true" />

            <div className={styles.windowActions}>
              {firstRun ? null : screen === 'panel' ? (
                <HeaderActionButton label={t('common.openSettings')} onClick={onOpenSettings}>
                  <GearIcon />
                </HeaderActionButton>
              ) : (
                <HeaderActionButton label={t('common.back')} onClick={onBack}>
                  <ArrowLeftIcon />
                </HeaderActionButton>
              )}
              {!firstRun ? (
                <HeaderActionButton label={t('common.minimize')} onClick={onMinimize}>
                  <MinusIcon />
                </HeaderActionButton>
              ) : null}
              <HeaderActionButton label={t('common.close')} onClick={onCloseRequest}>
                <CloseIcon />
              </HeaderActionButton>
            </div>
          </header>

          {error ? <p className={styles.errorBanner}>{error}</p> : null}

          <section className={styles.contentArea}>
            {firstRun ? (
              <OnboardingView settings={draftSettings} onApply={onOnboardingApply} />
            ) : screen === 'panel' ? (
              <PanelView
                settings={savedSettings}
                runtimeState={runtimeState}
                todaySummary={todaySummary}
                reminder={activeReminder}
                onOpenDistance={onOpenDistance}
                onStartBreak={onStartBreak}
                onResume={onResume}
                onToggleLanguage={onToggleLanguage}
              />
            ) : screen === 'settings' ? (
              <SettingsView
                savedSettings={savedSettings}
                draftSettings={draftSettings}
                runtimeState={runtimeState}
                saveStatus={saveStatus}
                hasUnsavedChanges={hasUnsavedChanges}
                onChange={onDraftChange}
                onSave={onSettingsSave}
                onPause={onPause}
                onResume={onResume}
              />
            ) : (
              <DistanceView
                settings={savedSettings}
                onCancel={onBack}
                onAutoDetect={onAutoDetectDistance}
                onApply={onDistanceApply}
              />
            )}
          </section>
        </section>
      </main>

      {closePromptOpen ? (
        <ClosePrompt
          windowOpacity={effectiveWindowOpacity}
          onMinimize={() => onCloseDecision('minimize')}
          onQuit={() => onCloseDecision('quit')}
        />
      ) : null}
    </>
  )
}

type HeaderActionButtonProps = {
  label: string
  onClick: () => void
  children: ReactNode
}

function HeaderActionButton({ label, onClick, children }: HeaderActionButtonProps) {
  return (
    <button type="button" className={styles.headerButton} aria-label={label} onClick={onClick}>
      {children}
    </button>
  )
}
