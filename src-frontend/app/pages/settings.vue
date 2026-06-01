<script setup lang="ts">
definePageMeta({
  title: 'Settings',
})

const servicesStore = useServicesStore()
const api = useServiceApi()

const restoringBackup = ref(false)
const backupRestoreResult = ref<{ success: boolean, message: string } | null>(null)

async function restoreBackup() {
  restoringBackup.value = true
  backupRestoreResult.value = null
  try {
    await api.restoreOsPathBackup()
    backupRestoreResult.value = { success: true, message: 'PATH backup restored successfully.' }
  }
  catch (err) {
    backupRestoreResult.value = { success: false, message: String(err) }
  }
  finally {
    restoringBackup.value = false
  }
}
</script>

<template>
  <UDashboardPanel id="settings">
    <template #header>
      <UDashboardNavbar title="Settings">
        <template #leading>
          <UDashboardSidebarCollapse />
        </template>
      </UDashboardNavbar>
    </template>

    <template #body>
      <div class="max-w-2xl mx-auto w-full p-6 space-y-8">
        <!-- Active Versions -->
        <UCard variant="outline">
          <template #header>
            <span class="font-semibold text-sm text-highlighted">Active OS Versions</span>
          </template>

          <div class="divide-y divide-default">
            <div
              v-for="(version, id) in servicesStore.activeVersions"
              :key="id"
              class="flex items-center justify-between py-2 first:pt-0 last:pb-0"
            >
              <span class="text-sm text-default capitalize">{{ id }}</span>
              <span class="text-sm font-mono text-muted">{{ version }}</span>
            </div>
            <div v-if="Object.keys(servicesStore.activeVersions).length === 0" class="text-sm text-dimmed py-2">
              No active versions registered.
            </div>
          </div>
        </UCard>

        <!-- PATH Management -->
        <UCard variant="outline">
          <template #header>
            <span class="font-semibold text-sm text-highlighted">PATH Management</span>
          </template>

          <p class="text-sm text-muted mb-4">
            Restore the OS PATH environment variable from the most recent backup. Backups are created
            automatically before every PATH modification.
          </p>

          <UButton
            color="warning"
            variant="soft"
            icon="i-lucide-rotate-ccw"
            :loading="restoringBackup"
            @click="restoreBackup"
          >
            Restore from Backup
          </UButton>

          <div
            v-if="backupRestoreResult"
            class="mt-3 text-sm"
            :class="backupRestoreResult.success ? 'text-success' : 'text-error'"
          >
            {{ backupRestoreResult.message }}
          </div>
        </UCard>

        <!-- Catalog Info -->
        <UCard variant="outline">
          <template #header>
            <span class="font-semibold text-sm text-highlighted">Service Catalog</span>
          </template>

          <p class="text-sm text-muted mb-2">
            {{ servicesStore.catalog.length }} services available in catalog.
          </p>

          <div class="divide-y divide-default text-sm">
            <div
              v-for="item in servicesStore.catalog"
              :key="item.id"
              class="flex items-center justify-between py-1.5 first:pt-0 last:pb-0"
            >
              <div class="flex items-center gap-2">
                <span class="text-default capitalize">{{ item.name }}</span>
                <span class="font-mono text-dimmed text-xs">{{ item.version }}</span>
              </div>
              <span v-if="item.port > 0" class="text-muted font-mono text-xs">Port {{ item.port }}</span>
            </div>
          </div>
        </UCard>
      </div>
    </template>
  </UDashboardPanel>
</template>
