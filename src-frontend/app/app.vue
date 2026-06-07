<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window'

const colorMode = useColorMode()

const color = computed(() => colorMode.value === 'dark' ? '#0f172a' : 'white')

useHead({
  meta: [
    { charset: 'utf-8' },
    { name: 'viewport', content: 'width=device-width, initial-scale=1' },
    { key: 'theme-color', name: 'theme-color', content: color },
  ],
  htmlAttrs: {
    lang: 'en',
  },
})

const title = 'VoltEnv'
const description = 'Local Development Environment Manager'

useSeoMeta({
  title,
  description,
  ogTitle: title,
  ogDescription: description,
})

const servicesStore = useServicesStore()
const logManager = useLogManagerStore()
const shutdownOpen = ref(false)
const shuttingDown = ref(false)
const api = useServiceApi()

// Store unlisten refs so cleanup can be registered at setup level
let _closeUnlisten: (() => void) | null = null
onUnmounted(() => {
  servicesStore.disposeListeners()
  logManager.stopListening()
  _closeUnlisten?.()
})

onMounted(async () => {
  await servicesStore.init()
  await logManager.startListening()

  // Auto-start: first setup (idempotent for installed), then start
  try {
    const settings = await api.getSettings()
    if (settings) {
      const autoGroups = settings.autoStartGroups.filter(g => g.autoStart)
      for (const group of autoGroups) {
        for (const sid of group.services) {
          const def = servicesStore.getDefinition(sid)
          if (!def || servicesStore.isRunning(sid)) {
            continue
          }
          // Silently setup first — noop if already installed
          try {
            await servicesStore.setupService(sid, def.defaultVersion)
          }
          catch {
            // Setup failed (e.g. no download URL) — skip
            continue
          }
          servicesStore.startService(sid).catch((e: unknown) => {
            console.warn(`[voltenv] Auto-start failed for ${sid}:`, e)
          })
        }
      }
    }
  }
  catch (e) {
    console.warn('[voltenv] Auto-start check failed:', e)
  }

  _closeUnlisten = await getCurrentWindow().onCloseRequested(async (event) => {
    const running = servicesStore.allDefinitions.some(d => servicesStore.isRunning(d.id))
    if (!running) {
      return
    }
    event.preventDefault()
    shutdownOpen.value = true
  })
})

async function handleShutdown(action: 'stop' | 'keep') {
  shuttingDown.value = true
  if (action === 'stop') {
    const running = servicesStore.allDefinitions.filter(d => servicesStore.isRunning(d.id))
    await Promise.allSettled(running.map(d => servicesStore.stopService(d.id)))
  }
  shuttingDown.value = false
  shutdownOpen.value = false
  const win = getCurrentWindow()
  await win.close()
}
</script>

<template>
  <UApp>
    <NuxtLoadingIndicator />

    <NuxtLayout>
      <NuxtPage />
    </NuxtLayout>

    <UModal
      v-model:open="shutdownOpen"
      title="Shutdown"
      :dismissible="false"
      :close="false"
    >
      <template #body>
        <p class="text-sm text-muted">
          One or more services are still running. What would you like to do?
        </p>
      </template>

      <template #footer>
        <div class="flex items-center justify-end gap-2">
          <UButton
            color="neutral"
            variant="outline"
            :disabled="shuttingDown"
            @click="shutdownOpen = false"
          >
            Cancel
          </UButton>
          <UButton
            color="neutral"
            variant="subtle"
            :loading="shuttingDown"
            @click="handleShutdown('keep')"
          >
            Keep Running
          </UButton>
          <UButton color="error" :loading="shuttingDown" @click="handleShutdown('stop')">
            Stop All
          </UButton>
        </div>
      </template>
    </UModal>
  </UApp>
</template>
