import { defineStore } from 'pinia'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

export interface LogPayload {
  service_id: string
  version: string
  message: string
  timestamp: string
  is_error: boolean
}

const MAX_LOGS_PER_INSTANCE = 1000

function stamp(): string {
  return new Date().toISOString()
}

export const useLogManagerStore = defineStore('logManager', {
  state: () => ({
    logs: {} as Record<string, LogPayload[]>,
    _listenerInitialized: false,
    _unlisten: null as UnlistenFn | null,
  }),

  actions: {
    initLogListener() {
      if (this._listenerInitialized) return
      this._listenerInitialized = true

      listen<LogPayload>('service-log', (event) => {
        const { service_id, version, message, timestamp, is_error } = event.payload
        this._appendLog(service_id, version, message, is_error, timestamp)
      }).then((unlisten) => {
        this._unlisten = unlisten
      })

      listen<{ id: string, status: string }>('service-status-changed', (event) => {
        const { id, status } = event.payload
        const svc = useServicesStore().services.find(s => s.id === id)
        const version = svc?.version ?? 'unknown'
        const message = status === 'Stopped' ? 'Service stopped' : `Status changed to ${status}`
        this._appendLog(id, version, message, status === 'Stopped', stamp())
      })
    },

    pushLog(serviceId: string, version: string, message: string, isError = false) {
      this._appendLog(serviceId, version, message, isError, stamp())
    },

    clearLogs(serviceId: string, version: string) {
      const key = `${serviceId}:${version}`
      this.logs = Object.fromEntries(
        Object.entries(this.logs).filter(([k]) => k !== key),
      )
    },

    _appendLog(
      serviceId: string,
      version: string,
      message: string,
      isError: boolean,
      timestamp: string,
    ) {
      const key = `${serviceId}:${version}`
      const current = this.logs[key] ?? []
      this.logs = {
        ...this.logs,
        [key]: [
          ...current.slice(-(MAX_LOGS_PER_INSTANCE - 1)),
          { service_id: serviceId, version, message, timestamp, is_error: isError },
        ],
      }
    },
  },
})
