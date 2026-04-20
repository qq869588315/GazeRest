export function calculateViewingDistance(widthCm: number, heightCm: number) {
  if (!Number.isFinite(widthCm) || !Number.isFinite(heightCm) || widthCm <= 0 || heightCm <= 0) {
    return null
  }

  return Math.hypot(widthCm, heightCm)
}

export function formatDistance(
  value: number | null,
  _approximate = false,
  _locale: string | undefined = undefined,
) {
  if (value === null || !Number.isFinite(value) || value <= 0) {
    return '--'
  }

  return `${Math.round(value)} cm`
}
