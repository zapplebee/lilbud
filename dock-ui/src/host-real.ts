import type { Host, HostCommand } from './types'

// Wry WebView IPC implementation.
// Rust calls window.__boardFrame() and window.__fileResult() to push data in.
// JS calls window.ipc.postMessage() to send commands to Rust.
export function createRealHost(): Host {
  const frameCallbacks: Array<(points: number[]) => void> = []
  const fileCallbacks: Array<(path: string, data: string) => void> = []

  ;(window as any).__boardFrame = (points: number[]) => {
    frameCallbacks.forEach(cb => cb(points))
  }

  ;(window as any).__fileResult = (path: string, data: string) => {
    fileCallbacks.forEach(cb => cb(path, data))
  }

  return {
    sendCommand(msg: HostCommand) {
      ;(window as any).ipc?.postMessage(JSON.stringify(msg))
    },
    onFrame(cb) { frameCallbacks.push(cb) },
    onFileResult(cb) { fileCallbacks.push(cb) },
  }
}
