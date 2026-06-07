import { listen } from '@tauri-apps/api/event'
import type { UnlistenFn } from '@tauri-apps/api/event'
import type { ServiceLogPayload } from '#shared/types/events'

interface LogEntry {
  message: string
  timestamp: string
  isError: boolean
}

interface SystemLogEntry {
  level: 'info' | 'warn' | 'error'
  message: string
  timestamp: string
}

const MAX_LOGS_PER_INSTANCE = 1000
const MAX_SYSTEM_LOGS = 500

export const useLogManagerStore = defineStore('logManager', () => {
  const logs = ref<Record<string, LogEntry[]>>({})
  const systemLogs = ref<SystemLogEntry[]>([])

  let _unlistenLog: UnlistenFn | null = null
  let _unlistenSystem: UnlistenFn | null = null
  const _unlistenLogService: Map<string, UnlistenFn> = new Map()

  function _key(serviceId: string, version: string): string {
    return `${serviceId}:${version}`
  }

  function _appendLog(key: string, message: string, isError: boolean) {
    if (!logs.value[key]) {
      logs.value[key] = []
    }
    const arr = logs.value[key]
    arr.push({
      message,
      timestamp: new Date().toISOString(),
      isError,
    })
    if (arr.length > MAX_LOGS_PER_INSTANCE) {
      arr.splice(0, arr.length - MAX_LOGS_PER_INSTANCE)
    }
  }

  async function startListening() {
    _unlistenLog = await listen<ServiceLogPayload>('service-log', (event) => {
      const { service_id, version, message, is_error } = event.payload
      _appendLog(_key(service_id, version), message, is_error)
    })

    _unlistenSystem = await listen<SystemLogEntry>('system-log', (event) => {
      systemLogs.value.push(event.payload)
      if (systemLogs.value.length > MAX_SYSTEM_LOGS) {
        systemLogs.value.splice(0, systemLogs.value.length - MAX_SYSTEM_LOGS)
      }
    })
  }

  function stopListening() {
    _unlistenLog?.()
    _unlistenLog = null
    _unlistenSystem?.()
    _unlistenSystem = null
    _unlistenLogService.forEach(fn => fn())
    _unlistenLogService.clear()
  }

  function pushLog(serviceId: string, version: string, message: string, isError: boolean) {
    _appendLog(_key(serviceId, version), message, isError)
  }

  function clearLogs(serviceId: string, version: string) {
    const k = _key(serviceId, version)
    if (logs.value[k]) {
      logs.value[k] = []
    }
  }

  function getLogs(serviceId: string, version: string): LogEntry[] {
    return logs.value[_key(serviceId, version)] ?? []
  }

  return {
    logs,
    systemLogs,
    startListening,
    stopListening,
    pushLog,
    clearLogs,
    getLogs,
  }
})
