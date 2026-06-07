export interface VersionInfo {
  downloadUrl: string
  sha256: string | null
}

export interface HealthCheckConfig {
  type: string
  command: string | null
  timeoutMs: number
}

export interface ServiceDefinition {
  id: string
  name: string
  kind: string
  defaultVersion: string
  versions: Record<string, VersionInfo>
  binaryName: string
  startArgs: string[]
  stopArgs: string[]
  port: number
  configTemplateName: string | null
  healthCheck: HealthCheckConfig | null
  postInstallCommands: string[]
}

export interface ServiceStatus {
  id: string
  version: string
  status: 'running' | 'stopped' | 'starting' | 'error'
  port: number
}
