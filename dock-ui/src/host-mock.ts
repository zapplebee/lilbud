import type { Host, HostCommand } from './types'
import { FACES } from './face-fixtures'

export interface MockControls {
  pushFrame(points: number[]): void
  pushFileResult(path: string, data: string): void
  commands(): HostCommand[]
}

export function createMockHost(): Host {
  const frameCallbacks: Array<(points: number[]) => void> = []
  const fileCallbacks: Array<(path: string, data: string) => void> = []
  const listCallbacks: Array<(path: string, entries: string[]) => void> = []
  const _commands: HostCommand[] = []

  const files: Record<string, string> = {
    '/id_card.txt': 'lilbud\na small creature who loves snacks and naps',
    '/status.txt': 'just had a really long nap',
    '/collected/sprout.txt': 'gave away my last potato chip. no regrets',
    '/collected/mossdog.txt': 'found a really good stick today',
    '/collected/pebble.txt': 'recovering from a big week',
  }

  const controls: MockControls = {
    pushFrame: (points) => frameCallbacks.forEach(cb => cb(points)),
    pushFileResult: (path, data) => fileCallbacks.forEach(cb => cb(path, data)),
    commands: () => [..._commands],
  }

  ;(window as any).__mockHost = controls

  let faceIdx = 0
  let tickCount = 0
  const FACE_HOLD_TICKS = 40
  setInterval(() => {
    controls.pushFrame(FACES[faceIdx])
    tickCount++
    if (tickCount >= FACE_HOLD_TICKS) {
      tickCount = 0
      faceIdx = (faceIdx + 1) % FACES.length
    }
  }, 50)

  return {
    sendCommand(msg) {
      _commands.push(msg)
      if (msg.cmd === 'READ') {
        const data = files[msg.path] ?? ''
        setTimeout(() => fileCallbacks.forEach(cb => cb(msg.path, data)), 0)
      } else if (msg.cmd === 'WRITE') {
        files[msg.path] = msg.data
      } else if (msg.cmd === 'DELETE') {
        delete files[msg.path]
      } else if (msg.cmd === 'LIST') {
        const prefix = msg.path.endsWith('/') ? msg.path : msg.path + '/'
        const entries = Object.keys(files)
          .filter(k => k.startsWith(prefix))
          .map(k => k.slice(prefix.length))
          .filter(k => !k.includes('/'))
        setTimeout(() => listCallbacks.forEach(cb => cb(msg.path, entries)), 0)
      }
    },
    onFrame(cb) { frameCallbacks.push(cb) },
    onFileResult(cb) { fileCallbacks.push(cb) },
    onListResult(cb) { listCallbacks.push(cb) },
  }
}
