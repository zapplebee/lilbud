import type { Host, HostCommand } from './types'

// In-memory mock — used in test builds (DOCK_MOCK=1).
// Exposes window.__mockHost for Playwright test control.
export interface MockControls {
  pushFrame(points: number[]): void
  pushFileResult(path: string, data: string): void
  commands(): HostCommand[]
}

export function createMockHost(): Host {
  const frameCallbacks: Array<(points: number[]) => void> = []
  const fileCallbacks: Array<(path: string, data: string) => void> = []
  const _commands: HostCommand[] = []

  const controls: MockControls = {
    pushFrame: (points) => frameCallbacks.forEach(cb => cb(points)),
    pushFileResult: (path, data) => fileCallbacks.forEach(cb => cb(path, data)),
    commands: () => [..._commands],
  }

  ;(window as any).__mockHost = controls

  return {
    sendCommand(msg) { _commands.push(msg) },
    onFrame(cb) { frameCallbacks.push(cb) },
    onFileResult(cb) { fileCallbacks.push(cb) },
  }
}
