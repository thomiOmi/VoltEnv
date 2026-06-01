import { defineStore } from 'pinia'

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
  }),

  actions: {
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
