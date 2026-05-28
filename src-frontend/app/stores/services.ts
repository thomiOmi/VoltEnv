import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

interface ServiceInfo {
  id: string
  name: string
  port: number
}

const SERVICE_INFO: Record<string, ServiceInfo> = {
  nginx: { id: 'nginx', name: 'Nginx', port: 80 },
  php: { id: 'php', name: 'PHP-CGI', port: 9000 },
  mysql: { id: 'mysql', name: 'MySQL', port: 3306 },
}

export interface Service {
  id: string
  name: string
  status: 'Running' | 'Stopped' | 'Starting' | { Error: string }
  port: number
}

export const useServicesStore = defineStore('services', {
  state: () => ({
    services: Object.values(SERVICE_INFO).map(s => ({
      id: s.id,
      name: s.name,
      status: 'Stopped' as Service['status'],
      port: s.port,
    })),
    loadingStates: {} as Record<string, boolean>,
    downloadProgress: {} as Record<string, number>,
  }),

  actions: {
    init() {
      listen<{ id: string, status: string }>('service-status-changed', (event) => {
        const { id, status } = event.payload
        this._updateStatus(id, status as Service['status'])
      })

      listen<{ id: string, progress: number }>('provision-progress', (event) => {
        this.downloadProgress = {
          ...this.downloadProgress,
          [event.payload.id]: event.payload.progress,
        }
      })
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
      if (!info) {
        return
      }

      this.loadingStates = { ...this.loadingStates, [id]: true }
      this._updateStatus(id, 'Starting')

      try {
        await invoke('provision_service', { id })
        await invoke('start_service', { id })

        this._updateStatus(id, 'Running')
      }
      catch (error) {
        console.error(`Failed to start service ${id}`, error)
        this._updateStatus(id, 'Stopped')
      }
      finally {
        this.loadingStates = { ...this.loadingStates, [id]: false }
      }
    },

    async stopService(id: string) {
      const info = SERVICE_INFO[id]
      if (!info) {
        return
      }

      this.loadingStates = { ...this.loadingStates, [id]: true }

      try {
        await invoke('stop_service', { id })
        this._updateStatus(id, 'Stopped')
      }
      catch (error) {
        console.error(`Failed to stop service ${id}`, error)
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
