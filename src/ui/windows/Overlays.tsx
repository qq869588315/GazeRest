import { useEffect, useState } from 'react'
import { useTranslation } from 'react-i18next'
import clsx from 'clsx'
import { formatDuration } from '../../modules/format'
import type { BreakSession, ReminderEvent, Settings } from '../../types/app'
import { EyeIcon, HourglassIcon } from '../icons'
import styles from './Overlays.module.css'

type ReminderWindowProps = {
  language: Settings['language']
  reminder: ReminderEvent | null
  windowOpacity: number
  onStartBreak: () => void
  onSnooze: () => void
  onSkip: () => void
}

type LevelReminderProps = {
  reminder: ReminderEvent | null
  onStartBreak: () => void
  onSnooze: () => void
}

type Level2ReminderProps = LevelReminderProps & {
  onSkip: () => void
}

type BreakWindowProps = {
  language: Settings['language']
  breakSession: BreakSession | null
  windowOpacity: number
  onCancel: () => void
}

type ClosePromptProps = {
  windowOpacity: number
  onHideToTray: () => void
  onQuit: () => void
}

export function ReminderWindow({
  language,
  reminder,
  windowOpacity,
  onStartBreak,
  onSnooze,
  onSkip,
}: ReminderWindowProps) {
  const { i18n } = useTranslation()
  const level = reminder?.reminderLevel ?? 1

  useEffect(() => {
    if (i18n.language !== language) {
      void i18n.changeLanguage(language)
    }
  }, [i18n, language])

  if (level === 3) {
    return (
      <Level3Reminder
        reminder={reminder}
        windowOpacity={windowOpacity}
        onStartBreak={onStartBreak}
        onSnooze={onSnooze}
      />
    )
  }

  if (level === 2) {
    return (
      <Level2Reminder
        reminder={reminder}
        windowOpacity={windowOpacity}
        onStartBreak={onStartBreak}
        onSnooze={onSnooze}
        onSkip={onSkip}
      />
    )
  }

  return (
    <Level1Reminder
      reminder={reminder}
      windowOpacity={windowOpacity}
      onStartBreak={onStartBreak}
      onSnooze={onSnooze}
    />
  )
}

export function BreakWindow({
  language,
  breakSession,
  windowOpacity: _windowOpacity,
  onCancel,
}: BreakWindowProps) {
  const { t, i18n } = useTranslation()
  const [displayRemainingSeconds, setDisplayRemainingSeconds] = useState(
    breakSession?.remainingSeconds ?? breakSession?.durationSeconds ?? 20,
  )
  const style = breakSession?.style ?? 'minimal'
  const totalSeconds = breakSession?.durationSeconds ?? 20
  const remainingSeconds = breakSession === null ? totalSeconds : displayRemainingSeconds
  const progress = Math.max(0, Math.min(1, 1 - remainingSeconds / Math.max(totalSeconds, 1)))
  const hint =
    style === 'guided'
      ? t('break.guidedHint')
      : style === 'breathing'
        ? t('break.breathingHint')
        : t('break.minimalHint')

  useEffect(() => {
    if (i18n.language !== language) {
      void i18n.changeLanguage(language)
    }
  }, [i18n, language])

  useEffect(() => {
    if (!breakSession) {
      setDisplayRemainingSeconds(totalSeconds)
      return
    }

    const startedAtMs = new Date(breakSession.startedAt).getTime()
    const endAtMs = Number.isNaN(startedAtMs)
      ? Date.now() + breakSession.remainingSeconds * 1000
      : startedAtMs + breakSession.durationSeconds * 1000

    const syncRemaining = () => {
      setDisplayRemainingSeconds(Math.max(0, Math.ceil((endAtMs - Date.now()) / 1000)))
    }

    syncRemaining()
    const timer = window.setInterval(() => {
      syncRemaining()
    }, 200)

    return () => {
      window.clearInterval(timer)
    }
  }, [breakSession, totalSeconds])

  return (
    <main
      className={clsx(styles.viewport, styles.breakViewport)}
      style={{ ['--window-shell-opacity' as string]: '0.98' }}
    >
      <div className={styles.scaleFrame}>
        <section className={clsx(styles.breakShell, style === 'guided' && styles.breakShellGuided)}>
          <p className={styles.breakEyebrow}>{t('break.eyebrow')}</p>
          <h2>{t('break.title')}</h2>
          <div className={styles.countdownFace}>
            <div
              className={clsx(
                styles.countdownAura,
                style === 'breathing' && styles.countdownAuraBreathing,
                style === 'guided' && styles.countdownAuraGuided,
              )}
              style={{ ['--progress' as string]: `${progress}` }}
            />
            <strong className={styles.breakValue}>{remainingSeconds}</strong>
          </div>
          <p className={styles.breakHint}>{hint}</p>
          <button type="button" className={styles.secondaryButton} onClick={onCancel}>
            {t('common.cancelBreak')}
          </button>
        </section>
      </div>
    </main>
  )
}

