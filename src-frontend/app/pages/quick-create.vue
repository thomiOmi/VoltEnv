<script setup lang="ts">
definePageMeta({
  title: 'Quick Create',
})

const toast = useToast()
const api = useServiceApi()

const projectName = ref('')
const createDatabase = ref(true)
const submitting = ref(false)
const result = ref<{
  projectName: string
  domain: string
  rootPath: string
  createdVhost: boolean
  createdDatabase: boolean
} | null>(null)

async function handleCreate() {
  const name = projectName.value.trim()
  if (!name) {
    toast.add({ title: 'Project name is required', color: 'error' })
    return
  }

  submitting.value = true
  result.value = null

  try {
    result.value = await api.quickCreate(name, createDatabase.value)
    toast.add({
      title: `Project "${name}" created`,
      description: `Visit https://${name.toLocaleLowerCase().replace(/[^a-z0-9_-]/g, '-')}.test`,
      color: 'success',
    })
  }
  catch (e) {
    toast.add({ title: String(e), color: 'error' })
  }
  finally {
    submitting.value = false
  }
}

function reset() {
  projectName.value = ''
  result.value = null
}
</script>

<template>
  <UDashboardPanel>
    <template #header>
      <UDashboardNavbar title="Quick Create" />
    </template>

    <template #body>
      <div class="p-4 max-w-lg">
        <UForm :state="{ projectName, createDatabase }" @submit="handleCreate">
          <UFormField label="Project Name" required>
            <UInput
              v-model="projectName"
              placeholder="e.g. my-project"
              class="w-full"
              :disabled="submitting"
            />
          </UFormField>

          <UFormField label="Options" class="mt-4">
            <UToggle v-model="createDatabase" label="Create MySQL database" />
          </UFormField>

          <div class="flex items-center gap-2 mt-6">
            <UButton type="submit" :loading="submitting" color="primary">
              Create Project
            </UButton>
            <UButton
              v-if="result"
              color="neutral"
              variant="outline"
              @click="reset"
            >
              Reset
            </UButton>
          </div>
        </UForm>

        <div v-if="result" class="mt-6 p-4 rounded-lg bg-elevated border border-default space-y-2">
          <h3 class="font-medium text-default">
            Result
          </h3>
          <p class="text-sm text-muted">
            Domain: <span class="text-highlighted">{{ result.domain }}</span>
          </p>
          <p class="text-sm text-muted">
            Root: <span class="text-highlighted">{{ result.rootPath }}</span>
          </p>
          <p class="text-sm text-muted">
            Vhost:
            <UBadge :color="result.createdVhost ? 'success' : 'error'" variant="subtle" size="sm">
              {{ result.createdVhost ? 'Created' : 'Failed' }}
            </UBadge>
          </p>
          <p class="text-sm text-muted">
            Database:
            <UBadge :color="result.createdDatabase ? 'success' : 'neutral'" variant="subtle" size="sm">
              {{ result.createdDatabase ? 'Created' : 'Skipped' }}
            </UBadge>
          </p>
        </div>
      </div>
    </template>
  </UDashboardPanel>
</template>
