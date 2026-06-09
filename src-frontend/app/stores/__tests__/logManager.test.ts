import { describe, it, expect, beforeEach, vi } from 'vitest'
import { setActivePinia, createPinia, defineStore } from 'pinia'
import { ref } from 'vue'
import { useLogManagerStore } from '../logManager'

// Mock globals for Vitest
(global as any).defineStore = defineStore;
(global as any).ref = ref;

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {}))
}))

describe('logManager store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('initializes with empty logs', () => {
    const store = useLogManagerStore()
    expect(store.logs).toEqual({})
    expect(store.systemLogs).toEqual([])
  })

  it('appends logs and respects MAX_LOGS_PER_INSTANCE', () => {
    const store = useLogManagerStore()
    const serviceId = 'nginx'
    const version = '1.26.2'

    for (let i = 0; i < 1005; i++) {
      store.pushLog(serviceId, version, `log line ${i}`, false)
    }

    const logs = store.getLogs(serviceId, version)
    expect(logs.length).toBe(1000)
    expect(logs[0].message).toBe('log line 5')
    expect(logs[999].message).toBe('log line 1004')
  })
})
