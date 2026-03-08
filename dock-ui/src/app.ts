import { host } from './host'
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

function drawFace(points: number[]) {
  ctx.clearRect(0, 0, canvas.width, canvas.height)
  ctx.fillStyle = '#1e3a5f'
  ctx.fillRect(0, 0, canvas.width, canvas.height)
  if (points.length < 2) return
  ctx.strokeStyle = '#7dd3fc'
  ctx.lineWidth = 2
  ctx.beginPath()
  ctx.moveTo(points[0], points[1])
  for (let i = 2; i < points.length; i += 2) {
    ctx.lineTo(points[i], points[i + 1])
  }
  ctx.stroke()
}

// --- State transitions ---
function transition(next: DockState) {
  if (state === next) return
  state = next
  render()
}

// --- Host events ---
host.onFrame((points) => {
  if (state === 'UNDOCKED') transition('DOCKED_FACE')
  drawFace(points)
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
