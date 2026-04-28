import type { ReactNode } from 'react'
import { useTranslation } from 'react-i18next'
import clsx from 'clsx'
import { formatShortTime, localizePausePreset } from '../../modules/format'
import { playReminderPreview } from '../../modules/sound'
import type {
  CloseButtonBehavior,
  PausePreset,
  ReminderLevel,
  RuntimeState,
  Settings,
  TimerStyle,
} from '../../types/app'
import { StatusPill } from '../components/StatusPill'
import styles from './SettingsScreen.module.css'

const REMINDER_INTERVALS = [20, 30, 40, 50, 60]
const TIMER_STYLES: TimerStyle[] = ['minimal', 'breathing', 'guided']
const REMINDER_LEVELS: ReminderLevel[] = [0, 1, 2, 3]
const SCHEDULE_DAYS = [1, 2, 3, 4, 5, 6, 7]
const CLOSE_BUTTON_BEHAVIORS: CloseButtonBehavior[] = ['hide_main_window', 'quit_app']

function clampPercent(value: number) {
  return Math.min(100, Math.max(0, value))
}

function windowOpacityToBackgroundTransparency(windowOpacity: number) {
  return clampPercent(Math.round((1 - windowOpacity) * 100))
}

function backgroundTransparencyToWindowOpacity(transparency: number) {
  return 1 - clampPercent(transparency) / 100
}

type SaveStatus = 'idle' | 'saving' | 'saved' | 'error'

type SettingsViewProps = {
  savedSettings: Settings
  draftSettings: Settings
  runtimeState: RuntimeState
  saveStatus: SaveStatus
  hasUnsavedChanges: boolean
  onChange: (settings: Settings) => void
  onSave: () => void
  onPause: (preset: PausePreset) => void
  onResume: () => void
}

