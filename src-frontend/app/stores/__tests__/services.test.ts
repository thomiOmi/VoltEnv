import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useServicesStore } from '../services'

// Mocks
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {}))
}))

// We need to provide useServiceApi and useLogManagerStore because they are auto-imported in Nuxt
import { useLogManagerStore } from '../logManager'
(global as any).useLogManagerStore = useLogManagerStore;

(global as any).useServiceApi = vi.fn(() => ({
  getServices: vi.fn(() => Promise.resolve([
    { id: 'nginx', name: 'Nginx', port: 8080, defaultVersion: '1.26.2' }
  ])),
  setupService: vi.fn(),
  startService: vi.fn(),
  stopService: vi.fn()
}))

describe('services store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('initializes and fetches definitions', async () => {
    const store = useServicesStore()
    await store.fetchDefinitions()

    expect(store.allDefinitions.length).toBe(1)
    expect(store.getDefinition('nginx')).toBeDefined()
  })
})
