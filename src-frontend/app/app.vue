<script setup lang="ts">
import { useServiceStore } from '~/stores/services'

const store = useServiceStore()

onMounted(() => {
  store.initFromStorage()
  store.refreshStatus()

  // Auto refresh status setiap 5 detik
  const interval = setInterval(() => {
    store.refreshStatus()
  }, 5000)

  onUnmounted(() => clearInterval(interval))
})

const getStatusColor = (status: any) => {
  if (status === 'Running') return 'success'
  if (status === 'Stopped') return 'neutral'
  if (typeof status === 'object' && 'Error' in status) return 'error'
  return 'warning'
}
</script>

<template>
  <UApp>
    <UContainer class="py-10">
      <header class="mb-10 flex justify-between items-center">
        <div>
          <h1 class="text-3xl font-bold text-gray-900 dark:text-white">VoltEnv</h1>
          <p class="text-gray-500 dark:text-gray-400">Manage your local development services</p>
        </div>
        <UButton
          icon="i-heroicons-arrow-path"
          color="neutral"
          variant="ghost"
          @click="store.refreshStatus()"
        />
      </header>

      <div class="grid grid-cols-1 md:grid-cols-3 gap-6">
        <UCard v-for="service in store.services" :key="service.id">
          <template #header>
            <div class="flex justify-between items-center">
              <span class="font-semibold text-lg">{{ service.name }}</span>
              <UBadge :color="getStatusColor(service.status)" variant="subtle">
                {{ typeof service.status === 'string' ? service.status : 'Error' }}
              </UBadge>
            </div>
          </template>

          <div class="space-y-2">
            <div class="flex justify-between text-sm">
              <span class="text-gray-500">Port:</span>
              <span class="font-mono">{{ service.port }}</span>
            </div>
          </div>

          <template #footer>
            <div class="flex gap-2 justify-end">
              <UButton
                v-if="service.status === 'Stopped'"
                icon="i-heroicons-play"
                color="success"
                variant="soft"
                size="sm"
                @click="store.startService(service.id)"
              >
                Start
              </UButton>
              <UButton
                v-if="service.status === 'Running'"
                icon="i-heroicons-stop"
                color="error"
                variant="soft"
                size="sm"
                @click="store.stopService(service.id)"
              >
                Stop
              </UButton>
            </div>
          </template>
        </UCard>
      </div>

      <footer class="mt-20 pt-10 border-t border-gray-200 dark:border-gray-800 text-center text-sm text-gray-500">
        VoltEnv v0.1.0 - Blazing Fast Local Dev
      </footer>
    </UContainer>
  </UApp>
</template>

<style>
body {
  background-color: #f9fafb;
}
.dark body {
  background-color: #030712;
}
</style>
