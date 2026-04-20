import clsx from 'clsx'
import styles from './MetricCard.module.css'

type MetricCardProps = {
  label: string
  value: string
  tone: 'ink' | 'calm' | 'bright' | 'muted'
}

export function MetricCard({ label, value, tone }: MetricCardProps) {
  return (
    <article className={clsx(styles.card, styles[tone])}>
      <span>{label}</span>
      <strong>{value}</strong>
    </article>
  )
}
