import { host } from './host'
import { renderFace } from './render'
import type { DockState } from './types'

const DISCONNECT_TIMEOUT_MS = 1500

let state: DockState = 'UNDOCKED'
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
function render() {
  elEmptyRoom.hidden = state !== 'UNDOCKED'
  elDockView.hidden = state === 'UNDOCKED'
  elEditIdPanel.hidden = state !== 'DOCKED_EDITING'
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
    if (state !== 'UNDOCKED' && currentPts.length === 36) {
      for (let i = 0; i < 36; i++) currentPts[i] = interp(currentPts[i], targetPts[i])
      renderFace(ctx, currentPts)
    }
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

// --- Collected statuses ---
const pendingReads = new Set<string>()

function loadCollected() {
  host.sendCommand({ cmd: 'LIST', path: '/collected' })
}

function escHtml(s: string) {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
}

function renderCollected(name: string, text: string) {
  const id = `collected-${name}`
  if (document.getElementById(id)) return

  const el = document.createElement('div')
  el.className = 'status-entry'
  el.id = id
  el.innerHTML = `
    <span class="status-name">${escHtml(name)}</span>
    <span class="status-text">${escHtml(text)}</span>
    <button class="status-dismiss" title="dismiss">×</button>
  `
  el.querySelector('.status-dismiss')!.addEventListener('click', () => {
    host.sendCommand({ cmd: 'DELETE', path: `/collected/${name}.txt` })
    el.remove()
  })
  elCollectedList.appendChild(el)
}

// --- My status editing ---
elMyStatusDisplay.addEventListener('click', () => {
  if (state !== 'DOCKED_FACE') return
  elStatusInput.value = elMyStatusDisplay.textContent ?? ''
  elMyStatusDisplay.hidden = true
  elMyStatusEdit.hidden = false
  elStatusInput.focus()
})

document.getElementById('btn-save-status')?.addEventListener('click', () => {
  const text = elStatusInput.value.trim()
  if (text) {
    host.sendCommand({ cmd: 'WRITE', path: '/status.txt', data: text })
    elMyStatusDisplay.textContent = text
  }
  elMyStatusDisplay.hidden = false
  elMyStatusEdit.hidden = true
})

document.getElementById('btn-cancel-status')?.addEventListener('click', () => {
  elMyStatusDisplay.hidden = false
  elMyStatusEdit.hidden = true
})

// --- Host events ---
host.onFrame((points) => {
  if (state === 'UNDOCKED') {
    currentPts = [...points]
    transition('DOCKED_FACE')
    host.sendCommand({ cmd: 'READ', path: '/id_card.txt' })
    host.sendCommand({ cmd: 'READ', path: '/status.txt' })
    loadCollected()
  }
  targetPts = [...points]
  if (disconnectTimer) clearTimeout(disconnectTimer)
  disconnectTimer = setTimeout(() => transition('UNDOCKED'), DISCONNECT_TIMEOUT_MS)
})

host.onFileResult((path, data) => {
  if (path === '/id_card.txt') {
    const nl = data.indexOf('\n')
    if (state === 'DOCKED_EDITING') {
      elEditName.value = nl === -1 ? data : data.slice(0, nl)
      elEditBio.value = nl === -1 ? '' : data.slice(nl + 1)
    } else {
      elIdName.textContent = nl === -1 ? data : data.slice(0, nl)
      elIdBio.textContent = nl === -1 ? '' : data.slice(nl + 1)
    }
    return
  }
  if (path === '/status.txt') {
    elMyStatusDisplay.textContent = data
    return
  }
  if (path.startsWith('/collected/')) {
    const filename = path.slice('/collected/'.length)
    const name = filename.endsWith('.txt') ? filename.slice(0, -4) : filename
    pendingReads.delete(filename)
    renderCollected(name, data)
  }
})

host.onListResult((path, entries) => {
  if (path !== '/collected') return
  for (const filename of entries) {
    if (!pendingReads.has(filename) && !document.getElementById(`collected-${filename.replace(/\.txt$/, '')}`)) {
      pendingReads.add(filename)
      host.sendCommand({ cmd: 'READ', path: `/collected/${filename}` })
    }
  }
})

// --- Identity edit ---
document.getElementById('btn-edit-id')?.addEventListener('click', () => {
  if (state !== 'DOCKED_FACE') return
  elEditName.value = elIdName.textContent ?? ''
  elEditBio.value = elIdBio.textContent ?? ''
  transition('DOCKED_EDITING')
})

document.getElementById('btn-save-id')?.addEventListener('click', () => {
  if (state !== 'DOCKED_EDITING') return
  const data = `${elEditName.value}\n${elEditBio.value}`
  host.sendCommand({ cmd: 'WRITE', path: '/id_card.txt', data })
  elIdName.textContent = elEditName.value
  elIdBio.textContent = elEditBio.value
  transition('DOCKED_FACE')
})

document.getElementById('btn-cancel-id')?.addEventListener('click', () => {
  transition('DOCKED_FACE')
})

// --- Action buttons ---
document.getElementById('btn-feed')?.addEventListener('click', () => {
  host.sendCommand({ cmd: 'WRITE', path: '/action.txt', data: 'feed' })
})
document.getElementById('btn-play')?.addEventListener('click', () => {
  host.sendCommand({ cmd: 'WRITE', path: '/action.txt', data: 'play' })
})
document.getElementById('btn-sleep')?.addEventListener('click', () => {
  host.sendCommand({ cmd: 'WRITE', path: '/action.txt', data: 'sleep' })
})

// --- Init ---
render()
