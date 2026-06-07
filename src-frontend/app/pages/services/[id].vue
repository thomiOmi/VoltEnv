<script setup lang="ts">
const route = useRoute()
const router = useRouter()
const id = computed(() => route.params.id as string)

const servicesStore = useServicesStore()

const def = computed(() => servicesStore.getDefinition(id.value))
const status = computed(() => servicesStore.getStatus(id.value))

async function handleStart() {
  try {
    await servicesStore.startService(id.value)
  }
  catch (e) {
    console.error('Start failed:', e)
  }
}

async function handleStop() {
  try {
    await servicesStore.stopService(id.value)
  }
  catch (e) {
    console.error('Stop failed:', e)
  }
}
</script>

<template>
  <UDashboardPanel>
    <template #header>
      <UDashboardNavbar>
        <template #left>
          <UButton
            color="neutral"
            variant="ghost"
            icon="i-lucide-arrow-left"
            @click="router.push('/')"
          />
          <span class="font-semibold">{{ def?.name ?? id }}</span>
        </template>

        <template #right>
          <div class="flex items-center gap-2">
            <UButton
              v-if="status?.status !== 'running'"
              color="primary"
              variant="solid"
              :loading="servicesStore.isLoading(id)"
              :disabled="servicesStore.isLoading(id)"
              @click="handleStart"
            >
              Start
            </UButton>
            <UButton
              v-else
              color="error"
              variant="soft"
              :loading="servicesStore.isLoading(id)"
              :disabled="servicesStore.isLoading(id)"
              @click="handleStop"
            >
              Stop
            </UButton>
          </div>
        </template>
      </UDashboardNavbar>
    </template>

    <template #body>
      <div v-if="!def" class="flex flex-col items-center justify-center h-full gap-2 text-muted">
        <UIcon name="i-lucide-search-x" class="size-12" />
        <p>Service "{{ id }}" not found</p>
      </div>

      <div v-else class="p-4 space-y-6">
        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
          <UCard>
            <template #header>
              <span class="text-sm font-medium text-muted">Status</span>
            </template>
            <div class="flex items-center gap-2">
              <UBadge
                :color="status?.status === 'running' ? 'success' : 'neutral'"
                variant="solid"
                size="sm"
              >
                {{ status?.status ?? 'stopped' }}
              </UBadge>
              <span v-if="status?.port" class="text-sm text-muted">
                Port {{ status.port }}
              </span>
            </div>
          </UCard>

          <UCard>
            <template #header>
              <span class="text-sm font-medium text-muted">Version</span>
            </template>
            <span class="text-lg font-semibold">{{ status?.version ?? def.defaultVersion }}</span>
          </UCard>

          <UCard>
            <template #header>
              <span class="text-sm font-medium text-muted">Binary</span>
            </template>
            <span class="text-sm font-mono">{{ def.binaryName }}</span>
          </UCard>
        </div>

        <UCard>
          <template #header>
            <span class="text-sm font-medium text-muted">Logs</span>
          </template>
          <LogConsole :service-id="id" :version="status?.version ?? def.defaultVersion" />
        </UCard>
      </div>
    </template>
  </UDashboardPanel>
</template>
