import { defineStore } from 'pinia'
import { fetch } from '@tauri-apps/plugin-http'
import { mkdir, writeFile, exists } from '@tauri-apps/plugin-fs'
import { Command } from '@tauri-apps/plugin-shell'
import { appLocalDataDir } from '@tauri-apps/api/path'
import JSZip from 'jszip'

interface ServiceDefinition {
  id: string
  name: string
  port: number
  version: string
  downloadUrl: string
  startArgs: string[]
  stopArgs: string[]
}

const SERVICE_CATALOG: Record<string, ServiceDefinition> = {
  nginx: {
    id: 'nginx',
    name: 'Nginx',
    port: 80,
    version: 'nginx-1.26.2',
    downloadUrl: 'https://nginx.org/download/nginx-1.26.2.zip',
    startArgs: ['-c', 'conf/nginx.conf'],
    stopArgs: ['-s', 'stop'],
  },
  php: {
    id: 'php',
    name: 'PHP-CGI',
    port: 9000,
    version: 'unknown',
    downloadUrl: '',
    startArgs: [],
    stopArgs: [],
  },
  mysql: {
    id: 'mysql',
    name: 'MySQL',
    port: 3306,
    version: 'unknown',
    downloadUrl: '',
    startArgs: [],
    stopArgs: [],
  },
}

async function getBinDir(def: ServiceDefinition): Promise<string> {
  const dataDir = await appLocalDataDir()
  return `${dataDir}bin\\${def.id}\\${def.version}\\`
}

async function getBinPath(def: ServiceDefinition): Promise<string> {
  const dir = await getBinDir(def)
  return `${dir}${def.id}.exe`
}

export interface Service {
  id: string
  name: string
  status: 'Running' | 'Stopped' | 'Starting' | { Error: string }
  port: number
}

export const useServicesStore = defineStore('services', {
  state: () => ({
    services: Object.values(SERVICE_CATALOG).map(s => ({
      id: s.id,
      name: s.name,
      status: 'Stopped' as Service['status'],
      port: s.port,
    })),
    loadingStates: {} as Record<string, boolean>,
    downloadProgress: {} as Record<string, number>,
    _children: {} as Record<string, Awaited<ReturnType<typeof Command.prototype.spawn>>>,
  }),

  actions: {
    init() {
      // Status is managed reactively — no listener needed.
    },

    _updateStatus(id: string, status: Service['status']) {
      const idx = this.services.findIndex(s => s.id === id)
      const service = idx !== -1 ? this.services[idx] : undefined
      if (service) {
        service.status = status
      }
    },

    async _ensureProvisioned(id: string, def: ServiceDefinition): Promise<void> {
      const binPath = await getBinPath(def)
      if (await exists(binPath)) {
        return
      }

      this.downloadProgress = { ...this.downloadProgress, [id]: 0 }

      const response = await fetch(def.downloadUrl)
      if (!response.ok) {
        throw new Error(`Download failed: ${response.status}`)
      }

      const total = Number(response.headers.get('content-length') || '0')
      const binDir = await getBinDir(def)
      await mkdir(binDir, { recursive: true })

      const chunks: Uint8Array[] = []
      let downloaded = 0

      if (response.body) {
        const reader = response.body.getReader()
        while (true) {
          const { done, value } = await reader.read()
          if (done) break
          chunks.push(value)
          downloaded += value.length
          if (total > 0) {
            this.downloadProgress = {
              ...this.downloadProgress,
              [id]: Math.round((downloaded / total) * 50),
            }
          }
        }
      }
      else {
        const buffer = await response.arrayBuffer()
        chunks.push(new Uint8Array(buffer))
        this.downloadProgress = { ...this.downloadProgress, [id]: 50 }
      }

      // Concatenate all chunks into a single buffer
      const totalLen = chunks.reduce((s, c) => s + c.length, 0)
      const full = new Uint8Array(totalLen)
      let offset = 0
      for (const chunk of chunks) {
        full.set(chunk, offset)
        offset += chunk.length
      }

      // Extract zip
      const zip = await JSZip.loadAsync(full)
      const zipRoot = Object.keys(zip.files).find(k => k.includes('/'))?.split('/')[0] || ''
      const entries = Object.entries(zip.files)
      let extracted = 0

      for (const [path, file] of entries) {
        const relativePath = zipRoot ? path.slice(zipRoot.length + 1) : path
        if (!relativePath) continue

        const targetPath = `${binDir}${relativePath.replace(/\//g, '\\')}`

        if (file.dir) {
          await mkdir(targetPath, { recursive: true })
        }
        else {
          const parent = targetPath.substring(0, targetPath.lastIndexOf('\\'))
          if (parent) {
            await mkdir(parent, { recursive: true })
          }
          const content = await file.async('uint8array')
          await writeFile(targetPath, content)
        }

        extracted++
        this.downloadProgress = {
          ...this.downloadProgress,
          [id]: 50 + Math.round((extracted / entries.length) * 50),
        }
      }
    },

    async startService(id: string) {
      const def = SERVICE_CATALOG[id]
      if (!def) return

      this.loadingStates = { ...this.loadingStates, [id]: true }
      this._updateStatus(id, 'Starting')

      try {
        await this._ensureProvisioned(id, def)

        const binPath = await getBinPath(def)
        const binDir = await getBinDir(def)

        const command = Command.create(binPath, def.startArgs, { cwd: binDir })
        const child = await command.spawn()
        this._children = { ...this._children, [id]: child }
        this._updateStatus(id, 'Running')
      }
      catch (error) {
        console.error(`Failed to start service ${id}`, error)
        this._updateStatus(id, 'Stopped')
      }
      finally {
        this.loadingStates = { ...this.loadingStates, [id]: false }
      }
    },

    async stopService(id: string) {
      const def = SERVICE_CATALOG[id]
      if (!def) return

      this.loadingStates = { ...this.loadingStates, [id]: true }

      try {
        // Graceful stop via the service binary itself
        if (def.stopArgs.length > 0) {
          const binPath = await getBinPath(def)
          const binDir = await getBinDir(def)
          const cmd = Command.create(binPath, def.stopArgs, { cwd: binDir })
          cmd.spawn().catch(() => {})
        }

        // Force-kill tracked child
        const child = this._children[id]
        if (child) {
          try {
            await child.kill()
          }
          catch { /* already dead */ }
        }

        const { [id]: _, ...rest } = this._children
        this._children = rest
        this._updateStatus(id, 'Stopped')
      }
      catch (error) {
        console.error(`Failed to stop service ${id}`, error)
      }
      finally {
        this.loadingStates = { ...this.loadingStates, [id]: false }
      }
    },

    async fetchServicesStatus() {
      // Status is managed reactively by startService / stopService.
    },
  },
})
