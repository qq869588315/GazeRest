import {
  formatCountdown,
  formatDuration,
  formatMetricTime,
  formatRemaining,
} from './format'

describe('format helpers', () => {
  it('formats short durations', () => {
    expect(formatDuration(75)).toBe('1m 15s')
  })

  it('formats countdown clocks', () => {
    expect(formatCountdown(1192)).toBe('19:52')
  })

  it('formats metric time for dashboard stats', () => {
    expect(formatMetricTime(488)).toBe('08:08')
  })

  it('returns a readable future countdown label', () => {
    const value = formatRemaining(new Date(Date.now() + 90_000).toISOString())
    expect(value).toMatch(/1m/)
  })
})
