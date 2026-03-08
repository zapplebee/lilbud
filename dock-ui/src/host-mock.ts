import type { Host, DeviceAction, DeviceEvent, DeviceState } from './types'
import { FACES } from './face-fixtures'

export function createMockHost(): Host {
  const callbacks: Array<(event: DeviceEvent) => void> = []

  let state: DeviceState = {
    mode: 'awake',
    identity: { name: 'lilbud', bio: 'a small creature who loves snacks and naps' },
    status: 'just had a really long nap',
    collected: [
      { from: 'sprout', text: 'gave away my last potato chip. no regrets' },
      { from: 'mossdog', text: 'found a really good stick today' },
      { from: 'pebble', text: 'recovering from a big week' },
    ],
    face: { current: FACES[0], target: FACES[0] },
  }

  function emit(event: DeviceEvent) {
    callbacks.forEach(cb => cb(event))
  }

  function sync() {
    emit({ type: 'SYNC', ...state })
  }

  // Simulate device already connected: send initial sync after a tick.
  setTimeout(sync, 0)

  let faceIdx = 0
  let tick = 0
  const FACE_HOLD = 90 // ~3s at 30fps

  setInterval(() => {
    tick++
    if (tick >= FACE_HOLD) {
      tick = 0
      faceIdx = (faceIdx + 1) % FACES.length
      state = { ...state, face: { ...state.face, target: FACES[faceIdx] } }
    }
    emit({ type: 'FRAME', points: FACES[faceIdx] })
  }, 1000 / 30)

  return {
    dispatch(action: DeviceAction) {
      switch (action.type) {
        case 'REQUEST_SYNC':
          setTimeout(sync, 0)
          break
        case 'SET_IDENTITY':
          state = { ...state, identity: { name: action.name, bio: action.bio } }
          sync()
          break
        case 'SET_STATUS':
          state = { ...state, status: action.text }
          sync()
          break
        case 'DISMISS':
          state = { ...state, collected: state.collected.filter(c => c.from !== action.from) }
          sync()
          break
        case 'DO_ACTION': {
          const mode = action.kind === 'sleep' ? 'sleeping' : action.kind === 'feed' ? 'eating' : 'playing'
          state = { ...state, mode }
          sync()
          setTimeout(() => { state = { ...state, mode: 'awake' }; sync() }, 3000)
          break
        }
      }
    },
    onEvent(cb) { callbacks.push(cb) },
  }
}