export function SettingsView({
  savedSettings,
  draftSettings,
  runtimeState,
  saveStatus,
  hasUnsavedChanges,
  onChange,
  onSave,
  onPause,
  onResume,
}: SettingsViewProps) {
  const { t } = useTranslation()
  const privacyItems = t('settings.privacyItems', { returnObjects: true }) as string[]
  const backgroundTransparency = windowOpacityToBackgroundTransparency(draftSettings.windowOpacity)
  const quickActionStatus =
    runtimeState.currentStatus === 'paused' && runtimeState.pausedUntil
      ? t('settings.quickActionsStatusPausedUntil', {
          time: formatShortTime(runtimeState.pausedUntil),
        })
      : runtimeState.currentStatus === 'paused'
        ? t('settings.quickActionsStatusPaused')
        : t('settings.quickActionsStatusRunning')

  return (
    <section className={styles.settingsView}>
      <div className={styles.settingsColumns}>
        <div className={styles.settingsColumn}>
          <SectionCard
            title={t('settings.reminderSection')}
            body={t('settings.reminderSectionBody')}
          >
            <LabeledField title={t('settings.interval')} body={t('settings.intervalHint')}>
              <select
                className={styles.selectControl}
                value={draftSettings.reminderIntervalMinutes}
                onChange={(event) =>
                  onChange({
                    ...draftSettings,
                    reminderIntervalMinutes: Number(event.target.value),
                  })
                }
              >
                {REMINDER_INTERVALS.map((minutes) => (
                  <option key={minutes} value={minutes}>
                    {t('common.minutesValue', { value: minutes })}
                  </option>
                ))}
              </select>
            </LabeledField>

            <LabeledField title={t('settings.language')} body={t('settings.languageHint')}>
              <select
                className={styles.selectControl}
                value={draftSettings.language}
                onChange={(event) =>
                  onChange({
                    ...draftSettings,
                    language: event.target.value as Settings['language'],
                  })
                }
              >
                <option value="zh-CN">{t('settings.languageOptions.zh-CN')}</option>
                <option value="en-US">{t('settings.languageOptions.en-US')}</option>
              </select>
            </LabeledField>

            <LabeledField title={t('settings.windowOpacity')} body={t('settings.windowOpacityHint')}>
              <div className={styles.rangeGroup}>
                <input
                  className={styles.rangeInput}
                  type="range"
                  min="0"
                  max="100"
                  step="1"
                  value={backgroundTransparency}
                  onChange={(event) =>
                    onChange({
                      ...draftSettings,
                      windowOpacity: backgroundTransparencyToWindowOpacity(Number(event.target.value)),
                    })
                  }
                />
                <span className={styles.rangeValue}>
                  {t('settings.opacityValue', {
                    value: backgroundTransparency,
                  })}
                </span>
              </div>
            </LabeledField>

            <div className={styles.subSection}>
              <div className={styles.subSectionHeader}>
                <strong>{t('settings.reminderLevel')}</strong>
                <p>{t('settings.reminderLevelHint')}</p>
              </div>
              <div className={styles.levelGrid}>
                {REMINDER_LEVELS.map((level) => {
                  const active = draftSettings.reminderLevel === level
                  return (
                    <button
                      key={level}
                      type="button"
                      className={clsx(styles.levelCard, active && styles.levelCardActive)}
                      onClick={() =>
                        onChange({
                          ...draftSettings,
                          reminderLevel: level,
                        })
                      }
                    >
                      <span className={styles.levelTitle}>{t(`settings.levels.${level}.title`)}</span>
                      <span className={styles.levelBody}>{t(`settings.levels.${level}.body`)}</span>
                    </button>
                  )
                })}
              </div>
              <p className={styles.inlineHint}>{t(`settings.levelDetails.${draftSettings.reminderLevel}`)}</p>
            </div>

            <div className={styles.toggleGrid}>
              <ToggleTile
                title={t('settings.sound')}
                body={t('settings.soundHint')}
                checked={draftSettings.soundEnabled}
                onToggle={(checked) => {
                  onChange({ ...draftSettings, soundEnabled: checked })
                  if (checked && !draftSettings.soundEnabled) {
                    void playReminderPreview()
                  }
                }}
              />
              <ToggleTile
                title={t('settings.lowDistraction')}
                body={t('settings.lowDistractionHint')}
                checked={draftSettings.lowDistractionMode}
                onToggle={(checked) =>
                  onChange({
                    ...draftSettings,
                    lowDistractionMode: checked,
                  })
                }
              />
            </div>
          </SectionCard>

          <SectionCard title={t('settings.timerSection')} body={t('settings.timerSectionBody')}>
            <LabeledField title={t('settings.breakDuration')} body={t('settings.breakDurationHint')}>
              <div className={styles.fixedValuePill}>
                {t('common.secondsValue', { value: draftSettings.breakDurationSeconds })}
              </div>
            </LabeledField>

            <ToggleTile
              title={t('settings.autoCloseBreakWindow')}
              body={t('settings.autoCloseBreakWindowHint')}
              checked={draftSettings.autoCloseBreakWindow}
              onToggle={(checked) =>
                onChange({
                  ...draftSettings,
                  autoCloseBreakWindow: checked,
                })
              }
            />

            <div className={styles.timerStyleList}>
              {TIMER_STYLES.map((style) => {
                const active = draftSettings.timerStyle === style

                return (
                  <button
                    key={style}
                    type="button"
                    className={clsx(styles.timerStyleCard, active && styles.timerStyleCardActive)}
                    onClick={() =>
                      onChange({
                        ...draftSettings,
                        timerStyle: style,
                      })
                    }
                  >
                    <div>
                      <strong>{t(`settings.timerStyles.${style}.title`)}</strong>
                      <p>{t(`settings.timerStyles.${style}.body`)}</p>
                    </div>
                    <span className={clsx(styles.radioMark, active && styles.radioMarkActive)} />
                  </button>
                )
              })}
            </div>
          </SectionCard>

          <SectionCard
            title={t('settings.runtimeSection')}
            body={t('settings.runtimeSectionBody')}
          >
            <div className={styles.toggleGrid}>
              <ToggleTile
                title={t('settings.fullscreenDelay')}
                body={t('settings.fullscreenDelayHint')}
                checked={draftSettings.fullscreenDelayEnabled}
                onToggle={(checked) =>
                  onChange({
                    ...draftSettings,
                    fullscreenDelayEnabled: checked,
                  })
                }
              />
              <ToggleTile
                title={t('settings.launchAtStartup')}
                body={t('settings.launchAtStartupHint')}
                checked={draftSettings.launchAtStartup}
                onToggle={(checked) => onChange({ ...draftSettings, launchAtStartup: checked })}
              />
            </div>

            <LabeledField
              title={t('settings.closeButtonBehavior')}
              body={t('settings.closeButtonBehaviorHint')}
            >
              <select
                className={styles.selectControl}
                value={draftSettings.closeButtonBehavior}
                onChange={(event) =>
                  onChange({
                    ...draftSettings,
                    closeButtonBehavior: event.target.value as CloseButtonBehavior,
                  })
                }
              >
                {CLOSE_BUTTON_BEHAVIORS.map((behavior) => (
                  <option key={behavior} value={behavior}>
                    {t(`settings.closeButtonOptions.${behavior}`)}
                  </option>
                ))}
              </select>
            </LabeledField>

            <div className={styles.scheduleCard}>
              <div className={styles.scheduleHeader}>
                <div>
                  <strong>{t('settings.workSchedule')}</strong>
                  <p>{t('settings.workScheduleHint')}</p>
                </div>
                <ToggleSwitch
                  checked={draftSettings.workScheduleEnabled}
                  onToggle={(checked) => onChange({ ...draftSettings, workScheduleEnabled: checked })}
                />
              </div>

              <div className={styles.scheduleTimeRow}>
                <input
                  className={styles.timeInput}
                  type="time"
                  disabled={!draftSettings.workScheduleEnabled}
                  value={draftSettings.workTimeStart ?? '09:00'}
                  onChange={(event) =>
                    onChange({
                      ...draftSettings,
                      workTimeStart: event.target.value,
                    })
                  }
                />
                <span className={styles.scheduleDivider}>{t('settings.scheduleDivider')}</span>
                <input
                  className={styles.timeInput}
                  type="time"
                  disabled={!draftSettings.workScheduleEnabled}
                  value={draftSettings.workTimeEnd ?? '18:00'}
                  onChange={(event) =>
                    onChange({
                      ...draftSettings,
                      workTimeEnd: event.target.value,
                    })
                  }
                />
              </div>

              <div className={styles.dayChipRow}>
                {SCHEDULE_DAYS.map((day) => {
                  const active = draftSettings.activeDays.includes(day)
                  return (
                    <button
                      key={day}
                      type="button"
                      disabled={!draftSettings.workScheduleEnabled}
                      className={clsx(styles.dayChip, active && styles.dayChipActive)}
                      onClick={() =>
                        onChange({
                          ...draftSettings,
                          activeDays: active
                            ? draftSettings.activeDays.filter((item) => item !== day)
                            : [...draftSettings.activeDays, day].sort((left, right) => left - right),
                        })
                      }
                    >
                      {t(`settings.scheduleDays.${day - 1}`)}
                    </button>
                  )
                })}
              </div>
            </div>
          </SectionCard>
        </div>

        <aside className={styles.previewColumn}>
          <SectionCard title={t('settings.previewTitle')} body={t('settings.previewBody')}>
            <div className={styles.previewPanel}>
              <span className={styles.previewLabel}>{t('settings.currentStatus')}</span>
              <StatusPill status={runtimeState.currentStatus} />
              <PreviewRow
                label={t('settings.interval')}
                value={t('common.minutesValue', { value: savedSettings.reminderIntervalMinutes })}
              />
              <PreviewRow
                label={t('settings.reminderLevel')}
                value={t(`settings.levels.${savedSettings.reminderLevel}.body`)}
              />
              <PreviewRow
                label={t('settings.timerStyle')}
                value={t(`settings.timerStyles.${savedSettings.timerStyle}.title`)}
              />
              <PreviewRow
                label={t('settings.language')}
                value={
                  savedSettings.language === 'zh-CN'
                    ? t('settings.languageOptions.zh-CN')
                    : t('settings.languageOptions.en-US')
                }
              />
              <PreviewRow
                label={t('settings.closeButtonBehavior')}
                value={t(`settings.closeButtonOptions.${savedSettings.closeButtonBehavior}`)}
              />
            </div>
          </SectionCard>

          <SectionCard title={t('settings.privacyTitle')} body={t('settings.privacyBody')}>
            <div className={styles.privacyList}>
              {privacyItems.map((item) => (
                <div key={item} className={styles.privacyItem}>
                  <span className={styles.privacyDot} />
                  <p>{item}</p>
                </div>
              ))}
            </div>
          </SectionCard>

          <SectionCard title={t('settings.quickActionsTitle')} body={t('settings.quickActionsBody')}>
            <div className={styles.quickActions}>
              {(['minutes_30', 'hour_1', 'today'] as const).map((preset) => (
                <button
                  key={preset}
                  type="button"
                  className={styles.quickActionButton}
                  onClick={() => onPause(preset)}
                >
                  {localizePausePreset(t, preset)}
                </button>
              ))}
              <button type="button" className={styles.secondaryButton} onClick={onResume}>
                {t('common.resumeNow')}
              </button>
            </div>
            <p className={styles.quickActionsStatus}>{quickActionStatus}</p>
          </SectionCard>

          <div className={styles.saveDock}>
            <p className={styles.saveHint}>
              {hasUnsavedChanges ? t('settings.saveHintDirty') : t('settings.saveHintSaved')}
            </p>
            <button
              type="button"
              className={styles.primaryButton}
              disabled={saveStatus === 'saving'}
              onClick={onSave}
            >
              {saveStatus === 'saving'
                ? t('common.saving')
                : saveStatus === 'saved'
                  ? t('common.saved')
                  : t('common.save')}
            </button>
          </div>
        </aside>
      </div>
    </section>
  )
}

