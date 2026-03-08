const mock = process.env.DOCK_MOCK === '1'

const result = await Bun.build({
  entrypoints: ['./src/app.ts'],
  outdir: './dist',
  naming: 'app.js',
  target: 'browser',
  minify: false,
})

if (!result.success) {
  for (const log of result.logs) console.error(log)
  process.exit(1)
}

const server = Bun.serve({
  port: Number(process.env.PORT ?? 3000),
  async fetch(req) {
    const path = new URL(req.url).pathname
    if (path === '/' || path === '/index.html') {
      return new Response(Bun.file('./index.html'), {
        headers: { 'Content-Type': 'text/html' },
      })
    }
    if (path === '/dist/app.js') {
      return new Response(Bun.file('./dist/app.js'), {
        headers: { 'Content-Type': 'application/javascript' },
      })
    }
    return new Response('Not Found', { status: 404 })
  },
})

console.log(`dock-ui serving on http://localhost:${server.port}${mock ? ' [mock]' : ''}`)
