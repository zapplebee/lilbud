import type { Host, DeviceAction, DeviceEvent } from './types'

// Wry WebView IPC implementation.
export function createRealHost(): Host {
  const callbacks: Array<(event: DeviceEvent) => void> = []

  // Called by the Rust host to push events into the web UI.
  ;(window as any).__deviceEvent = (event: DeviceEvent) => {
    callbacks.forEach(cb => cb(event))
  }

  return {
    dispatch(action: DeviceAction) {
      ;(window as any).ipc?.postMessage(JSON.stringify(action))
    },
    onEvent(cb) { callbacks.push(cb) },
  }
}
