import { host } from './host'
import { renderFace, renderEmptyRoom } from './render'
import type { DockState } from './types'

const DISCONNECT_TIMEOUT_MS = 1500

let state: DockState = 'UNDOCKED'
let disconnectTimer: ReturnType<typeof setTimeout> | null = null

// --- DOM refs ---
const elEmptyRoom = document.getElementById('empty-room')!
const elDockView = document.getElementById('dock-view')!
const elEditPanel = document.getElementById('edit-panel')!
const elIdName = document.getElementById('id-name')!
const elIdBio = document.getElementById('id-bio')!
const elEditName = document.getElementById('edit-name') as HTMLInputElement
const elEditBio = document.getElementById('edit-bio') as HTMLTextAreaElement

// --- Rendering ---
function render() {
  elEmptyRoom.hidden = state !== 'UNDOCKED'
  elDockView.hidden = state === 'UNDOCKED'
  elEditPanel.hidden = state !== 'DOCKED_EDITING'
}

// --- Face canvas ---
const canvas = document.getElementById('face-canvas') as HTMLCanvasElement
const ctx = canvas.getContext('2d')!

// --- Animation state (matches ui.rs state::FACE / TARGET) ---
// currentPts interpolates toward targetPts by ±2 per coordinate each RAF tick,
// mirroring the interp(start, target, step=2) logic in tick().
// Jitter is re-sampled every draw, exactly as draw_ui() does.
let currentPts: number[] = []
let targetPts:  number[] = []

// Matches state::interp(start, target, step=2)
function interp(start: number, target: number): number {
  if (start < target) return Math.min(start + 2, target)
  if (start > target) return Math.max(start - 2, target)
  return start
}

function animLoop() {
  if (state !== 'UNDOCKED' && currentPts.length === 36) {
    for (let i = 0; i < 36; i++) currentPts[i] = interp(currentPts[i], targetPts[i])
    renderFace(ctx, currentPts)
  }
  requestAnimationFrame(animLoop)
}
requestAnimationFrame(animLoop)

// --- State transitions ---
function transition(next: DockState) {
  if (state === next) return
  state = next
  render()
}

// --- Host events ---
host.onFrame((points) => {
  if (state === 'UNDOCKED') {
    // Snap current to the incoming frame so the first draw is correct,
    // not a lerp from wherever currentPts was before.
    currentPts = [...points]
    transition('DOCKED_FACE')
  }
  targetPts = [...points]
  if (disconnectTimer) clearTimeout(disconnectTimer)
  disconnectTimer = setTimeout(() => transition('UNDOCKED'), DISCONNECT_TIMEOUT_MS)
})

host.onFileResult((path, data) => {
  if (path !== '/id_card.txt') return
  const nl = data.indexOf('\n')
  if (state === 'DOCKED_EDITING') {
    // Populate edit fields
    elEditName.value = nl === -1 ? data : data.slice(0, nl)
    elEditBio.value = nl === -1 ? '' : data.slice(nl + 1)
  } else {
    // Update display
    elIdName.textContent = nl === -1 ? data : data.slice(0, nl)
    elIdBio.textContent = nl === -1 ? '' : data.slice(nl + 1)
  }
})

// --- Button handlers ---
document.getElementById('btn-edit')?.addEventListener('click', () => {
  if (state !== 'DOCKED_FACE') return
  host.sendCommand({ cmd: 'READ', path: '/id_card.txt' })
  transition('DOCKED_EDITING')
})

document.getElementById('btn-save')?.addEventListener('click', () => {
  if (state !== 'DOCKED_EDITING') return
  const data = `${elEditName.value}\n${elEditBio.value}`
  host.sendCommand({ cmd: 'WRITE', path: '/id_card.txt', data })
  elIdName.textContent = elEditName.value
  elIdBio.textContent = elEditBio.value
  transition('DOCKED_FACE')
})

document.getElementById('btn-cancel')?.addEventListener('click', () => {
  transition('DOCKED_FACE')
})

// --- Init ---
render()
