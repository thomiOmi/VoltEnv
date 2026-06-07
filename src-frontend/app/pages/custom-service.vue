<script setup lang="ts">
definePageMeta({
  title: 'Custom Service',
})

const toast = useToast()
const api = useServiceApi()
const router = useRouter()

const id = ref('')
const name = ref('')
const defaultVersion = ref('1.0.0')
const binaryName = ref('')
const port = ref(8080)
const startArgs = ref('')
const stopArgs = ref('')
const downloadUrl = ref('')
const sha256 = ref('')
const configTemplateName = ref('')
const healthCheckType = ref<'none' | 'port' | 'command'>('none')
const healthCheckCommand = ref('')
const healthCheckTimeoutMs = ref(5000)
const postInstallCommands = ref('')
const submitting = ref(false)

async function handleSubmit() {
  if (!id.value.trim() || !name.value.trim() || !binaryName.value.trim()) {
    toast.add({ title: 'ID, Name, and Binary Name are required', color: 'error' })
    return
  }

  const idClean = id.value.trim().toLowerCase().replace(/[^a-z0-9_-]/g, '-')
  const versions: Record<string, { downloadUrl: string, sha256: string | null }> = {}
  if (downloadUrl.value.trim()) {
    versions[defaultVersion.value || '1.0.0'] = {
      downloadUrl: downloadUrl.value.trim(),
      sha256: sha256.value.trim() || null,
    }
  }

  const service = {
    id: idClean,
    name: name.value.trim(),
    kind: 'custom',
    defaultVersion: defaultVersion.value || '1.0.0',
    versions,
    binaryName: binaryName.value.trim(),
    startArgs: startArgs.value
      .split('\n')
      .map(s => s.trim())
      .filter(Boolean),
    stopArgs: stopArgs.value
      .split('\n')
      .map(s => s.trim())
      .filter(Boolean),
    port: port.value || 8080,
    configTemplateName: configTemplateName.value.trim() || null,
    healthCheck: healthCheckType.value === 'none'
      ? null
      : {
          type: healthCheckType.value,
          command: healthCheckType.value === 'command' ? healthCheckCommand.value.trim() : null,
          timeoutMs: healthCheckTimeoutMs.value,
        },
    postInstallCommands: postInstallCommands.value
      .split('\n')
      .map(s => s.trim())
      .filter(Boolean),
  }

  submitting.value = true
  try {
    await api.saveCustomService(service)
    toast.add({ title: `Custom service "${name.value}" saved`, color: 'success' })
    router.push('/')
  }
  catch (e) {
    toast.add({ title: String(e), color: 'error' })
  }
  finally {
    submitting.value = false
  }
}
</script>

<template>
  <UDashboardPanel>
    <template #header>
      <UDashboardNavbar title="Custom Service">
        <template #left>
          <UButton
            color="neutral"
            variant="ghost"
            icon="i-lucide-arrow-left"
            @click="router.push('/')"
          />
        </template>
      </UDashboardNavbar>
    </template>

    <template #body>
      <div class="p-4 max-w-lg">
        <UForm :state="{}" @submit="handleSubmit">
          <UFormField label="ID" required hint="Used as unique identifier, auto-lowered">
            <UInput v-model="id" placeholder="e.g. my-service" class="w-full" />
          </UFormField>

          <UFormField label="Name" required class="mt-4">
            <UInput v-model="name" placeholder="e.g. My Service" class="w-full" />
          </UFormField>

          <UFormField label="Binary Name" required class="mt-4">
            <UInput v-model="binaryName" placeholder="e.g. my-service.exe" class="w-full" />
          </UFormField>

          <UFormField label="Default Version" class="mt-4">
            <UInput v-model="defaultVersion" placeholder="1.0.0" class="w-full" />
          </UFormField>

          <UFormField label="Port" class="mt-4">
            <UInput v-model="port" type="number" class="w-full" />
          </UFormField>

          <UCard class="mt-4">
            <template #header>
              <span class="text-sm font-medium text-muted">Version / Download</span>
            </template>
            <UFormField label="Download URL">
              <UInput v-model="downloadUrl" placeholder="https://..." class="w-full" />
            </UFormField>
            <UFormField label="SHA-256" class="mt-3">
              <UInput v-model="sha256" placeholder="Optional" class="w-full" />
            </UFormField>
          </UCard>

          <UCard class="mt-4">
            <template #header>
              <span class="text-sm font-medium text-muted">Arguments</span>
            </template>
            <UFormField label="Start Args (one per line)">
              <UTextarea
                v-model="startArgs"
                placeholder="-p&#10;-c {{config_path}}"
                class="w-full"
                :rows="2"
              />
            </UFormField>
            <UFormField label="Stop Args (one per line)" class="mt-3">
              <UTextarea
                v-model="stopArgs"
                placeholder="-s quit"
                class="w-full"
                :rows="2"
              />
            </UFormField>
          </UCard>

          <UFormField label="Config Template Name" class="mt-4">
            <UInput v-model="configTemplateName" placeholder="e.g. my-app.conf.tpl" class="w-full" />
          </UFormField>

          <UCard class="mt-4">
            <template #header>
              <span class="text-sm font-medium text-muted">Health Check</span>
            </template>
            <USelect
              v-model="healthCheckType"
              :items="[
                { label: 'None', value: 'none' },
                { label: 'Port', value: 'port' },
                { label: 'Command', value: 'command' },
              ]"
            />
            <UFormField v-if="healthCheckType === 'command'" label="Health Check Command" class="mt-3">
              <UInput v-model="healthCheckCommand" placeholder="e.g. {{bin_dir}}/ping" class="w-full" />
            </UFormField>
            <UFormField label="Timeout (ms)" class="mt-3">
              <UInput v-model="healthCheckTimeoutMs" type="number" class="w-full" />
            </UFormField>
          </UCard>

          <UFormField label="Post-Install Commands (one per line)" class="mt-4">
            <UTextarea
              v-model="postInstallCommands"
              placeholder="e.g. {{bin_dir}}/init.sh"
              class="w-full"
              :rows="2"
            />
          </UFormField>

          <div class="mt-6">
            <UButton type="submit" :loading="submitting" color="primary">
              Save Service
            </UButton>
          </div>
        </UForm>
      </div>
    </template>
  </UDashboardPanel>
</template>
