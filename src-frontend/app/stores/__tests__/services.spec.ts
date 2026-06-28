import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useServicesStore } from '../services'

// Mock dependencies
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}))

vi.mock('../../composables/useServiceApi', () => ({
  useServiceApi: () => ({
    getServices: vi.fn(() => Promise.resolve([
      { id: 'nginx', name: 'Nginx', defaultVersion: '1.25.0' }
    ])),
    startService: vi.fn(() => Promise.resolve(0)),
    stopService: vi.fn(() => Promise.resolve()),
  })
}))

describe('Services Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('initializes with default values', () => {
    const store = useServicesStore()
    expect(store.allDefinitions).toEqual([])
    expect(store.loading.size).toBe(0)
  })

  it('updates status correctly', async () => {
    const store = useServicesStore()

    // We can't access local functions in setup stores easily if they aren't returned
    // Let's test the public init and listen mechanism or just mock the data

    store.statuses.set('nginx', {
      id: 'nginx',
      version: '1.25.0',
      status: 'running',
      port: 80
    })

    expect(store.isRunning('nginx')).toBe(true)
    expect(store.getStatus('nginx')?.port).toBe(80)
  })
})