type SectionCardProps = {
  title: string
  body: string
  children: ReactNode
}

function SectionCard({ title, body, children }: SectionCardProps) {
  return (
    <article className={styles.sectionCard}>
      <div className={styles.sectionHeader}>
        <h3>{title}</h3>
        <p>{body}</p>
      </div>
      {children}
    </article>
  )
}

type LabeledFieldProps = {
  title: string
  body: string
  children: ReactNode
}

function LabeledField({ title, body, children }: LabeledFieldProps) {
  return (
    <label className={styles.fieldCard}>
      <div className={styles.fieldHeader}>
        <strong>{title}</strong>
        <p>{body}</p>
      </div>
      {children}
    </label>
  )
}

type ToggleTileProps = {
  title: string
  body: string
  checked: boolean
  onToggle: (checked: boolean) => void
}

function ToggleTile({ title, body, checked, onToggle }: ToggleTileProps) {
  return (
    <div className={styles.toggleTile}>
      <div>
        <strong>{title}</strong>
        <p>{body}</p>
      </div>
      <ToggleSwitch checked={checked} onToggle={onToggle} />
    </div>
  )
}

type ToggleSwitchProps = {
  checked: boolean
  onToggle: (checked: boolean) => void
}

function ToggleSwitch({ checked, onToggle }: ToggleSwitchProps) {
  return (
    <button
      type="button"
      className={clsx(styles.toggleSwitch, checked && styles.toggleSwitchActive)}
      aria-pressed={checked}
      onClick={() => onToggle(!checked)}
    >
      <span className={styles.toggleThumb} />
    </button>
  )
}

type PreviewRowProps = {
  label: string
  value: string
}

function PreviewRow({ label, value }: PreviewRowProps) {
  return (
    <div className={styles.previewRow}>
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  )
}
