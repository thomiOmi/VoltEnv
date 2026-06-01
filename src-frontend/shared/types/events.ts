export interface DownloadProgressPayload {
  id: string
  progress: number
}

export interface InstallProgressPayload {
  id: string
  progress: number
}

export interface ServiceLogPayload {
  service_id: string
  version: string
  message: string
  timestamp: string
  is_error: boolean
}

export interface ServiceStatusChangedPayload {
  id: string
  status: string
}
