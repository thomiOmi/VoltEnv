import { invoke } from '@tauri-apps/api/core'
import type { DownloadManifest } from '#shared/types/service'

async function execCommand(command: string, args?: Record<string, unknown>): Promise<void> {
  try {
    await invoke(command, args)
  }
  catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    console.error(`[ServiceApi] ${command}: ${message}`)
    throw error
  }
}

async function invokeApi<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(command, args)
  }
  catch (error) {
    const message = error instanceof Error ? error.message : String(error)
    console.error(`[ServiceApi] ${command}: ${message}`)
    throw error
  }
}

export const useServiceApi = () => {
  const getServices = () =>
    invokeApi<ServiceInfo[]>('get_catalog')

  const getServiceStatus = (id: string) =>
    invokeApi<Service>('get_service_status', { id })

  const startService = (id: string, version: string) =>
    execCommand('start_service', { id, version })

  const stopService = (id: string) =>
    execCommand('stop_service', { id })

  const downloadService = (id: string, version: string) =>
    execCommand('download_service', { id, version })

  const installService = (id: string, version: string) =>
    execCommand('install_service', { id, version })

  const softStopService = (id: string) =>
    execCommand('soft_stop_service', { id })

  const forceStopService = (id: string) =>
    execCommand('force_stop_service', { id })

  const switchServiceVersion = (id: string, version: string) =>
    execCommand('switch_service_version', { id, version })

  const getActiveVersions = () =>
    invokeApi<Record<string, string>>('get_active_versions')

  const getServiceLogs = (id: string, linesCount: number = 100) =>
    invokeApi<string[]>('get_service_logs', { id, lines_count: linesCount })

  const registerToOsEnv = () =>
    execCommand('register_to_os_env')

  const registerServiceEnvironment = (id: string, version: string) =>
    execCommand('register_service_environment', { id, version })

  const unregisterServiceEnvironment = (id: string) =>
    execCommand('unregister_service_environment', { id })

  const restoreOsPathBackup = () =>
    execCommand('restore_os_path_backup')

  const isPortAvailable = (port: number) =>
    invokeApi<boolean>('is_port_available', { port })

  const downloadTemplateService = (id: string, manifest: DownloadManifest) =>
    execCommand('download_template_service', { id, manifest })

  return {
    getServices,
    getServiceStatus,
    startService,
    stopService,
    downloadService,
    installService,
    softStopService,
    forceStopService,
    switchServiceVersion,
    getActiveVersions,
    getServiceLogs,
    registerToOsEnv,
    registerServiceEnvironment,
    unregisterServiceEnvironment,
    restoreOsPathBackup,
    isPortAvailable,
    downloadTemplateService,
  }
}
