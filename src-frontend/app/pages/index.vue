<script setup lang="ts">
definePageMeta({
  title: 'Services',
})

const servicesStore = useServicesStore()
</script>

<template>
  <UDashboardPanel id="services">
    <template #header>
      <UDashboardNavbar title="Services">
        <template #leading>
          <UDashboardSidebarCollapse />
        </template>

        <template #right>
          <UButton
            color="neutral"
            variant="ghost"
            square
            icon="i-lucide-refresh-cw"
            @click="servicesStore.fetchServicesStatus()"
          />
        </template>
      </UDashboardNavbar>

      <UDashboardToolbar>
        <template #left>
          <div class="flex items-center gap-6 text-sm">
            <div class="flex items-center gap-2">
              <span class="text-muted">Total</span>
              <span class="font-semibold text-highlighted">{{ servicesStore.services.length }}</span>
            </div>
            <div class="flex items-center gap-2">
              <span class="size-2 rounded-full bg-success" />
              <span class="text-muted">Running</span>
              <span class="font-semibold text-highlighted">{{ servicesStore.services.filter(s => s.status === 'Running').length }}</span>
            </div>
            <div class="flex items-center gap-2">
              <span class="size-2 rounded-full bg-error" />
              <span class="text-muted">Stopped</span>
              <span class="font-semibold text-highlighted">{{ servicesStore.services.filter(s => s.status === 'Stopped').length }}</span>
            </div>
          </div>
        </template>
      </UDashboardToolbar>
    </template>

    <template #body>
      <!-- Service Cards -->
      <div class="grid grid-cols-1 md:grid-cols-2 gap-6 p-6">
        <ServiceCard
          v-for="service in servicesStore.services"
          :key="service.id"
          :service="service"
        />
      </div>

      <!-- Logs Section -->
      <div class="px-6 pb-6">
        <h2 class="text-sm font-semibold text-muted uppercase tracking-wider mb-4">
          Service Logs
        </h2>
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-4">
          <LogConsole
            v-for="service in servicesStore.services"
            :key="service.id"
            :service-id="service.id"
            :version="service.version"
          />
        </div>
      </div>
    </template>

    <template #footer>
      <div class="text-center py-3">
        <p class="text-xs text-dimmed">
          VoltEnv v0.1.0 &mdash; Local Development Environment Manager
        </p>
      </div>
    </template>
  </UDashboardPanel>
</template>