export function ClosePrompt({ windowOpacity, onHideToTray, onQuit }: ClosePromptProps) {
  const { t } = useTranslation()

  return (
    <div
      className={styles.closePromptLayer}
      style={{ ['--window-shell-opacity' as string]: `${Math.min(1, Math.max(0, windowOpacity))}` }}
    >
      <div className={styles.scaleFrame}>
        <section className={styles.closePromptCard} role="dialog" aria-modal="true">
          <h3>{t('closePrompt.title')}</h3>
          <p>{t('closePrompt.body')}</p>
          <div className={styles.closePromptActions}>
            <button type="button" className={styles.primaryButton} autoFocus onClick={onHideToTray}>
              {t('closePrompt.hideToTray')}
            </button>
            <button type="button" className={styles.secondaryButton} onClick={onQuit}>
              {t('closePrompt.quit')}
            </button>
          </div>
          <span className={styles.closePromptHint}>{t('closePrompt.hint')}</span>
        </section>
      </div>
    </div>
  )
}

function Level1Reminder({
  reminder,
  windowOpacity,
  onStartBreak,
  onSnooze,
}: LevelReminderProps & { windowOpacity: number }) {
  const { t } = useTranslation()

  return (
    <main
      className={clsx(styles.viewport, styles.reminderViewport)}
      style={{ ['--window-shell-opacity' as string]: `${Math.min(1, Math.max(0, windowOpacity))}` }}
    >
      <div className={styles.scaleFrame}>
        <section className={styles.reminderCardLevel1}>
          <div className={styles.reminderAccent} />
          <div className={styles.reminderContent}>
            <h2>{t('reminder.level1Title')}</h2>
            <p>{t('reminder.level1Body')}</p>
            <div className={styles.reminderActionsCompact}>
              <button type="button" className={styles.secondaryButton} onClick={onSnooze}>
                {t('common.snoozeShort')}
              </button>
              <button type="button" className={styles.primaryButton} onClick={onStartBreak}>
                {t('common.startBreak')}
              </button>
            </div>
          </div>
          {reminder ? <span className={styles.floatingDot} /> : null}
        </section>
      </div>
    </main>
  )
}

function Level2Reminder({
  reminder,
  windowOpacity,
  onStartBreak,
  onSnooze,
  onSkip,
}: Level2ReminderProps & { windowOpacity: number }) {
  const { t } = useTranslation()

  return (
    <main
      className={clsx(styles.viewport, styles.reminderViewport)}
      style={{ ['--window-shell-opacity' as string]: `${Math.min(1, Math.max(0, windowOpacity))}` }}
    >
      <div className={styles.scaleFrame}>
        <section className={styles.reminderCardLevel2}>
          <div className={styles.reminderIconCircle}>
            <EyeIcon />
          </div>
          <h2>{t('reminder.level2Title')}</h2>
          <p>{t('reminder.level2Body')}</p>
          <div className={styles.reminderActionRow}>
            <button type="button" className={styles.secondaryButton} onClick={onSkip}>
              {t('common.skip')}
            </button>
            <button type="button" className={styles.secondaryButton} onClick={onSnooze}>
              {t('common.snooze')}
            </button>
            <button type="button" className={styles.primaryButton} onClick={onStartBreak}>
              {t('common.startBreak')}
            </button>
          </div>
          {reminder ? (
            <span className={styles.reminderSubtitle}>{formatDuration(reminder.activeElapsedSeconds)}</span>
          ) : null}
        </section>
      </div>
    </main>
  )
}

function Level3Reminder({
  reminder,
  windowOpacity,
  onStartBreak,
  onSnooze,
}: LevelReminderProps & { windowOpacity: number }) {
  const { t } = useTranslation()

  return (
    <main
      className={clsx(styles.viewport, styles.immersiveViewport)}
      style={{ ['--window-shell-opacity' as string]: `${Math.min(1, Math.max(0, windowOpacity))}` }}
    >
      <div className={styles.scaleFrameFull}>
        <section className={styles.immersiveShell}>
          <div className={styles.immersiveCard}>
            <div className={styles.reminderIconCircle}>
              <HourglassIcon />
            </div>
            <h2>{t('reminder.level3Title')}</h2>
            <p>{t('reminder.level3Body')}</p>
            <div className={styles.immersiveHint}>{t('reminder.level3Hint')}</div>
            <div className={styles.reminderActionRow}>
              <button type="button" className={styles.secondaryButton} onClick={onSnooze}>
                {t('common.snooze')}
              </button>
              <button type="button" className={styles.primaryButton} onClick={onStartBreak}>
                {t('common.startBreakNow')}
              </button>
            </div>
            {reminder ? (
              <span className={styles.reminderSubtitle}>{formatDuration(reminder.activeElapsedSeconds)}</span>
            ) : null}
          </div>
        </section>
      </div>
    </main>
  )
}
