import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia, defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { useServicesStore } from '../services'

// Mock globals for Vitest
(global as any).defineStore = defineStore;
(global as any).ref = ref;
(global as any).computed = computed;

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {}))
}))

// Mock useLogManagerStore
const mockRemoveServiceLogs = vi.fn()
;(global as any).useLogManagerStore = () => ({
  removeServiceLogs: mockRemoveServiceLogs
})

;(global as any).useServiceApi = vi.fn(() => ({
  getServices: vi.fn(() => Promise.resolve([
    { id: 'nginx', name: 'Nginx', port: 8080, defaultVersion: '1.26.2' },
    { id: 'mysql', name: 'MySQL', port: 3306, defaultVersion: '8.0.35' }
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

    expect(store.allDefinitions.length).toBe(2)
    expect(store.getDefinition('mysql')).toBeDefined()
  })
})
