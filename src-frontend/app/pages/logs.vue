<script setup lang="ts">
definePageMeta({
  title: 'Logs',
})

const servicesStore = useServicesStore()
const logManager = useLogManagerStore()

const selectedService = ref<string | undefined>(undefined)
const selectedVersion = computed(() => {
  if (!selectedService.value) {
    return ''
  }
  const s = servicesStore.getStatus(selectedService.value)
  return s?.version ?? ''
})

const container = ref<HTMLElement | null>(null)

watch(
  () => logManager.systemLogs.length,
  async () => {
    await nextTick()
    if (container.value) {
      container.value.scrollTop = container.value.scrollHeight
    }
  },
)
</script>

<template>
  <UDashboardPanel>
    <template #header>
      <UDashboardNavbar title="Logs" />
    </template>

    <template #body>
      <div class="p-4 space-y-6">
        <UCard>
          <template #header>
            <div class="flex items-center justify-between">
              <span class="text-sm font-medium text-muted">System Log</span>
              <span class="text-xs text-muted">{{ logManager.systemLogs.length }} entries</span>
            </div>
          </template>
          <div
            ref="container"
            class="overflow-y-auto p-3 font-mono text-xs leading-relaxed space-y-0.5"
            style="max-height: 400px; min-height: 200px;"
          >
            <div v-if="logManager.systemLogs.length === 0" class="text-dimmed italic">
              No system logs yet.
            </div>
            <div
              v-for="(log, idx) in logManager.systemLogs"
              :key="idx"
              class="whitespace-pre-wrap break-all"
              :class="log.level === 'error' ? 'text-error' : log.level === 'warn' ? 'text-warning' : 'text-muted'"
            >
              <span class="text-dimmed">[{{ log.timestamp }}]</span>
              <span class="font-semibold mx-1 uppercase text-[10px]">{{ log.level }}</span>
              {{ log.message }}
            </div>
          </div>
        </UCard>

        <UCard>
          <template #header>
            <div class="flex items-center justify-between">
              <span class="text-sm font-medium text-muted">Service Logs</span>
              <USelect
                v-model="selectedService"
                :items="servicesStore.allDefinitions.map(d => ({ label: d.name, value: d.id }))"
                placeholder="Select a service"
                size="sm"
                class="w-48"
              />
            </div>
          </template>
          <div v-if="!selectedService" class="text-sm text-muted py-2">
            Select a service to view its logs.
          </div>
          <LogConsole v-else :service-id="selectedService!" :version="selectedVersion" />
        </UCard>
      </div>
    </template>
  </UDashboardPanel>
</template>
