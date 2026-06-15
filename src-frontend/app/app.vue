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
  htmlAttrs: { lang: 'en' },
})

useSeoMeta({
  title: 'VoltEnv',
  description: 'Local Development Environment Manager',
})

const servicesStore = useServicesStore()
const logManager = useLogManagerStore()
const api = useServiceApi()

onMounted(async () => {
  await servicesStore.init()
  await logManager.startListening()

  // Auto-start groups
  try {
    const settings = await api.getSettings()
    if (settings) {
      const autoGroups = settings.autoStartGroups.filter(g => g.autoStart)
      for (const group of autoGroups) {
        for (const sid of group.services) {
          const def = servicesStore.getDefinition(sid)
          if (!def || servicesStore.isRunning(sid)) continue
          try {
            await servicesStore.setupService(sid, def.defaultVersion)
          } catch { continue }
          servicesStore.startService(sid).catch(() => {})
        }
      }
    }
  } catch (e) {
    console.warn('[voltenv] Auto-start check failed:', e)
  }
})

onUnmounted(() => {
  servicesStore.disposeListeners()
  logManager.stopListening()
})
</script>

<template>
  <UApp>
    <NuxtLoadingIndicator />
    <NuxtLayout>
      <NuxtPage />
    </NuxtLayout>
  </UApp>
</template>
