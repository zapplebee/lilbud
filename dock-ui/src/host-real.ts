import type { Host, HostCommand } from './types'

// Wry WebView IPC implementation.
export function createRealHost(): Host {
  const frameCallbacks: Array<(points: number[]) => void> = []
  const fileCallbacks: Array<(path: string, data: string) => void> = []
  const listCallbacks: Array<(path: string, entries: string[]) => void> = []

  ;(window as any).__boardFrame = (points: number[]) => {
    frameCallbacks.forEach(cb => cb(points))
  }

  ;(window as any).__fileResult = (path: string, data: string) => {
    fileCallbacks.forEach(cb => cb(path, data))
  }

  ;(window as any).__listResult = (path: string, entries: string[]) => {
    listCallbacks.forEach(cb => cb(path, entries))
  }

  return {
    sendCommand(msg: HostCommand) {
      ;(window as any).ipc?.postMessage(JSON.stringify(msg))
    },
    onFrame(cb) { frameCallbacks.push(cb) },
    onFileResult(cb) { fileCallbacks.push(cb) },
    onListResult(cb) { listCallbacks.push(cb) },
  }
}
