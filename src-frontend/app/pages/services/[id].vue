<script setup lang="ts">
const route = useRoute()
const router = useRouter()
const id = computed(() => route.params.id as string)

const servicesStore = useServicesStore()
const api = useServiceApi()
const toast = useToast()

const def = computed(() => servicesStore.getDefinition(id.value))
const status = computed(() => servicesStore.getStatus(id.value))

// MySQL Tester State
const mysqlUser = ref('root')
const mysqlPass = ref('')
const testingConnection = ref(false)

async function handleStart() {
  try {
    await servicesStore.startService(id.value)
  }
  catch (e) {
    // Handled by global wrapper
  }
}

async function handleStop() {
  try {
    await servicesStore.stopService(id.value)
  }
  catch (e) {
    // Handled by global wrapper
  }
}

async function testConnection() {
  testingConnection.value = true
  try {
    const result = await api.testMysqlConnection(mysqlUser.value, mysqlPass.value)
    toast.add({
      title: 'Success',
      description: result,
      color: 'success',
      icon: 'i-lucide-database-zap'
    })
  }
  catch (e) {
    // Detailed error already shown by API wrapper
  }
  finally {
    testingConnection.value = false
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
            aria-label="Back to Dashboard"
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
        <!-- Status Cards -->
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
            <span class="text-sm font-mono truncate" :title="def.binaryName">{{ def.binaryName }}</span>
          </UCard>
        </div>

        <!-- Database Tools (Specific to MySQL) -->
        <UCard v-if="id === 'mysql'" variant="subtle">
          <template #header>
            <div class="flex items-center gap-2">
              <UIcon name="i-lucide-database" class="text-primary" />
              <span class="text-sm font-medium">Database Connection Tester</span>
            </div>
          </template>

          <div class="flex flex-col md:flex-row items-end gap-4 max-w-2xl">
            <UFormField label="Username" class="flex-1 w-full">
              <UInput v-model="mysqlUser" placeholder="root" class="w-full" />
            </UFormField>
            <UFormField label="Password" class="flex-1 w-full">
              <UInput v-model="mysqlPass" type="password" placeholder="Leave empty if none" class="w-full" />
            </UFormField>
            <UButton
              label="Test Connection"
              icon="i-lucide-plug-zap"
              color="primary"
              variant="soft"
              :loading="testingConnection"
              :disabled="status?.status !== 'running'"
              @click="testConnection"
            />
          </div>
          <p v-if="status?.status !== 'running'" class="text-xs text-muted mt-2 italic">
            MySQL must be running to test the connection.
          </p>
        </UCard>

        <!-- Logs Section -->
        <UCard>
          <template #header>
            <span class="text-sm font-medium text-muted">Real-time Logs</span>
          </template>
          <LogConsole :service-id="id" :version="status?.version ?? def.defaultVersion" />
        </UCard>
      </div>
    </template>
  </UDashboardPanel>
</template>
