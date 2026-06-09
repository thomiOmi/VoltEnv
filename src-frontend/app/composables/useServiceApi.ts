import { invoke } from '@tauri-apps/api/core'
import type { ServiceDefinition, ServiceStatus } from '#shared/types/service'
import type { VhostInfo } from '#shared/types/vhost'
import type { QuickCreateResult } from '#shared/types/quick-create'
import type { Settings } from '#shared/types/settings'

export function useServiceApi() {
  const toast = useToast()

  /**
   * Performance Architect Tip: Handles errors from Rust globally at the API layer.
   * Ensures user always gets a meaningful message via Toast notifications.
   */
  async function _handleInvoke<T>(cmd: string, args?: any): Promise<T> {
    try {
      return await invoke<T>(cmd, args)
    }
    catch (e) {
      const message = String(e)
      toast.add({
        title: 'Error',
        description: message,
        color: 'error',
        icon: 'i-lucide-circle-alert'
      })
      throw e
    }
  }

  async function getServices(): Promise<ServiceDefinition[]> {
    try {
      return await invoke<ServiceDefinition[]>('get_services')
    }
    catch (e) {
      console.error('[api] get_services failed:', e)
      return []
    }
  }

  async function setupService(id: string, version: string): Promise<void> {
    await _handleInvoke('setup_service', { id, version })
  }

  async function startService(id: string): Promise<number> {
    return await _handleInvoke<number>('start_service', { id })
  }

  async function stopService(id: string): Promise<void> {
    await _handleInvoke('stop_service', { id })
  }

  async function getServiceStatus(id: string): Promise<ServiceStatus | null> {
    try {
      return await invoke<ServiceStatus | null>('get_service_status', { id })
    }
    catch {
      return null
    }
  }

  async function switchServiceVersion(id: string, version: string): Promise<void> {
    await _handleInvoke('switch_service_version', { id, version })
  }

  async function getServiceLogs(id: string, version: string, linesCount = 50): Promise<string[]> {
    try {
      return await invoke<string[]>('get_service_logs', { id, version, linesCount })
    }
    catch {
      return []
    }
  }

  async function listVhosts(): Promise<VhostInfo[]> {
    try {
      return await invoke<VhostInfo[]>('list_vhosts')
    }
    catch {
      return []
    }
  }

  async function createVhost(domain: string, root: string, port: number, phpPort?: number): Promise<VhostInfo> {
    return await _handleInvoke<VhostInfo>('create_vhost', { domain, root, port, phpPort: phpPort ?? null })
  }

  async function deleteVhost(domain: string): Promise<void> {
    await _handleInvoke('delete_vhost', { domain })
  }

  async function listDatabases(): Promise<string[]> {
    try {
      return await invoke<string[]>('list_databases')
    }
    catch {
      return []
    }
  }

  async function createDatabase(name: string): Promise<void> {
    await _handleInvoke('create_database', { name })
  }

  async function dropDatabase(name: string): Promise<void> {
    await _handleInvoke('drop_database', { name })
  }

  async function createDbUser(username: string, password: string, database: string): Promise<void> {
    await _handleInvoke('create_db_user', { username, password, database })
  }

  async function quickCreate(projectName: string, createDatabase: boolean): Promise<QuickCreateResult> {
    return await _handleInvoke<QuickCreateResult>('quick_create', { projectName, createDatabase })
  }

  async function getSettings(): Promise<Settings | null> {
    try {
      return await invoke<Settings>('get_settings')
    }
    catch {
      return null
    }
  }

  async function updateSettings(settings: Settings): Promise<void> {
    await _handleInvoke('update_settings', { settings })
  }

  async function saveCustomService(service: ServiceDefinition): Promise<void> {
    await _handleInvoke('save_custom_service', { service })
  }

  async function deleteCustomService(id: string): Promise<void> {
    await _handleInvoke('delete_custom_service', { id })
  }

  return {
    getServices,
    setupService,
    startService,
    stopService,
    getServiceStatus,
    switchServiceVersion,
    getServiceLogs,
    listVhosts,
    createVhost,
    deleteVhost,
    listDatabases,
    createDatabase,
    dropDatabase,
    createDbUser,
    quickCreate,
    getSettings,
    updateSettings,
    saveCustomService,
    deleteCustomService,
  }
}
