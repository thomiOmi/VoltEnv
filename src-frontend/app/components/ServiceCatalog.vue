<script setup lang="ts">
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { DownloadProgressPayload, InstallProgressPayload, ServiceStatusChangedPayload } from '#shared/types/events'

const servicesStore = useServicesStore()

const downloadProgress = ref<Record<string, number>>({})
const installProgress = ref<Record<string, number>>({})
const activeSteps = ref<Record<string, 'idle' | 'downloading' | 'installing' | 'ready'>>({})

const catalogItems = computed(() => servicesStore.catalog)

const isInstalled = (item: ServiceInfo) => {
  return (item.installedVersions?.length ?? 0) > 0
}

const isDownloading = (id: string) => {
  return activeSteps.value[id] === 'downloading'
}

const isInstalling = (id: string) => {
  return activeSteps.value[id] === 'installing'
}

const progress = (id: string) => {
  if (isDownloading(id)) return downloadProgress.value[id] ?? 0
  if (isInstalling(id)) return installProgress.value[id] ?? 0
  return 0
}

const progressLabel = (id: string) => {
  if (isDownloading(id)) return 'Downloading…'
  if (isInstalling(id)) return 'Installing…'
  return ''
}

const setup = async (info: ServiceInfo) => {
  await servicesStore.provisionService(info.id, info.version)
}

let unlistenDownload: UnlistenFn | null = null
let unlistenInstall: UnlistenFn | null = null
let unlistenStatusChanged: UnlistenFn | null = null

onMounted(async () => {
  // Reconcile disk state immediately on mount so the catalog badge and
  // Setup / Start buttons reflect the actual filesystem.
  await servicesStore.fetchServicesStatus()

  listen<DownloadProgressPayload>('download-progress', (event) => {
    const { id, progress: pct } = event.payload
    activeSteps.value = { ...activeSteps.value, [id]: 'downloading' }
    downloadProgress.value = { ...downloadProgress.value, [id]: pct }
  }).then((fn) => {
    unlistenDownload = fn
  })

  listen<InstallProgressPayload>('install-progress', (event) => {
    const { id, progress: pct } = event.payload
    activeSteps.value = { ...activeSteps.value, [id]: 'installing' }
    installProgress.value = { ...installProgress.value, [id]: pct }
    if (pct >= 100) {
      setTimeout(() => {
        activeSteps.value = { ...activeSteps.value, [id]: 'ready' }
      }, 500)
    }
  }).then((fn) => {
    unlistenInstall = fn
  })

  listen<ServiceStatusChangedPayload>('service-status-changed', async () => {
    await servicesStore.fetchServicesStatus()
  }).then((fn) => {
    unlistenStatusChanged = fn
  })
})

onUnmounted(() => {
  unlistenDownload?.()
  unlistenInstall?.()
  unlistenStatusChanged?.()
})
</script>

<template>
  <UCard variant="outline">
    <template #header>
      <div class="flex items-center justify-between">
        <span class="font-semibold text-sm text-highlighted">Service Catalog</span>
        <span class="text-xs text-muted">{{ catalogItems.length }} available</span>
      </div>
    </template>

    <div class="divide-y divide-default">
      <div
        v-for="item in catalogItems"
        :key="item.id"
        class="py-3 first:pt-0 last:pb-0"
      >
        <div class="flex items-center justify-between gap-4">
          <div class="min-w-0 flex-1">
            <div class="flex items-center gap-2">
              <span class="text-sm font-medium text-highlighted truncate">
                {{ item.name }}
              </span>
              <UBadge
                v-if="isInstalled(item)"
                color="success"
                variant="subtle"
                size="xs"
              >
                Installed
              </UBadge>
              <span
                v-else
                class="text-dimmed text-xs"
              >
                Not installed
              </span>
            </div>

            <div class="mt-0.5 flex items-center gap-3 text-xs text-muted">
              <span class="font-mono">{{ item.version }}</span>
              <span v-if="item.port > 0" class="font-mono">Port {{ item.port }}</span>
              <span v-if="item.versions && item.versions.length > 1">
                {{ item.versions.length }} versions
              </span>
            </div>

            <div
              v-if="isDownloading(item.id) || isInstalling(item.id)"
              class="mt-2"
            >
              <div class="flex items-center justify-between text-xs mb-1">
                <span class="text-muted">{{ progressLabel(item.id) }}</span>
                <span class="text-dimmed font-mono">{{ Math.round(progress(item.id)) }}%</span>
              </div>
              <UProgress
                :model-value="progress(item.id)"
                color="primary"
                size="sm"
                animation="carousel"
              />
            </div>
          </div>

          <UButton
            v-if="!isInstalled(item) && !isDownloading(item.id) && !isInstalling(item.id)"
            color="primary"
            variant="soft"
            size="sm"
            icon="i-lucide-download"
            :loading="servicesStore.loadingStates[item.id]"
            @click="setup(item)"
          >
            Setup
          </UButton>
        </div>
      </div>
    </div>
  </UCard>
</template>
