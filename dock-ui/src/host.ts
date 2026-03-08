import { isMockBuild } from './host.macro' with { type: 'macro' }
import { createRealHost } from './host-real'
import { createMockHost } from './host-mock'

// MOCK is replaced with a boolean literal at bundle time by the macro.
// Tree-shaking drops the unused implementation.
const MOCK = isMockBuild()

export const host = MOCK ? createMockHost() : createRealHost()
