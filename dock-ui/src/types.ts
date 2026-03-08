export type DockState = 'UNDOCKED' | 'DOCKED_FACE' | 'DOCKED_EDITING'

export type DeviceMode = 'awake' | 'sleeping' | 'playing' | 'eating'

export type CollectedStatus = { from: string; text: string }

export type DeviceState = {
  mode: DeviceMode
  identity: { name: string; bio: string }
  status: string
  collected: CollectedStatus[]
  face: { current: number[]; target: number[] }
}

export type DeviceEvent =
  | ({ type: 'SYNC' } & DeviceState)
  | { type: 'FRAME'; points: number[] }

export type DeviceAction =
  | { type: 'REQUEST_SYNC' }
  | { type: 'SET_IDENTITY'; name: string; bio: string }
  | { type: 'SET_STATUS'; text: string }
  | { type: 'DISMISS'; from: string }
  | { type: 'DO_ACTION'; kind: 'feed' | 'play' | 'sleep' }

export interface Host {
  dispatch(action: DeviceAction): void
  onEvent(cb: (event: DeviceEvent) => void): void
}
