export interface ServiceInfo {
  id: string
  name: string
  port: number
  version: string
  versions?: string[]
  downloadUrl: string
  sha256?: string
  pgpSignatureUrl?: string
}

export interface Service {
  id: string
  name: string
  version: string
  status: 'Running' | 'Stopped' | 'Starting' | { Error: string }
  port: number
}
