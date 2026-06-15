<script setup lang="ts">
import { save, open } from '@tauri-apps/plugin-dialog'
import { writeTextFile, readTextFile } from '@tauri-apps/plugin-fs'

definePageMeta({
  title: 'Settings',
})

const toast = useToast()
const servicesStore = useServicesStore()
const api = useServiceApi()

const ports = ref<Record<string, number>>({})
const autoStartGroups = ref<Array<{ name: string, services: string[], autoStart: boolean }>>([])
const dirty = ref(false)

const editGroup = ref<{
  index: number
  name: string
  services: string
  autoStart: boolean
} | null>(null)
const editGroupModalOpen = ref(false)

const vhosts = ref<Array<{ domain: string, root: string, port: number, phpPort: number | null }>>([])
const databases = ref<string[]>([])
const loadingVhosts = ref(false)
const loadingDbs = ref(false)
const processingBackup = ref(false)

async function loadSettings() {
  const s = await api.getSettings()
  if (s) {
    ports.value = { ...s.preferredPorts }
    autoStartGroups.value = s.autoStartGroups.map(g => ({ ...g }))
  }
}

async function loadVhosts() {
  loadingVhosts.value = true
  try {
    vhosts.value = await api.listVhosts()
  }
  finally {
    loadingVhosts.value = false
  }
}

async function loadDatabases() {
  loadingDbs.value = true
  try {
    databases.value = await api.listDatabases()
  }
  finally {
    loadingDbs.value = false
  }
}

onMounted(() => {
  loadSettings()
  loadVhosts()
  loadDatabases()
})

async function handleSave() {
  try {
    await api.updateSettings({
      preferredPorts: ports.value,
      resolvedPorts: {},
      autoStartGroups: autoStartGroups.value,
      activeVersions: {},
    })
    dirty.value = false
    toast.add({ title: 'Settings saved', color: 'success' })
  }
  catch (e) {
    // Handled by API wrapper
  }
}

async function handleDeleteVhost(domain: string) {
  try {
    await api.deleteVhost(domain)
    toast.add({ title: `Vhost "${domain}" deleted`, color: 'success' })
    await loadVhosts()
  }
  catch (e) {}
}

async function handleDropDatabase(name: string) {
  try {
    await api.dropDatabase(name)
    toast.add({ title: `Database "${name}" dropped`, color: 'success' })
    await loadDatabases()
  }
  catch (e) {}
}

function openAddGroup() {
  editGroup.value = {
    index: -1,
    name: '',
    services: '',
    autoStart: true,
  }
  editGroupModalOpen.value = true
}

function openEditGroup(index: number) {
  const g = autoStartGroups.value[index]
  if (!g) return
  editGroup.value = {
    index,
    name: g.name,
    services: g.services.join(', '),
    autoStart: g.autoStart,
  }
  editGroupModalOpen.value = true
}

function saveGroup() {
  const g = editGroup.value
  if (!g) return
  const services = g.services.split(',').map(s => s.trim()).filter(Boolean)
  const entry = {
    name: g.name || 'Unnamed Group',
    services,
    autoStart: g.autoStart,
  }
  if (g.index >= 0) autoStartGroups.value[g.index] = entry
  else autoStartGroups.value.push(entry)
  dirty.value = true
  editGroupModalOpen.value = false
  editGroup.value = null
}

function removeGroup(index: number) {
  autoStartGroups.value.splice(index, 1)
  dirty.value = true
}

// Backup & Migration
async function handleExport() {
  processingBackup.value = true
  try {
    const json = await api.exportConfiguration()
    const path = await save({
      filters: [{ name: 'VoltEnv Config', extensions: ['json'] }],
      defaultPath: 'voltenv-backup.json'
    })

    if (path) {
      await writeTextFile(path, json)
      toast.add({ title: 'Configuration exported successfully', color: 'success' })
    }
  } catch (e) {
    toast.add({ title: 'Export failed', description: String(e), color: 'error' })
  } finally {
    processingBackup.value = false
  }
}

async function handleImport() {
  processingBackup.value = true
  try {
    const path = await open({
      filters: [{ name: 'VoltEnv Config', extensions: ['json'] }],
      multiple: false
    })

    if (path && typeof path === 'string') {
      const content = await readTextFile(path)
      await api.importConfiguration(content)
      toast.add({ title: 'Configuration imported', description: 'Settings updated successfully.', color: 'success' })
      await loadSettings()
      await loadVhosts()
      await loadDatabases()
      await servicesStore.fetchDefinitions()
    }
  } catch (e) {
    toast.add({ title: 'Import failed', description: String(e), color: 'error' })
  } finally {
    processingBackup.value = false
  }
}
</script>

