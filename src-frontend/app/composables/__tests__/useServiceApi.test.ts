import { describe, it, expect, vi, beforeEach } from 'vitest'
import { useServiceApi } from '../useServiceApi'

// Mocks
const mockInvoke = vi.fn()
vi.mock('@tauri-apps/api/core', () => ({
  invoke: (...args: any[]) => mockInvoke(...args)
}))

const mockAddToast = vi.fn()
// Manually globalize useToast to avoid auto-import issues in test
;(global as any).useToast = () => ({
  add: mockAddToast
})

describe('useServiceApi composable', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('adds a toast notification when a command fails', async () => {
    const api = useServiceApi()
    mockInvoke.mockRejectedValue('Something went wrong in Rust')

    await expect(api.setupService('nginx', '1.0')).rejects.toThrow()

    expect(mockAddToast).toHaveBeenCalledWith(expect.objectContaining({
      title: 'Error',
      description: 'Something went wrong in Rust',
      color: 'error'
    }))
  })
})
