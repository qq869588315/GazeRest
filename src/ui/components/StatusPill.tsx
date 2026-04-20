import { useTranslation } from 'react-i18next'
import clsx from 'clsx'
import type { AppStatus } from '../../types/app'
import styles from './StatusPill.module.css'

type StatusPillProps = {
  status: AppStatus
}

export function StatusPill({ status }: StatusPillProps) {
  const { t } = useTranslation()

  return (
    <span className={clsx(styles.pill, styles[status])}>
      <span className={styles.dot} />
      {t(`status.${status}`)}
    </span>
  )
}
