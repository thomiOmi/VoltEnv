export interface ServiceInfo {
  id: string
  name: string
  port: number
  version: string
  versions?: string[]
  /** Subset of `versions` that are actually installed on disk. */
  installedVersions?: string[]
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

export interface Asset {
  name: string
  url: string
  destinationSubdir: string
  sha256: string | null
  pgpSignatureUrl: string | null
  extract: boolean
}

export interface DownloadManifest {
  assets: Asset[]
}
