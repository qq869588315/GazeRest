import { render, screen, waitFor } from '@testing-library/react'
import '../../i18n'
import type { BreakSession, ReminderEvent } from '../../types/app'
import { BreakWindow, ReminderWindow } from './Overlays'

const baseBreakSession: BreakSession = {
  id: 1,
  startedAt: new Date().toISOString(),
  endedAt: null,
  durationSeconds: 20,
  status: 'running',
  cancelReason: null,
  triggeredByReminderEventId: null,
  createdAt: new Date().toISOString(),
  style: 'minimal',
  remainingSeconds: 20,
}

const baseReminder: ReminderEvent = {
  id: 1,
  triggeredAt: new Date().toISOString(),
  triggerReason: 'interval_due',
  reminderLevel: 2,
  wasFullscreenDelayed: false,
  deliveryType: 'card',
  userAction: null,
  actionAt: null,
  deferredMinutes: null,
  activeElapsedSeconds: 1200,
  createdAt: new Date().toISOString(),
  displayMode: 'card',
}

describe('BreakWindow', () => {
  it('does not render a fake countdown without a real break session', () => {
    render(
      <BreakWindow
        language="en-US"
        breakSession={null}
        windowOpacity={1}
        onCancel={() => undefined}
      />,
    )

    expect(screen.getByText('No active break')).toBeInTheDocument()
    expect(screen.queryByText('20')).not.toBeInTheDocument()
  })

  it('resets countdown when a new break session starts', async () => {
    const { rerender } = render(
      <BreakWindow
        language="en-US"
        breakSession={{
          ...baseBreakSession,
          id: 1,
          startedAt: new Date(Date.now() - 13_000).toISOString(),
          remainingSeconds: 7,
        }}
        windowOpacity={1}
        onCancel={() => undefined}
      />,
    )

    await waitFor(() => expect(screen.getByText('7')).toBeInTheDocument())

    rerender(
      <BreakWindow
        language="en-US"
        breakSession={{
          ...baseBreakSession,
          id: 2,
          startedAt: new Date().toISOString(),
          remainingSeconds: 20,
        }}
        windowOpacity={1}
        onCancel={() => undefined}
      />,
    )

    await waitFor(() => expect(screen.getByText('20')).toBeInTheDocument())
  })

  it('keeps counting down without backend ticks', async () => {
    render(
      <BreakWindow
        language="en-US"
        breakSession={{
          ...baseBreakSession,
          startedAt: new Date().toISOString(),
          remainingSeconds: 20,
        }}
        windowOpacity={1}
        onCancel={() => undefined}
      />,
    )

    expect(screen.getByText('20')).toBeInTheDocument()

    await waitFor(
      () => expect(screen.getByText('19')).toBeInTheDocument(),
      { timeout: 1800 },
    )
  })

  it('shows completed state without reusing it as an active countdown', () => {
    render(
      <BreakWindow
        language="en-US"
        breakSession={null}
        completedBreakSession={{
          ...baseBreakSession,
          status: 'completed',
          remainingSeconds: 0,
          endedAt: new Date().toISOString(),
        }}
        windowOpacity={1}
        onCancel={() => undefined}
      />,
    )

    expect(screen.getByText('Break complete')).toBeInTheDocument()
    expect(screen.queryByText('20')).not.toBeInTheDocument()
  })
})

describe('ReminderWindow', () => {
  it('does not render reminder actions without a real reminder event', () => {
    render(
      <ReminderWindow
        language="en-US"
        reminder={null}
        windowOpacity={1}
        onStartBreak={() => undefined}
        onSnooze={() => undefined}
        onSkip={() => undefined}
      />,
    )

    expect(screen.queryByRole('button', { name: 'Start break' })).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Start now' })).not.toBeInTheDocument()
  })

  it('renders real level 2 reminders', () => {
    render(
      <ReminderWindow
        language="en-US"
        reminder={baseReminder}
        windowOpacity={1}
        onStartBreak={() => undefined}
        onSnooze={() => undefined}
        onSkip={() => undefined}
      />,
    )

    expect(screen.getByRole('button', { name: 'Start break' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Skip' })).toBeInTheDocument()
  })
})
