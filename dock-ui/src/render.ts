// Matches draw_ui() in src/ui.rs exactly.
//
// Point layout (18 points, indices 0–17, alphabetical a–r):
//   a,b,c,d  — head quad corners (two triangles: abc + cda)
//   e,f / g,h / i,j / k,l / m,n / o,p  — 6 face line segments
//   q, r     — eyes
//
// embedded-graphics Circle::new(top_left, diameter) — center is top_left + (r, r).
// Colors are exact RGB565→RGB888 conversions.

// Rgb565::BLUE    0x001F → (0, 0, 248)
const BG     = '#0000f8'
// Rgb565::GREEN   0x07E0 → (0, 252, 0)
const HEAD   = '#00fc00'
// Rgb565::CSS_PURPLE #800080 → RGB565 round-trip → (132, 0, 132)
const SHADOW = '#840084'
// Rgb565::BLACK
const INK    = '#000000'

const EYE_DIAMETER = 10
const LINE_WIDTH    = 2

function rand(min: number, max: number): number {
  return Math.floor(Math.random() * (max - min + 1)) + min
}

// Matches state::jitter() — uniform (1..=5) offset applied to all points
function jitter(): [number, number] {
  return [rand(1, 5), rand(1, 5)]
}

// Matches state::jitter4() — four independent (-10..=10) offsets for shadow corners
function jitter4(): [[number,number],[number,number],[number,number],[number,number]] {
  return [
    [rand(-10, 10), rand(-10, 10)],
    [rand(-10, 10), rand(-10, 10)],
    [rand(-10, 10), rand(-10, 10)],
    [rand(-10, 10), rand(-10, 10)],
  ]
}

// On the device, embedded-graphics draws the head as two separate filled triangles
// (abc + cda). In Canvas 2D, drawing two triangles that share a diagonal edge (c→a)
// causes anti-aliased semi-transparent pixels along that seam — an artifact that does
// not exist on the device because its framebuffer has no alpha channel.
// Fix: draw the quad as a single closed 4-point polygon so the shared edge is never
// rendered at all. The fill result is identical to two triangles on solid color targets.
function quad(
  ctx: CanvasRenderingContext2D,
  x1: number, y1: number,
  x2: number, y2: number,
  x3: number, y3: number,
  x4: number, y4: number,
) {
  ctx.beginPath()
  ctx.moveTo(x1, y1)
  ctx.lineTo(x2, y2)
  ctx.lineTo(x3, y3)
  ctx.lineTo(x4, y4)
  ctx.closePath()
  ctx.fill()
}

export function renderFace(ctx: CanvasRenderingContext2D, pts: number[]) {
  const W = ctx.canvas.width, H = ctx.canvas.height

  // Background
  ctx.fillStyle = BG
  ctx.fillRect(0, 0, W, H)

  if (pts.length < 36) return

  const [jx, jy] = jitter()
  const sp = jitter4()

  // Offset point by main jitter (matches draw_ui jxy applied to all face points)
  const px = (i: number) => pts[i * 2]     + jx
  const py = (i: number) => pts[i * 2 + 1] + jy

  // Shadow point i with its own per-vertex jitter on top
  const sx = (i: number) => px(i) + sp[i][0]
  const sy = (i: number) => py(i) + sp[i][1]

  // Shadow head  (a=0, b=1, c=2, d=3) — single quad, no seam
  ctx.fillStyle = SHADOW
  quad(ctx, sx(0), sy(0), sx(1), sy(1), sx(2), sy(2), sx(3), sy(3))

  // Head — single quad, no seam
  ctx.fillStyle = HEAD
  quad(ctx, px(0), py(0), px(1), py(1), px(2), py(2), px(3), py(3))

  // Face lines  (pairs: e,f=4,5  g,h=6,7  i,j=8,9  k,l=10,11  m,n=12,13  o,p=14,15)
  ctx.strokeStyle = INK
  ctx.lineWidth = LINE_WIDTH
  for (let i = 4; i <= 14; i += 2) {
    ctx.beginPath()
    ctx.moveTo(px(i),   py(i))
    ctx.lineTo(px(i+1), py(i+1))
    ctx.stroke()
  }

  // Eyes  (q=16, r=17)
  // embedded-graphics Circle::new(top_left, diameter) — visual center = top_left + r
  const r = EYE_DIAMETER / 2
  ctx.fillStyle = INK
  for (const i of [16, 17]) {
    const cx = px(i) + r   // top_left.x + radius = center
    const cy = py(i) + r
    ctx.beginPath()
    ctx.arc(cx, cy, r, 0, Math.PI * 2)
    ctx.fill()
  }
}

export function renderEmptyRoom(ctx: CanvasRenderingContext2D) {
  // Darkened backdrop — same as board's docked state
  ctx.fillStyle = '#00007a'
  ctx.fillRect(0, 0, ctx.canvas.width, ctx.canvas.height)
}