<template>
  <UDashboardPanel>
    <template #header>
      <UDashboardNavbar title="Settings">
        <template #right>
          <UButton
            color="primary"
            :disabled="!dirty"
            @click="handleSave"
          >
            Save
          </UButton>
        </template>
      </UDashboardNavbar>
    </template>

    <template #body>
      <div class="p-4 space-y-6 max-w-2xl">
        <!-- Backup & Migration -->
        <UCard variant="subtle">
          <template #header>
            <span class="text-sm font-medium">Backup & Migration</span>
          </template>
          <div class="flex items-center gap-3">
            <UButton
              icon="i-lucide-download"
              label="Export Config"
              variant="soft"
              color="neutral"
              size="sm"
              :loading="processingBackup"
              @click="handleExport"
            />
            <UButton
              icon="i-lucide-upload"
              label="Import Config"
              variant="soft"
              color="neutral"
              size="sm"
              :loading="processingBackup"
              @click="handleImport"
            />
          </div>
          <p class="text-[10px] text-muted mt-2 italic">
            Save your vhosts, custom services, and settings to a file or restore them.
          </p>
        </UCard>

        <!-- Ports -->
        <UCard>
          <template #header>
            <span class="text-sm font-medium text-muted">Ports</span>
          </template>
          <div class="space-y-3">
            <div v-for="svc in servicesStore.allDefinitions" :key="svc.id" class="flex items-center justify-between">
              <span class="text-sm font-medium">{{ svc.name }}</span>
              <UInput
                v-model="ports[svc.id]"
                type="number"
                class="w-24"
                size="sm"
                @update:model-value="dirty = true"
              />
            </div>
          </div>
        </UCard>

        <!-- Auto-Start Groups -->
        <UCard>
          <template #header>
            <div class="flex items-center justify-between">
              <span class="text-sm font-medium text-muted">Auto-Start Groups</span>
              <UButton
                color="primary"
                variant="outline"
                size="xs"
                @click="openAddGroup"
              >
                Add Group
              </UButton>
            </div>
          </template>
          <div v-if="autoStartGroups.length === 0" class="text-sm text-muted py-2">
            No groups configured.
          </div>
          <div v-else class="space-y-3">
            <div v-for="(group, idx) in autoStartGroups" :key="idx" class="flex items-center justify-between p-3 rounded-lg border border-default">
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2">
                  <span class="font-medium text-sm">{{ group.name }}</span>
                  <UBadge v-if="group.autoStart" color="success" variant="subtle" size="sm">auto</UBadge>
                </div>
                <div class="text-xs text-muted mt-1">{{ group.services.join(', ') }}</div>
              </div>
              <div class="flex items-center gap-1 shrink-0">
                <UButton color="neutral" variant="ghost" size="xs" icon="i-lucide-pencil" aria-label="Edit group" @click="openEditGroup(idx)" />
                <UButton color="error" variant="ghost" size="xs" icon="i-lucide-trash-2" aria-label="Remove group" @click="removeGroup(idx)" />
              </div>
            </div>
          </div>
        </UCard>

        <!-- Vhosts & DBs (Existing UI, keeping for completeness) -->
        <UCard>
          <template #header><span class="text-sm font-medium text-muted">Vhosts</span></template>
          <div v-if="loadingVhosts" class="text-sm text-muted py-2">Loading...</div>
          <div v-else-if="vhosts.length === 0" class="text-sm text-muted py-2">No vhosts configured.</div>
          <div v-else class="space-y-2">
            <div v-for="vh in vhosts" :key="vh.domain" class="flex items-center justify-between py-1 border-b border-default last:border-0">
              <div class="min-w-0">
                <span class="font-medium text-sm">{{ vh.domain }}</span>
                <span class="text-xs text-muted ml-2 block truncate">{{ vh.root }}</span>
              </div>
              <UButton color="error" variant="ghost" size="sm" icon="i-lucide-trash-2" aria-label="Delete vhost" @click="handleDeleteVhost(vh.domain)" />
            </div>
          </div>
        </UCard>
      </div>
    </template>

    <UModal v-model:open="editGroupModalOpen" title="Auto-Start Group">
      <template #body>
        <div class="space-y-4">
          <UFormField label="Group Name" required><UInput v-model="editGroup!.name" class="w-full" /></UFormField>
          <UFormField label="Services (comma-separated IDs)"><UInput v-model="editGroup!.services" class="w-full" placeholder="nginx, php, mysql" /></UFormField>
          <UFormField label="Options"><USwitch v-model="editGroup!.autoStart" label="Start automatically on launch" /></UFormField>
        </div>
      </template>
      <template #footer><div class="flex justify-end"><UButton color="primary" @click="saveGroup">Save</UButton></div></template>
    </UModal>
  </UDashboardPanel>
</template>
