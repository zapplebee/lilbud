import { test, expect, beforeAll, afterAll, beforeEach } from 'bun:test'
import { chromium, type Browser, type Page } from 'playwright'

const PORT = 3099
const BASE = `http://localhost:${PORT}`

// 18 x,y pairs — a minimal valid face frame
const FAKE_FRAME = Array.from({ length: 36 }, (_, i) => i * 5)

let browser: Browser
let page: Page
let serverProc: ReturnType<typeof Bun.spawn>

beforeAll(async () => {
  serverProc = Bun.spawn(['bun', 'run', 'server.ts'], {
    cwd: import.meta.dir + '/..',
    env: { ...process.env, DOCK_MOCK: '1', PORT: String(PORT) },
    stdout: 'pipe',
    stderr: 'inherit',
  })

  // Wait for the "serving on" line before launching browser
  const reader = serverProc.stdout.getReader()
  while (true) {
    const { value, done } = await reader.read()
    if (done) break
    if (new TextDecoder().decode(value).includes('serving on')) break
  }

  browser = await chromium.launch()
  page = await browser.newPage()
})

afterAll(async () => {
  await browser?.close()
  serverProc?.kill()
})

beforeEach(async () => {
  await page.goto(BASE)
  await page.waitForSelector('#empty-room')
})

// Helpers
const visible = (sel: string) => page.locator(sel).isVisible()
const hidden = (sel: string) => page.locator(sel).isHidden()
const waitVisible = (sel: string) => page.locator(sel).waitFor({ state: 'visible' })
const waitHidden = (sel: string) => page.locator(sel).waitFor({ state: 'hidden' })

const pushFrame = (pts = FAKE_FRAME) =>
  page.evaluate((p) => { (window as any).__mockHost.pushFrame(p) }, pts)

test('starts undocked — shows empty room', async () => {
  expect(await visible('#empty-room')).toBe(true)
  expect(await hidden('#dock-view')).toBe(true)
})

test('first frame transitions to docked', async () => {
  await pushFrame()
  await waitVisible('#dock-view')
  expect(await hidden('#empty-room')).toBe(true)
})

test('no frame for 1.5s returns to undocked', async () => {
  await pushFrame()
  await waitVisible('#dock-view')

  await page.waitForTimeout(1600)
  await waitVisible('#empty-room')
})

test('keepalive frames reset the disconnect timer', async () => {
  await pushFrame()
  await waitVisible('#dock-view')

  for (let i = 0; i < 4; i++) {
    await page.waitForTimeout(500)
    await pushFrame()
  }

  expect(await visible('#dock-view')).toBe(true)
})

test('edit button sends READ and shows edit panel', async () => {
  await pushFrame()
  await waitVisible('#dock-view')

  await page.click('#btn-edit')
  await waitVisible('#edit-panel')

  const cmds = await page.evaluate(() => (window as any).__mockHost.commands())
  expect(cmds).toContainEqual({ cmd: 'READ', path: '/id_card.txt' })
})

test('file result populates edit fields', async () => {
  await pushFrame()
  await page.click('#btn-edit')
  await waitVisible('#edit-panel')

  await page.evaluate(() => {
    (window as any).__mockHost.pushFileResult('/id_card.txt', 'Bud McBudface\nA very cool bud.')
  })

  expect(await page.inputValue('#edit-name')).toBe('Bud McBudface')
  expect(await page.inputValue('#edit-bio')).toBe('A very cool bud.')
})

test('save sends WRITE and updates display', async () => {
  await pushFrame()
  await page.click('#btn-edit')
  await waitVisible('#edit-panel')

  await page.fill('#edit-name', 'New Name')
  await page.fill('#edit-bio', 'New bio text')
  await page.click('#btn-save')

  await waitHidden('#edit-panel')
  expect(await page.textContent('#id-name')).toBe('New Name')

  const cmds = await page.evaluate(() => (window as any).__mockHost.commands())
  const write = cmds.find((c: any) => c.cmd === 'WRITE')
  expect(write?.path).toBe('/id_card.txt')
  expect(write?.data).toContain('New Name')
})

test('cancel closes edit panel without saving', async () => {
  await pushFrame()
  await page.click('#btn-edit')
  await waitVisible('#edit-panel')

  await page.fill('#edit-name', 'Ghost Name')
  await page.click('#btn-cancel')

  await waitHidden('#edit-panel')
  const cmds = await page.evaluate(() => (window as any).__mockHost.commands())
  expect(cmds.filter((c: any) => c.cmd === 'WRITE')).toHaveLength(0)
})
