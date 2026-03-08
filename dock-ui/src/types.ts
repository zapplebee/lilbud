export type DockState = 'UNDOCKED' | 'DOCKED_FACE' | 'DOCKED_EDITING'

export interface Host {
  sendCommand(msg: HostCommand): void
  onFrame(cb: (points: number[]) => void): void
  onFileResult(cb: (path: string, data: string) => void): void
  onListResult(cb: (path: string, entries: string[]) => void): void
}

export type HostCommand =
  | { cmd: 'READ'; path: string }
  | { cmd: 'WRITE'; path: string; data: string }
  | { cmd: 'DELETE'; path: string }
  | { cmd: 'LIST'; path: string }
