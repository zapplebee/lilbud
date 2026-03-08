import { host } from './host'
import { renderFace } from './render'
import type { DockState, DeviceState } from './types'

const DISCONNECT_TIMEOUT_MS = 1500

let uiState: DockState = 'UNDOCKED'
let deviceState: DeviceState | null = null
let disconnectTimer: ReturnType<typeof setTimeout> | null = null

// --- DOM refs ---
const elEmptyRoom = document.getElementById('empty-room')!
const elDockView = document.getElementById('dock-view')!
const elIdName = document.getElementById('id-name')!
const elIdBio = document.getElementById('id-bio')!
const elEditIdPanel = document.getElementById('edit-id-panel')!
const elEditName = document.getElementById('edit-name') as HTMLInputElement
const elEditBio = document.getElementById('edit-bio') as HTMLTextAreaElement
const elMyStatusDisplay = document.getElementById('my-status-display')!
const elMyStatusEdit = document.getElementById('my-status-edit')!
const elStatusInput = document.getElementById('status-input') as HTMLTextAreaElement
const elCollectedList = document.getElementById('collected-list')!

// --- Render ---
function escHtml(s: string) {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
}

function renderFromState(s: DeviceState) {
  elIdName.textContent = s.identity.name
  elIdBio.textContent = s.identity.bio
  elMyStatusDisplay.textContent = s.status

  elCollectedList.innerHTML = ''
  for (const entry of s.collected) {
    const el = document.createElement('div')
    el.className = 'status-entry'
    el.innerHTML = `
      <span class="status-name">${escHtml(entry.from)}</span>
      <span class="status-text">${escHtml(entry.text)}</span>
      <button class="status-dismiss" title="dismiss">×</button>
    `
    el.querySelector('.status-dismiss')!.addEventListener('click', () => {
      host.dispatch({ type: 'DISMISS', from: entry.from })
    })
    elCollectedList.appendChild(el)
  }
}

function render() {
  elEmptyRoom.hidden = uiState !== 'UNDOCKED'
  elDockView.hidden = uiState === 'UNDOCKED'
  elEditIdPanel.hidden = uiState !== 'DOCKED_EDITING'
}

// --- Face canvas ---
const canvas = document.getElementById('face-canvas') as HTMLCanvasElement
const ctx = canvas.getContext('2d')!

let currentPts: number[] = []
let targetPts: number[] = []

function interp(start: number, target: number): number {
  if (start < target) return Math.min(start + 2, target)
  if (start > target) return Math.max(start - 2, target)
  return start
}

const FRAME_MS = 1000 / 30
let lastFrameTime = 0

function animLoop(now: number) {
  if (now - lastFrameTime >= FRAME_MS) {
    lastFrameTime = now
    if (uiState !== 'UNDOCKED' && currentPts.length === 36) {
      for (let i = 0; i < 36; i++) currentPts[i] = interp(currentPts[i], targetPts[i])
      renderFace(ctx, currentPts)
    }
  }
  requestAnimationFrame(animLoop)
}
requestAnimationFrame(animLoop)

// --- UI state transitions ---
function transition(next: DockState) {
  if (uiState === next) return
  uiState = next
  render()
}

function resetDisconnectTimer() {
  if (disconnectTimer) clearTimeout(disconnectTimer)
  disconnectTimer = setTimeout(() => transition('UNDOCKED'), DISCONNECT_TIMEOUT_MS)
}

// --- Host events ---
host.onEvent((event) => {
  if (event.type === 'SYNC') {
    deviceState = event
    renderFromState(event)
    if (uiState === 'UNDOCKED') {
      currentPts = [...event.face.current]
      targetPts = [...event.face.target]
      transition('DOCKED_FACE')
    }
    resetDisconnectTimer()
    return
  }

  if (event.type === 'FRAME') {
    if (event.points.length !== 36) return
    if (uiState === 'UNDOCKED') {
      currentPts = [...event.points]
      targetPts = [...event.points]
      transition('DOCKED_FACE')
      host.dispatch({ type: 'REQUEST_SYNC' })
    }
    targetPts = [...event.points]
    resetDisconnectTimer()
  }
})

// --- My status editing ---
elMyStatusDisplay.addEventListener('click', () => {
  if (uiState !== 'DOCKED_FACE') return
  elStatusInput.value = elMyStatusDisplay.textContent ?? ''
  elMyStatusDisplay.hidden = true
  elMyStatusEdit.hidden = false
  elStatusInput.focus()
})

document.getElementById('btn-save-status')?.addEventListener('click', () => {
  const text = elStatusInput.value.trim()
  if (text) host.dispatch({ type: 'SET_STATUS', text })
  elMyStatusDisplay.hidden = false
  elMyStatusEdit.hidden = true
})

document.getElementById('btn-cancel-status')?.addEventListener('click', () => {
  elMyStatusDisplay.hidden = false
  elMyStatusEdit.hidden = true
})

// --- Identity edit ---
document.getElementById('btn-edit-id')?.addEventListener('click', () => {
  if (uiState !== 'DOCKED_FACE' || !deviceState) return
  elEditName.value = deviceState.identity.name
  elEditBio.value = deviceState.identity.bio
  transition('DOCKED_EDITING')
})

document.getElementById('btn-save-id')?.addEventListener('click', () => {
  if (uiState !== 'DOCKED_EDITING') return
  host.dispatch({ type: 'SET_IDENTITY', name: elEditName.value, bio: elEditBio.value })
  transition('DOCKED_FACE')
})

document.getElementById('btn-cancel-id')?.addEventListener('click', () => {
  transition('DOCKED_FACE')
})

// --- Action buttons ---
document.getElementById('btn-feed')?.addEventListener('click', () => {
  host.dispatch({ type: 'DO_ACTION', kind: 'feed' })
})
document.getElementById('btn-play')?.addEventListener('click', () => {
  host.dispatch({ type: 'DO_ACTION', kind: 'play' })
})
document.getElementById('btn-sleep')?.addEventListener('click', () => {
  host.dispatch({ type: 'DO_ACTION', kind: 'sleep' })
})

// --- Init ---
render()
host.dispatch({ type: 'REQUEST_SYNC' })
