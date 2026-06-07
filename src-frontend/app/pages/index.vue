<script setup lang="ts">
definePageMeta({
  title: 'Services',
})

const servicesStore = useServicesStore()

const runningCount = computed(() =>
  Array.from(servicesStore.statuses.values()).filter(s => s.status === 'running').length,
)

const stoppedCount = computed(() =>
  Array.from(servicesStore.statuses.values()).filter(s => s.status === 'stopped' || !s.status).length,
)

async function refresh() {
  await servicesStore.fetchDefinitions()
}
</script>

<template>
  <UDashboardPanel>
    <template #header>
      <UDashboardNavbar title="Services">
        <template #right>
          <UButton
            color="neutral"
            variant="ghost"
            icon="i-lucide-refresh-cw"
            aria-label="Refresh services"
            @click="refresh"
          />
        </template>
      </UDashboardNavbar>
    </template>

    <template #body>
      <div class="p-4 space-y-4">
        <div class="flex items-center gap-4 text-sm text-muted">
          <span>Total: <strong class="text-default">{{ servicesStore.definitions.size }}</strong></span>
          <span>Running: <strong class="text-success">{{ runningCount }}</strong></span>
          <span>Stopped: <strong class="text-neutral">{{ stoppedCount }}</strong></span>
        </div>

        <ServiceTable />

        <ServiceCatalog />
      </div>
    </template>

    <template #footer>
      <div class="text-center text-xs text-muted py-2">
        VoltEnv v0.1.0
      </div>
    </template>
  </UDashboardPanel>
</template>
