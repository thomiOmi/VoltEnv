<script setup lang="ts">
const route = useRoute()
const router = useRouter()
const id = computed(() => route.params.id as string)

const servicesStore = useServicesStore()
const api = useServiceApi()
const toast = useToast()

const def = computed(() => servicesStore.getDefinition(id.value))
const status = computed(() => servicesStore.getStatus(id.value))

const extensions = ref<[string, boolean][]>([])
const loadingExtensions = ref(false)

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

async function loadExtensions() {
  if (id.value !== 'php') return
  loadingExtensions.value = true
  try {
    const version = status.value?.version || def.value?.defaultVersion || '0.0.0'
    extensions.value = await api.getPhpExtensions(version)
  }
  finally {
    loadingExtensions.value = false
  }
}

async function toggleExtension(ext: string, current: boolean) {
  try {
    const version = status.value?.version || def.value?.defaultVersion || '0.0.0'
    await api.togglePhpExtension(version, ext, !current)
    await loadExtensions()
    toast.add({
      title: 'Success',
      description: `Extension ${ext} ${!current ? 'enabled' : 'disabled'}. Restart PHP to apply changes.`,
      color: 'success'
    })
  } catch (e) {
    toast.add({ title: 'Error', description: String(e), color: 'error' })
  }
}

onMounted(() => {
  if (id.value === 'php') {
    loadExtensions()
  }
})

// Watch for version changes to reload extensions
watch(() => status.value?.version, () => {
  if (id.value === 'php') loadExtensions()
})
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

      <div v-else class="p-4 space-y-6 overflow-y-auto">
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

        <!-- Projects / Vhost List for Nginx -->
        <UCard v-if="id === 'nginx'">
          <template #header>
            <span class="text-sm font-medium text-muted">Projects</span>
          </template>
          <VhostList />
        </UCard>

        <!-- PHP Extension Manager -->
        <UCard v-if="id === 'php'" class="overflow-hidden">
          <template #header>
            <div class="flex items-center justify-between">
              <span class="text-sm font-medium text-muted">PHP Extensions</span>
              <UButton
                variant="ghost"
                icon="i-lucide-refresh-cw"
                size="xs"
                :loading="loadingExtensions"
                @click="loadExtensions"
              />
            </div>
          </template>

          <div v-if="extensions.length === 0 && !loadingExtensions" class="text-sm text-muted p-4 text-center">
            No extensions found in php.ini
          </div>

          <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-2">
            <div
              v-for="[name, enabled] in extensions"
              :key="name"
              class="flex items-center justify-between p-2 rounded border border-default bg-elevated/50"
            >
              <span class="text-xs font-mono truncate mr-2" :title="name">{{ name }}</span>
              <USwitch
                :model-value="enabled"
                size="xs"
                @update:model-value="toggleExtension(name, enabled)"
              />
            </div>
          </div>
        </UCard>

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
