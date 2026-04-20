let sharedAudioContext: AudioContext | null = null

function getAudioContext() {
  if (typeof window === 'undefined') {
    return null
  }

  const AudioContextCtor =
    window.AudioContext ??
    (window as typeof window & { webkitAudioContext?: typeof AudioContext }).webkitAudioContext

  if (!AudioContextCtor) {
    return null
  }

  if (!sharedAudioContext) {
    sharedAudioContext = new AudioContextCtor()
  }

  return sharedAudioContext
}

export async function playReminderPreview() {
  const audioContext = getAudioContext()
  if (!audioContext) {
    return
  }

  if (audioContext.state === 'suspended') {
    await audioContext.resume()
  }

  const gain = audioContext.createGain()
  const oscillator = audioContext.createOscillator()
  const now = audioContext.currentTime

  oscillator.type = 'sine'
  oscillator.frequency.setValueAtTime(784, now)

  gain.gain.setValueAtTime(0.0001, now)
  gain.gain.exponentialRampToValueAtTime(0.08, now + 0.02)
  gain.gain.exponentialRampToValueAtTime(0.0001, now + 0.28)

  oscillator.connect(gain)
  gain.connect(audioContext.destination)
  oscillator.start(now)
  oscillator.stop(now + 0.3)

  oscillator.onended = () => {
    oscillator.disconnect()
    gain.disconnect()
  }
}
