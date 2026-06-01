import { defineStore } from 'pinia'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { ServiceStatusChangedPayload } from '#shared/types/events'

export const useServicesStore = defineStore('services', {
  state: () => ({
    services: [] as Service[],
    catalog: [] as ServiceInfo[],
    loadingStates: {} as Record<string, boolean>,
    downloadProgress: {} as Record<string, number>,
    installProgress: {} as Record<string, number>,
    serviceStep: {} as Record<string, 'idle' | 'downloading' | 'installing' | 'ready'>,
    portError: null as string | null,
    activeVersions: {} as Record<string, string>,
    switchingVersions: {} as Record<string, boolean>,
    _unlistenStatus: null as UnlistenFn | null,
  }),

  getters: {
    isActiveVersion: (state) => {
      return (id: string, version: string): boolean => {
        return state.activeVersions[id] === version
      }
    },
    versionsFor: (state) => {
      return (id: string): string[] => {
        return state.catalog.find(s => s.id === id)?.versions ?? []
      }
    },
  },

  actions: {
    async init() {
      await this.fetchServicesStatus()

      listen<ServiceStatusChangedPayload>('service-status-changed', async () => {
        await this.fetchServicesStatus()
      }).then((unlisten) => {
        this._unlistenStatus = unlisten
      })
    },

    disposeListeners() {
      this._unlistenStatus?.()
      this._unlistenStatus = null
    },

    /// Unified re-sync with the filesystem.
    ///
    /// 1. Re-fetches the catalog — the backend calls
    ///    `catalog::scan_installed_versions` which scans `bin/` on disk,
    ///    making the filesystem the single source of truth for
    ///    `installedVersions` and purging any stale active version from
    ///    the in-memory cache.
    /// 2. Rebuilds the entire `services` array from the fresh catalog so
    ///    stale service objects (orphaned after manual directory deletion)
    ///    are automatically removed.
    /// 3. Refreshes the OS PATH active-versions map.
    async fetchServicesStatus() {
      const api = useServiceApi()
      this.catalog = await api.getServices()
      this.services = await Promise.all(
        this.catalog.map(svc => api.getServiceStatus(svc.id)),
      )
      await this.fetchActiveVersions()
    },

    /// Downloads a custom template manifest for a service.
    /// Each asset in the manifest is downloaded, verified, and extracted
    /// under `$ROOT/bin/{id}/`.
    async downloadTemplates(id: string, manifest: DownloadManifest) {
      const api = useServiceApi()
      this.loadingStates = { ...this.loadingStates, [id]: true }
      const log = useLogManagerStore()
      log.pushLog(id, '', 'Downloading template assets…', false)

      try {
        await api.downloadTemplateService(id, manifest)
        await this.fetchServicesStatus()
        log.pushLog(id, '', 'Templates downloaded successfully', false)
      }
      catch (error) {
        const msg = error instanceof Error ? error.message : String(error)
        log.pushLog(id, '', `Template download failed: ${msg}`, true)
        console.error(`Failed to download templates for ${id}`, error)
      }
      finally {
        this.loadingStates = { ...this.loadingStates, [id]: false }
      }
    },

    async fetchActiveVersions() {
      const api = useServiceApi()
      try {
        this.activeVersions = await api.getActiveVersions()
      }
      catch (e) {
        console.error('Failed to fetch active versions', e)
      }
    },

    async switchServiceVersion(id: string, version: string) {
      const api = useServiceApi()
      this.switchingVersions = { ...this.switchingVersions, [id]: true }

      const log = useLogManagerStore()
      log.pushLog(id, version, `Switching to version ${version}…`, false)

      try {
        await api.switchServiceVersion(id, version)
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

    /// First-time setup: download + install + register to OS PATH.
    /// Does NOT start the service — use `startService` for that.
    async provisionService(id: string, version: string) {
      const api = useServiceApi()
      const info = this.catalog.find(s => s.id === id)
      if (!info) return

      this.loadingStates = { ...this.loadingStates, [id]: true }
      this._updateStatus(id, 'Starting')

      const log = useLogManagerStore()
      log.pushLog(id, version, 'Provisioning service…', false)

      try {
        if (!info.downloadUrl) {
          log.pushLog(id, version, 'No download URL configured — cannot provision', true)
          this._updateStatus(id, 'Stopped')
          return
        }

        await api.downloadService(id, version)
        await api.installService(id, version)
        await api.registerToOsEnv()
        await api.registerServiceEnvironment(id, version)
        this.activeVersions = { ...this.activeVersions, [id]: version }

        // Full re-sync — backend re-scans disk, frontend gets fresh
        // catalog (installedVersions) and statuses.
        await this.fetchServicesStatus()

        log.pushLog(id, version, 'Service provisioned successfully', false)
        this._updateStatus(id, 'Stopped')
      }
      catch (error) {
        const msg = error instanceof Error ? error.message : String(error)
        log.pushLog(id, version, `Provisioning failed: ${msg}`, true)
        console.error(`Failed to provision service ${id}`, error)
        this._updateStatus(id, 'Stopped')
      }
      finally {
        this.loadingStates = { ...this.loadingStates, [id]: false }
      }
    },

    /// Starts an already-provisioned service. Binary must exist on disk.
    async startService(id: string) {
      const api = useServiceApi()
      const info = this.catalog.find(s => s.id === id)
      if (!info) return

      const hasInstall = (info.installedVersions?.length ?? 0) > 0
      if (!hasInstall) {
        const log = useLogManagerStore()
        log.pushLog(id, info.version, 'Service not installed — use Setup first', true)
        return
      }

      const active = await api.getActiveVersions()
      const ver = active[id] ?? info.version

      this.loadingStates = { ...this.loadingStates, [id]: true }
      this._updateStatus(id, 'Starting')

      const log = useLogManagerStore()
      log.pushLog(id, ver, 'Starting service…', false)

      try {
        if (info.port > 0) {
          const available = await api.isPortAvailable(info.port)
          if (!available) {
            this.portError = `Port ${info.port} (${info.name}) is already in use. Please free the port and try again.`
            log.pushLog(id, ver, `Port ${info.port} is already in use`, true)
            this._updateStatus(id, 'Stopped')
            return
          }
          this.portError = null
        }

        await api.startService(id, ver)
        log.pushLog(id, ver, 'Service started', false)
      }
      catch (error) {
        const msg = error instanceof Error ? error.message : String(error)
        log.pushLog(id, ver, `Failed: ${msg}`, true)
        console.error(`Failed to start service ${id}`, error)
        this._updateStatus(id, 'Stopped')
      }
      finally {
        this.loadingStates = { ...this.loadingStates, [id]: false }
      }
    },

    async stopService(id: string) {
      const api = useServiceApi()
      const info = this.catalog.find(s => s.id === id)
      if (!info) return

      this.loadingStates = { ...this.loadingStates, [id]: true }

      const log = useLogManagerStore()
      log.pushLog(id, info.version, 'Stopping service…', false)

      try {
        await api.stopService(id)
        await api.unregisterServiceEnvironment(id)
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
      const api = useServiceApi()
      try {
        await api.softStopService(id)
      }
      catch (error) {
        console.error(`Failed to soft-stop service ${id}`, error)
      }
    },

    async forceStopService(id: string) {
      const api = useServiceApi()
      const info = this.catalog.find(s => s.id === id)
      if (!info) return

      this.loadingStates = { ...this.loadingStates, [id]: true }

      const log = useLogManagerStore()
      log.pushLog(id, info.version, 'Force-stopping service…', false)

      try {
        await api.forceStopService(id)
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

  },
})
