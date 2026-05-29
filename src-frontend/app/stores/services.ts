import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

interface ServiceInfo {
  id: string
  name: string
  port: number
  version: string
  versions?: string[]
  downloadUrl: string
  sha256?: string
  pgpSignatureUrl?: string
}

const SERVICE_INFO: Record<string, ServiceInfo> = {
  nginx: {
    id: 'nginx',
    name: 'Nginx',
    port: 80,
    version: 'nginx-1.26.2',
    versions: ['nginx-1.26.2'],
    downloadUrl: 'https://nginx.org/download/nginx-1.26.2.zip',
    pgpSignatureUrl: 'https://nginx.org/download/nginx-1.26.2.zip.asc',
  },
  php: { id: 'php', name: 'PHP-CGI', port: 9000, version: 'unknown', downloadUrl: '' },
  mysql: { id: 'mysql', name: 'MySQL', port: 3306, version: 'unknown', downloadUrl: '' },
  redis: {
    id: 'redis',
    name: 'Redis',
    port: 6379,
    version: 'redis-7.2.14',
    versions: ['redis-7.2.14', 'redis-7.0.15'],
    downloadUrl: '',
    sha256: 'B31D0F867608017F0B0962624D55A4C569A745587AD4B08F7FE9EEA59D6916C1',
  },
}

export interface Service {
  id: string
  name: string
  version: string
  status: 'Running' | 'Stopped' | 'Starting' | { Error: string }
  port: number
}

