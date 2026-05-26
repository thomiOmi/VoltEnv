import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

export interface Service {
  id: string
  name: string
  status: 'Running' | 'Stopped' | 'Starting' | { Error: string }
  port: number
}

export const useServiceStore = defineStore('services', {
  state: () => ({
    services: [] as Service[],
    loading: false,
    lastUpdate: Date.now()
  }),

  actions: {
    // Inisialisasi state dari localStorage secara manual
    initFromStorage() {
      const saved = localStorage.getItem('voltenv_services')
      if (saved) {
        try {
          this.services = JSON.parse(saved)
        } catch (e) {
          console.error('Gagal memuat state dari localStorage', e)
        }
      }
    },

    // Simpan state ke localStorage secara manual
    saveToStorage() {
      localStorage.setItem('voltenv_services', JSON.stringify(this.services))
      localStorage.setItem('voltenv_last_update', Date.now().toString())
    },

    async refreshStatus() {
      try {
        // @ts-ignore
        if (window.__TAURI_INTERNALS__) {
          const data = await invoke<Service[]>('get_services_status')
          this.services = data
        } else {
          // Mock data for development/verification outside Tauri
          this.services = [
            { id: 'nginx', name: 'Nginx', status: 'Stopped', port: 80 },
            { id: 'php', name: 'PHP-CGI', status: 'Running', port: 9000 },
            { id: 'mysql', name: 'MySQL', status: 'Stopped', port: 3306 },
          ]
        }
        this.saveToStorage()
      } catch (error) {
        console.error('Failed to fetch services status', error)
        // Fallback to mock on error (likely missing Tauri environment)
        this.services = [
          { id: 'nginx', name: 'Nginx', status: 'Stopped', port: 80 },
          { id: 'php', name: 'PHP-CGI', status: 'Running', port: 9000 },
          { id: 'mysql', name: 'MySQL', status: 'Stopped', port: 3306 },
        ]
      }
    },

    async startService(id: string) {
      try {
        await invoke('start_service', { id })
        await this.refreshStatus()
      } catch (error) {
        console.error(`Failed to start service ${id}`, error)
      }
    },

    async stopService(id: string) {
      try {
        await invoke('stop_service', { id })
        await this.refreshStatus()
      } catch (error) {
        console.error(`Failed to stop service ${id}`, error)
      }
    }
  }
})
