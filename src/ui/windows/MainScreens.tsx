import type { ReactNode } from 'react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import clsx from 'clsx'
import { StatusPill } from '../components/StatusPill'
import { ChevronDownIcon, ChevronRightIcon, GlobeIcon } from '../icons'
import { formatCountdown, formatMetricTime, formatShortTime } from '../../modules/format'
import { calculateViewingDistance, formatDistance } from '../../modules/viewingDistance'
import type { ReminderEvent, RuntimeState, Settings, TodaySummary } from '../../types/app'
import styles from './MainScreens.module.css'

type OnboardingProps = {
  settings: Settings
  onApply: () => void
}

type PanelProps = {
  settings: Settings
  runtimeState: RuntimeState
  todaySummary: TodaySummary
  reminder: ReminderEvent | null
  onOpenDistance: () => void
  onStartBreak: () => void
  onResume: () => void
  onToggleLanguage: () => void
}

type DistanceViewProps = {
  settings: Settings
  onCancel: () => void
  onAutoDetect: () => Promise<{ displayWidthCm: number; displayHeightCm: number } | null>
  onApply: (displayWidthCm: number, displayHeightCm: number, recommendedViewingDistanceCm: number) => void
}

export function OnboardingView({ settings, onApply }: OnboardingProps) {
  const { t } = useTranslation()

  return (
    <section className={styles.onboardingView}>
      <div className={styles.heroPanel}>
        <span className={styles.surfaceTag}>{t('onboarding.eyebrow')}</span>
        <h2 className={styles.heroTitle}>{t('onboarding.title')}</h2>
        <p className={styles.heroBody}>{t('onboarding.body')}</p>
        <div className={styles.recommendationGrid}>
          <FeatureChip>{t('onboarding.recommendedItems.interval')}</FeatureChip>
          <FeatureChip>{t('onboarding.recommendedItems.level')}</FeatureChip>
          <FeatureChip>{t('onboarding.recommendedItems.timer')}</FeatureChip>
          <FeatureChip>{t('onboarding.recommendedItems.startup')}</FeatureChip>
        </div>
        <button type="button" className={styles.primaryButton} onClick={onApply}>
          {t('onboarding.useRecommended')}
        </button>
      </div>

      <div className={styles.onboardingFooter}>
        <span>{t('brand.subtitle')}</span>
        <span>{t('common.minutesValue', { value: settings.reminderIntervalMinutes })}</span>
      </div>
    </section>
  )
}

export function PanelView({
  settings,
  runtimeState,
  todaySummary,
  reminder,
  onOpenDistance,
  onStartBreak,
  onResume,
  onToggleLanguage,
}: PanelProps) {
  const { i18n, t } = useTranslation()
  const intervalSeconds = settings.reminderIntervalMinutes * 60
  const awaitingResume =
    runtimeState.currentStatus === 'paused' && runtimeState.pausedUntil === null
  const remainingSeconds = reminder ? 0 : Math.max(0, intervalSeconds - runtimeState.activeElapsedSeconds)
  const countdownLabel = formatCountdown(remainingSeconds)
  const progress = reminder ? 1 : Math.min(1, runtimeState.activeElapsedSeconds / Math.max(intervalSeconds, 1))
  const nextReminderText = reminder
    ? t('panel.readyNow')
    : awaitingResume
      ? t('panel.waitingResume')
    : runtimeState.nextReminderDueAt
      ? t('panel.nextReminderAt', { time: formatShortTime(runtimeState.nextReminderDueAt) })
      : t('status.outside_schedule')
  const currentLanguage = i18n.language === 'en-US' ? 'EN' : '\u4e2d\u6587'

  return (
    <section className={styles.panelView}>
      <article className={styles.countdownCard}>
        <div className={styles.countdownMain}>
          <strong className={styles.countdownValue}>{countdownLabel}</strong>
          <p className={styles.countdownSubtext}>{nextReminderText}</p>
        </div>

        <div className={styles.progressTrack} aria-hidden="true">
          <div className={styles.progressSegments} />
          <div className={styles.progressFill} style={{ width: `${progress * 100}%` }} />
        </div>

        <div className={styles.countdownFooter}>
          <span>
            {t('panel.activeElapsed')}: {formatMetricTime(runtimeState.activeElapsedSeconds)}
          </span>
          <button type="button" className={styles.inlineLinkButton} onClick={onOpenDistance}>
            {t('panel.distanceShortcut')}
            <ChevronRightIcon />
          </button>
        </div>
      </article>

      <article className={styles.summaryStrip}>
        <SummaryStat label={t('panel.todayUsage')} value={formatMetricTime(runtimeState.activeElapsedSeconds)} />
        <SummaryStat
          label={t('panel.longestStreak')}
          value={formatMetricTime(todaySummary.maxActiveStreakSeconds)}
        />
        <SummaryStat
          label={t('panel.recommendedDistance')}
          value={formatDistance(settings.recommendedViewingDistanceCm, true, i18n.language)}
        />
      </article>

      <div className={styles.panelFooter}>
        <div className={styles.panelStatusSlot}>
          <StatusPill status={runtimeState.currentStatus} />
        </div>

        <button
          type="button"
          className={clsx(
            styles.primaryButton,
            styles.panelPrimaryButton,
            awaitingResume && styles.panelResumeButton,
          )}
          onClick={awaitingResume ? onResume : onStartBreak}
        >
          {awaitingResume ? t('common.continueWork') : t('common.takeBreak')}
        </button>

        <div className={styles.panelQuickActions}>
          <button type="button" className={styles.languageButton} onClick={onToggleLanguage}>
            <GlobeIcon />
            <span>{currentLanguage}</span>
            <ChevronDownIcon />
          </button>
        </div>
      </div>

      <div className={styles.panelDots} aria-label={t('panel.dotsLabel')}>
        <span className={clsx(styles.panelDot, styles.panelDotActive)} />
        <span className={styles.panelDot} />
      </div>
    </section>
  )
}

