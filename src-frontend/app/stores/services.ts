import { defineStore } from 'pinia'
import { invoke } from '@tauri-apps/api/core'

export interface Service {
  id: string
  name: string
  status: 'Running' | 'Stopped' | 'Starting' | { Error: string }
  port: number
}

export const useServicesStore = defineStore('services', {
  state: () => ({
    services: [] as Service[],
    loadingStates: {} as Record<string, boolean>,
  }),

  actions: {
    async fetchServicesStatus() {
      try {
        this.services = await invoke<Service[]>('get_services_status')
      }
      catch (error) {
        console.error('Failed to fetch services status', error)
      }
    },

    async startService(id: string) {
      this.loadingStates = { ...this.loadingStates, [id]: true }
      try {
        await invoke('start_service', { id })
        await this.fetchServicesStatus()
      }
      catch (error) {
        console.error(`Failed to start service ${id}`, error)
      }
      finally {
        this.loadingStates = { ...this.loadingStates, [id]: false }
      }
    },

    async stopService(id: string) {
      this.loadingStates = { ...this.loadingStates, [id]: true }
      try {
        await invoke('stop_service', { id })
        await this.fetchServicesStatus()
      }
      catch (error) {
        console.error(`Failed to stop service ${id}`, error)
      }
      finally {
        this.loadingStates = { ...this.loadingStates, [id]: false }
      }
    },
  },
})
