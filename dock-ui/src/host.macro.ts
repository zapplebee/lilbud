// Bun macro — runs at bundle time. Returns whether this is a mock build.
// Set DOCK_MOCK=1 in the bundler process env to get the mock implementation.
export function isMockBuild(): boolean {
  return process.env.DOCK_MOCK === '1'
}