export const useServicesStore = defineStore('services', {
  state: () => ({
    services: Object.values(SERVICE_INFO).map(s => ({
      id: s.id,
      name: s.name,
      version: s.version,
      status: 'Stopped' as Service['status'],
      port: s.port,
    })),
    loadingStates: {} as Record<string, boolean>,
    downloadProgress: {} as Record<string, number>,
    installProgress: {} as Record<string, number>,
    serviceStep: {} as Record<string, 'idle' | 'downloading' | 'installing' | 'ready'>,
    portError: null as string | null,
    activeVersions: {} as Record<string, string>,
    switchingVersions: {} as Record<string, boolean>,
  }),

  getters: {
    isActiveVersion: (state) => {
      return (id: string, version: string): boolean => {
        return state.activeVersions[id] === version
      }
    },
    versionsFor: () => {
      return (id: string): string[] => {
        return SERVICE_INFO[id]?.versions ?? []
      }
    },
  },

  actions: {
    async init() {
      listen<{ id: string, progress: number }>('download-progress', (event) => {
        const { id, progress } = event.payload
        this.serviceStep = { ...this.serviceStep, [id]: 'downloading' }
        this.downloadProgress = { ...this.downloadProgress, [id]: progress }
      })

      listen<{ id: string, progress: number }>('install-progress', (event) => {
        const { id, progress } = event.payload
        this.serviceStep = { ...this.serviceStep, [id]: 'installing' }
        this.installProgress = { ...this.installProgress, [id]: progress }
        if (progress >= 100) {
          this.serviceStep = { ...this.serviceStep, [id]: 'ready' }
        }
      })

      listen<{ id: string, status: string }>('service-status-changed', (event) => {
        const { id, status } = event.payload
        this._updateStatus(id, status as Service['status'])
      })

      await this.fetchActiveVersions()
    },

    async fetchActiveVersions() {
      try {
        const versions = await invoke<Record<string, string>>('get_active_versions')
        this.activeVersions = versions
      }
      catch (e) {
        console.error('Failed to fetch active versions', e)
      }
    },

    async switchServiceVersion(id: string, version: string) {
      this.switchingVersions = { ...this.switchingVersions, [id]: true }

      const log = useLogManagerStore()
      log.pushLog(id, version, `Switching to version ${version}\u2026`, false)

      try {
        await invoke('switch_service_version', { id, version })
        this.activeVersions = { ...this.activeVersions, [id]: version }
        log.pushLog(id, version, `Active version switched to ${version}`, false)
      }
      catch (error) {
        const msg = error instanceof Error ? error.message : String(error)
        log.pushLog(id, version, `Switch failed: ${msg}`, true)
        console.error(`Failed to switch version for ${id}`, error)
      }
      finally {
        this.switchingVersions = { ...this.switchingVersions, [id]: false }
      }
    },

    _updateStatus(id: string, status: Service['status']) {
      const idx = this.services.findIndex(s => s.id === id)
      const service = idx !== -1 ? this.services[idx] : undefined
      if (service) {
        service.status = status
      }
    },

    async startService(id: string) {
      const info = SERVICE_INFO[id]
      if (!info) return

      this.loadingStates = { ...this.loadingStates, [id]: true }
      this._updateStatus(id, 'Starting')

      const log = useLogManagerStore()
      log.pushLog(id, info.version, 'Starting service\u2026', false)

      try {
        const available = await invoke<boolean>('is_port_available', { port: info.port })
        if (!available) {
          this.portError = `Port ${info.port} (${info.name}) is already in use. Please free the port and try again.`
          log.pushLog(id, info.version, `Port ${info.port} is already in use`, true)
          this._updateStatus(id, 'Stopped')
          return
        }

        this.portError = null

        // download_service resolves URL from the catalog's url_template
        await invoke('download_service', { id, version: info.version })
        await invoke('install_service', { id, version: info.version })
        await invoke('register_to_os_env')
        await invoke('register_service_environment', { id, version: info.version })
        this.activeVersions = { ...this.activeVersions, [id]: info.version }
        await invoke('start_service', { id, version: info.version })
        log.pushLog(id, info.version, 'Service started', false)
        this._updateStatus(id, 'Running')
      }
      catch (error) {
        const msg = error instanceof Error ? error.message : String(error)
        log.pushLog(id, info.version, `Failed: ${msg}`, true)
        console.error(`Failed to start service ${id}`, error)
        this._updateStatus(id, 'Stopped')
      }
      finally {
        this.loadingStates = { ...this.loadingStates, [id]: false }
      }
    },

    async stopService(id: string) {
      const info = SERVICE_INFO[id]
      if (!info) return

      this.loadingStates = { ...this.loadingStates, [id]: true }

      const log = useLogManagerStore()
      log.pushLog(id, info.version, 'Stopping service\u2026', false)

      try {
        await invoke('stop_service', { id, version: info.version })
        await invoke('unregister_service_environment', { id })
        this._updateStatus(id, 'Stopped')
      }
      catch (error) {
        const msg = error instanceof Error ? error.message : String(error)
        log.pushLog(id, info.version, `Stop failed: ${msg}`, true)
        console.error(`Failed to stop service ${id}`, error)
      }
      finally {
        this.loadingStates = { ...this.loadingStates, [id]: false }
      }
    },

    async softStopService(id: string) {
      try {
        await invoke('soft_stop_service', { id, version: SERVICE_INFO[id]?.version })
      }
      catch (error) {
        console.error(`Failed to soft-stop service ${id}`, error)
      }
    },

    async forceStopService(id: string) {
      const info = SERVICE_INFO[id]
      if (!info) return

      this.loadingStates = { ...this.loadingStates, [id]: true }

      const log = useLogManagerStore()
      log.pushLog(id, info.version, 'Force-stopping service\u2026', false)

      try {
        await invoke('force_stop_service', { id, version: info.version })
        this._updateStatus(id, 'Stopped')
      }
      catch (error) {
        const msg = error instanceof Error ? error.message : String(error)
        log.pushLog(id, info.version, `Force-stop failed: ${msg}`, true)
        console.error(`Failed to force-stop service ${id}`, error)
      }
      finally {
        this.loadingStates = { ...this.loadingStates, [id]: false }
      }
    },

    async fetchServicesStatus() {
      // Status is managed reactively by events from Rust.
    },
  },
})
