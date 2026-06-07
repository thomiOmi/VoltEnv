export interface AutoStartGroup {
  name: string
  services: string[]
  autoStart: boolean
}

export interface Settings {
  preferredPorts: Record<string, number>
  resolvedPorts: Record<string, number>
  autoStartGroups: AutoStartGroup[]
  activeVersions: Record<string, string>
}
