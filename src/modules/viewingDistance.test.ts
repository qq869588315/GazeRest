import { calculateViewingDistance, formatDistance } from './viewingDistance'

describe('viewing distance helpers', () => {
  it('calculates distance from monitor diagonal', () => {
    expect(calculateViewingDistance(55, 31)).toBeCloseTo(63.13, 2)
  })

  it('formats distances without a prefix by default', () => {
    expect(formatDistance(63.13, true)).toBe('63 cm')
  })

  it('keeps English panels without a prefix', () => {
    expect(formatDistance(63.13, true, 'en-US')).toBe('63 cm')
  })
})
