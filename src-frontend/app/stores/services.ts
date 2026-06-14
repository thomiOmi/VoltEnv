import { listen } from '@tauri-apps/api/event'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { useLogManagerStore } from './logManager'
import type { ServiceDefinition, ServiceStatus, ResourceUsage } from '#shared/types/service'
import type { DownloadProgressPayload, InstallProgressPayload, ServiceStatusChangedPayload } from '#shared/types/events'

export const useServicesStore = defineStore('services', () => {
  const definitions = ref<Map<string, ServiceDefinition>>(new Map())
  const statuses = ref<Map<string, ServiceStatus>>(new Map())
  const resourceUsage = ref<Map<string, ResourceUsage>>(new Map())
  const loading = ref<Set<string>>(new Set())
  const installed = ref<Set<string>>(new Set())
  const downloadProgress = ref<Record<string, number>>({})
  const installProgress = ref<Record<string, number>>({})

  let _unlistenStatus: UnlistenFn | null = null
  let _unlistenDownload: UnlistenFn | null = null
  let _unlistenInstall: UnlistenFn | null = null
  let _pollingInterval: any = null

  const { getServices, getResourceUsage } = useServiceApi()
  const logManager = useLogManagerStore()

  const allDefinitions = computed(() => Array.from(definitions.value.values()))

  function getDefinition(id: string): ServiceDefinition | undefined {
    return definitions.value.get(id)
  }

  function getStatus(id: string): ServiceStatus | undefined {
    return statuses.value.get(id)
  }

  function getUsage(id: string): ResourceUsage | undefined {
    return resourceUsage.value.get(id)
  }

  function isRunning(id: string): boolean {
    return statuses.value.get(id)?.status === 'running'
  }

  function isStarting(id: string): boolean {
    return statuses.value.get(id)?.status === 'starting'
  }

  function isStopped(id: string): boolean {
    const s = statuses.value.get(id)
    return !s || s.status === 'stopped'
  }

  function isLoading(id: string): boolean {
    return loading.value.has(id)
  }

  function isInstalled(id: string): boolean {
    return installed.value.has(id)
  }

  async function fetchDefinitions() {
    const list = await getServices()
    const map = new Map<string, ServiceDefinition>()
    for (const svc of list) {
      map.set(svc.id, svc)
    }
    definitions.value = map
  }

  function _updateStatus(id: string, status: string, version?: string, port?: number) {
    if (status === 'installed') {
      installed.value.add(id)
      return
    }
    const existing = statuses.value.get(id) ?? {
      id,
      version: version ?? '',
      status: 'stopped',
      port: port ?? 0,
    }
    statuses.value.set(id, {
      ...existing,
      id,
      version: version ?? existing.version,
      status: status as ServiceStatus['status'],
      port: port ?? existing.port,
    })

    if (status === 'stopped') {
      logManager.removeServiceLogs(id, version ?? existing.version)
      resourceUsage.value.delete(id)
    }
  }

  async function init() {
    await fetchDefinitions()
    installed.value.clear()
    downloadProgress.value = {}
    installProgress.value = {}

    for (const id of definitions.value.keys()) {
      _updateStatus(id, 'stopped')
    }

    _unlistenStatus = await listen<ServiceStatusChangedPayload>('service-status-changed', (event) => {
      const { id, status, version, port } = event.payload
      _updateStatus(id, status, version, port)
    })

    _unlistenDownload = await listen<DownloadProgressPayload>('download-progress', (event) => {
      const { id, progress } = event.payload
      downloadProgress.value[id] = progress
    })

    _unlistenInstall = await listen<InstallProgressPayload>('install-progress', (event) => {
      const { id, progress } = event.payload
      installProgress.value[id] = progress
    })

    /**
     * Performance Architect Tip: Polling for resource usage only for running services.
     * Keeps the overhead low while providing real-time visibility.
     */
    if (!_pollingInterval) {
      _pollingInterval = setInterval(async () => {
        const runningIds = Array.from(statuses.value.values())
          .filter(s => s.status === 'running')
          .map(s => s.id)

        for (const id of runningIds) {
          const usage = await getResourceUsage(id)
          if (usage) {
            resourceUsage.value.set(id, usage)
          } else {
            resourceUsage.value.delete(id)
          }
        }
      }, 2500)
    }
  }

  function disposeListeners() {
    _unlistenStatus?.()
    _unlistenDownload?.()
    _unlistenInstall?.()
    if (_pollingInterval) {
      clearInterval(_pollingInterval)
      _pollingInterval = null
    }
  }

  async function setupService(id: string, version: string) {
    loading.value.add(id)
    try {
      const api = useServiceApi()
      await api.setupService(id, version)
    }
    finally {
      loading.value.delete(id)
    }
  }

  async function startService(id: string) {
    loading.value.add(id)
    _updateStatus(id, 'starting')
    try {
      const api = useServiceApi()
      await api.startService(id)
    }
    catch (e) {
      _updateStatus(id, 'error')
      throw e
    }
    finally {
      loading.value.delete(id)
    }
  }

  async function stopService(id: string) {
    loading.value.add(id)
    try {
      const api = useServiceApi()
      await api.stopService(id)
      _updateStatus(id, 'stopped')
    }
    finally {
      loading.value.delete(id)
    }
  }

  return {
    definitions,
    statuses,
    resourceUsage,
    loading,
    installed,
    downloadProgress,
    installProgress,
    allDefinitions,
    getDefinition,
    getStatus,
    getUsage,
    isRunning,
    isStarting,
    isStopped,
    isLoading,
    isInstalled,
    init,
    disposeListeners,
    fetchDefinitions,
    setupService,
    startService,
    stopService,
  }
})
