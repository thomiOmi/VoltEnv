<script setup lang="ts">
definePageMeta({
  title: 'Custom Service',
})

const toast = useToast()
const api = useServiceApi()
const router = useRouter()

// Form State
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

// UX State
const currentStep = ref(0)
const submitting = ref(false)
const idAvailable = ref<boolean | null>(null)
const checkingId = ref(false)

const steps = [
  { label: 'General', description: 'Basic service identification' },
  { label: 'Execution', description: 'How the service runs' },
  { label: 'Advanced', description: 'Post-install and Health' }
]

// ID Validation
const idClean = computed(() => id.value.trim().toLowerCase().replace(/[^a-z0-9_-]/g, '-'))

watch(idClean, async (newId) => {
  if (!newId) {
    idAvailable.value = null
    return
  }
  checkingId.value = true
  idAvailable.value = await api.checkServiceIdAvailable(newId)
  checkingId.value = false
})

async function nextStep() {
  if (currentStep.value === 0) {
    if (!idClean.value || idAvailable.value === false) {
      toast.add({ title: 'Invalid or taken Service ID', color: 'error' })
      return
    }
    if (!name.value.trim()) {
      toast.add({ title: 'Service Name is required', color: 'error' })
      return
    }
  }

  if (currentStep.value === 1) {
    if (!binaryName.value.trim()) {
      toast.add({ title: 'Binary Name is required', color: 'error' })
      return
    }
  }

  if (currentStep.value < steps.length - 1) {
    currentStep.value++
  } else {
    await handleSubmit()
  }
}

async function handleSubmit() {
  const versions: Record<string, { downloadUrl: string, sha256: string | null }> = {}
  if (downloadUrl.value.trim()) {
    versions[defaultVersion.value || '1.0.0'] = {
      downloadUrl: downloadUrl.value.trim(),
      sha256: sha256.value.trim() || null,
    }
  }

  const service = {
    id: idClean.value,
    name: name.value.trim(),
    kind: 'custom',
    defaultVersion: defaultVersion.value || '1.0.0',
    versions,
    binaryName: binaryName.value.trim(),
    startArgs: startArgs.value.split('\n').map(s => s.trim()).filter(Boolean),
    stopArgs: stopArgs.value.split('\n').map(s => s.trim()).filter(Boolean),
    port: port.value || 8080,
    configTemplateName: configTemplateName.value.trim() || null,
    healthCheck: healthCheckType.value === 'none' ? null : {
      type: healthCheckType.value,
      command: healthCheckType.value === 'command' ? healthCheckCommand.value.trim() : null,
      timeoutMs: healthCheckTimeoutMs.value,
    },
    postInstallCommands: postInstallCommands.value.split('\n').map(s => s.trim()).filter(Boolean),
  }

  submitting.value = true
  try {
    await api.saveCustomService(service)
    toast.add({ title: `Custom service "${name.value}" saved`, color: 'success' })
    router.push('/')
  }
  catch (e) {
    // Handled by API toast
  }
  finally {
    submitting.value = false
  }
}
</script>

<template>
  <UDashboardPanel>
    <template #header>
      <UDashboardNavbar title="Add Custom Service">
        <template #left>
          <UButton
            color="neutral"
            variant="ghost"
            icon="i-lucide-arrow-left"
            aria-label="Back to Services"
            @click="router.push('/')"
          />
        </template>
      </UDashboardNavbar>
    </template>

    <template #body>
      <div class="p-6 max-w-2xl mx-auto">
        <UStepper v-model="currentStep" :items="steps" class="mb-8" />

        <div class="space-y-6">
          <!-- Step 0: General -->
          <div v-if="currentStep === 0" class="space-y-4">
            <UFormField
              label="Service ID"
              required
              hint="Lowercase, no spaces"
              :error="idAvailable === false ? 'This ID is already taken' : undefined"
            >
              <template #label>
                <div class="flex items-center gap-2">
                  <span>Service ID</span>
                  <UIcon v-if="checkingId" name="i-lucide-loader-2" class="animate-spin size-3" aria-label="Checking ID availability" />
                  <UIcon v-else-if="idAvailable === true" name="i-lucide-check" class="text-success size-3" aria-label="ID is available" />
                </div>
              </template>
              <UInput v-model="id" placeholder="e.g. redis-local" class="w-full" />
            </UFormField>

            <UFormField label="Display Name" required>
              <UInput v-model="name" placeholder="e.g. My Local Redis" class="w-full" />
            </UFormField>

            <div class="grid grid-cols-2 gap-4">
              <UFormField label="Default Version">
                <UInput v-model="defaultVersion" placeholder="1.0.0" class="w-full" />
              </UFormField>
              <UFormField label="Listening Port">
                <UInput v-model="port" type="number" class="w-full" />
              </UFormField>
            </div>
          </div>

          <!-- Step 1: Execution -->
          <div v-if="currentStep === 1" class="space-y-4 animate-in fade-in slide-in-from-right-4">
            <UFormField label="Binary Name" required hint="Executable filename inside the version folder">
              <UInput v-model="binaryName" placeholder="e.g. redis-server.exe" class="w-full" />
            </UFormField>

            <UFormField label="Start Arguments" hint="One argument per line">
              <UTextarea v-model="startArgs" placeholder="--port {{port}}&#10;--dir {{data_dir}}" class="w-full" :rows="3" />
            </UFormField>

            <UFormField label="Stop Arguments" hint="Commands to gracefully shutdown">
              <UTextarea v-model="stopArgs" placeholder="shutdown" class="w-full" :rows="2" />
            </UFormField>
          </div>

          <!-- Step 2: Advanced -->
          <div v-if="currentStep === 2" class="space-y-4 animate-in fade-in slide-in-from-right-4">
            <UCard variant="subtle">
              <template #header>
                <span class="text-sm font-medium">Download Configuration</span>
              </template>
              <div class="space-y-3">
                <UFormField label="Download URL (ZIP/Tar)">
                  <UInput v-model="downloadUrl" placeholder="https://..." class="w-full" />
                </UFormField>
                <UFormField label="SHA-256 Checksum">
                  <UInput v-model="sha256" placeholder="Optional" class="w-full" />
                </UFormField>
              </div>
            </UCard>

            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
              <UFormField label="Health Check Type">
                <USelect v-model="healthCheckType" :items="[{ label: 'None', value: 'none' }, { label: 'Port', value: 'port' }, { label: 'Command', value: 'command' }]" />
              </UFormField>
              <UFormField v-if="healthCheckType === 'command'" label="Command">
                <UInput v-model="healthCheckCommand" placeholder="ping" />
              </UFormField>
            </div>

            <UFormField label="Post-Install Script" hint="Runs once after download">
              <UTextarea v-model="postInstallCommands" placeholder="./setup.sh" class="w-full" :rows="2" />
            </UFormField>
          </div>
        </div>

        <div class="mt-10 flex items-center justify-between border-t border-default pt-6">
          <UButton
            v-if="currentStep > 0"
            color="neutral"
            variant="ghost"
            label="Previous"
            @click="currentStep--"
          />
          <div v-else />

          <UButton
            color="primary"
            :loading="submitting"
            :label="currentStep === steps.length - 1 ? 'Save Service' : 'Next Step'"
            icon-trailing="i-lucide-chevron-right"
            @click="nextStep"
          />
        </div>
      </div>
    </template>
  </UDashboardPanel>
</template>
