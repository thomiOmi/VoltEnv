import { invoke } from '@tauri-apps/api/core'
import type { ServiceDefinition, ServiceStatus } from '#shared/types/service'
import type { VhostInfo } from '#shared/types/vhost'
import type { QuickCreateResult } from '#shared/types/quick-create'
import type { Settings } from '#shared/types/settings'

export function useServiceApi() {
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
    await invoke('setup_service', { id, version })
  }

  async function startService(id: string): Promise<number> {
    try {
      return await invoke<number>('start_service', { id })
    }
    catch (e) {
      throw new Error(String(e))
    }
  }

  async function stopService(id: string): Promise<void> {
    await invoke('stop_service', { id })
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
    await invoke('switch_service_version', { id, version })
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

  async function createVhost(domain: string, root: string, port: number, phpPort?: number, enableSsl = true): Promise<VhostInfo> {
    return await invoke<VhostInfo>('create_vhost', { domain, root, port, phpPort: phpPort ?? null, enableSsl })
  }

  async function deleteVhost(domain: string): Promise<void> {
    await invoke('delete_vhost', { domain })
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
    await invoke('create_database', { name })
  }

  async function dropDatabase(name: string): Promise<void> {
    await invoke('drop_database', { name })
  }

  async function createDbUser(username: string, password: string, database: string): Promise<void> {
    await invoke('create_db_user', { username, password, database })
  }

  async function quickCreate(projectName: string, createDatabase: boolean): Promise<QuickCreateResult> {
    return await invoke<QuickCreateResult>('quick_create', { projectName, createDatabase })
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
    await invoke('update_settings', { settings })
  }

  async function saveCustomService(service: ServiceDefinition): Promise<void> {
    await invoke('save_custom_service', { service })
  }

  async function deleteCustomService(id: string): Promise<void> {
    await invoke('delete_custom_service', { id })
  }

  async function getPhpExtensions(version: string): Promise<[string, boolean][]> {
    try {
      return await invoke<[string, boolean][]>('get_php_extensions', { version })
    }
    catch {
      return []
    }
  }

  async function togglePhpExtension(version: string, extension: string, enable: boolean): Promise<void> {
    await invoke('toggle_php_extension', { version, extension, enable })
  }

  async function runComposerCommand(projectPath: string, args: string[]): Promise<string> {
    return await invoke<string>('run_composer_command', { projectPath, args })
  }

  async function runSelfDiagnostic(): Promise<any> {
    return await invoke('run_self_diagnostic')
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
    getPhpExtensions,
    togglePhpExtension,
    runComposerCommand,
    runSelfDiagnostic,
  }
}