export function DistanceView({ settings, onCancel, onAutoDetect, onApply }: DistanceViewProps) {
  const { i18n, t } = useTranslation()
  const [widthInput, setWidthInput] = useState(
    settings.displayWidthCm === null ? '' : settings.displayWidthCm.toFixed(1),
  )
  const [heightInput, setHeightInput] = useState(
    settings.displayHeightCm === null ? '' : settings.displayHeightCm.toFixed(1),
  )
  const [isDetecting, setIsDetecting] = useState(false)

  const widthValue = Number.parseFloat(widthInput)
  const heightValue = Number.parseFloat(heightInput)
  const viewingDistance = calculateViewingDistance(widthValue, heightValue)

  return (
    <section className={styles.distanceView}>
      <article className={styles.distanceCard}>
        <FieldCard title={t('distance.width')}>
          <div className={styles.measureInputRow}>
            <input
              className={styles.textInput}
              type="number"
              min="1"
              step="0.1"
              inputMode="decimal"
              value={widthInput}
              onChange={(event) => setWidthInput(event.target.value)}
            />
            <span className={styles.unitPill}>cm</span>
          </div>
        </FieldCard>

        <FieldCard title={t('distance.height')}>
          <div className={styles.measureInputRow}>
            <input
              className={styles.textInput}
              type="number"
              min="1"
              step="0.1"
              inputMode="decimal"
              value={heightInput}
              onChange={(event) => setHeightInput(event.target.value)}
            />
            <span className={styles.unitPill}>cm</span>
          </div>
        </FieldCard>

        <div className={styles.assistRow}>
          <button
            type="button"
            className={styles.secondaryButton}
            disabled={isDetecting}
            onClick={async () => {
              setIsDetecting(true)
              try {
                const detected = await onAutoDetect()
                if (!detected) {
                  return
                }

                setWidthInput(detected.displayWidthCm.toFixed(1))
                setHeightInput(detected.displayHeightCm.toFixed(1))
              } finally {
                setIsDetecting(false)
              }
            }}
          >
            {isDetecting ? t('distance.autoDetecting') : t('distance.autoDetect')}
          </button>
          <p className={styles.assistHint}>{t('distance.autoDetectHint')}</p>
        </div>

        <div className={styles.distanceResultCard}>
          <span className={styles.resultLabel}>{t('distance.result')}</span>
          <strong className={styles.resultValue}>{formatDistance(viewingDistance, true, i18n.language)}</strong>
          <p>{t('distance.resultHint')}</p>
          <span className={styles.resultBadge}>{t('distance.localOnly')}</span>
        </div>

        <div className={styles.distanceActions}>
          <button type="button" className={styles.secondaryButton} onClick={onCancel}>
            {t('common.cancel')}
          </button>
          <button
            type="button"
            className={styles.primaryButton}
            disabled={viewingDistance === null}
            onClick={() => {
              if (viewingDistance === null) {
                return
              }

              onApply(widthValue, heightValue, viewingDistance)
            }}
          >
            {t('common.calculate')}
          </button>
        </div>
      </article>
    </section>
  )
}

type SummaryStatProps = {
  label: string
  value: string
}

function SummaryStat({ label, value }: SummaryStatProps) {
  return (
    <article className={styles.summaryStat}>
      <span className={styles.summaryLabel}>{label}</span>
      <strong className={styles.summaryValue}>{value}</strong>
    </article>
  )
}

type FeatureChipProps = {
  children: ReactNode
}

function FeatureChip({ children }: FeatureChipProps) {
  return <span className={styles.featureChip}>{children}</span>
}

type FieldCardProps = {
  title: string
  children: ReactNode
}

function FieldCard({ title, children }: FieldCardProps) {
  return (
    <label className={styles.fieldCard}>
      <span className={styles.fieldTitle}>{title}</span>
      {children}
    </label>
  )
}
