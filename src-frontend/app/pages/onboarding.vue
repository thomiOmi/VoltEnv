<script setup lang="ts">
definePageMeta({
  layout: 'blank',
})

const servicesStore = useServicesStore()
const api = useServiceApi()
const router = useRouter()
const toast = useToast()

const step = ref(1)
const checkingPorts = ref(false)
const portStatus = ref<{port: number, available: boolean}[]>([])
const downloading = ref(false)

const requiredServices = ['nginx', 'php', 'mysql']

async function checkPorts() {
  checkingPorts.value = true
  const ports = [80, 443, 3306, 9000]
  const results = []
  for (const p of ports) {
    const available = await api.isPortAvailable(p)
    results.push({ port: p, available })
  }
  portStatus.value = results
  checkingPorts.value = false
  step.value = 2
}

async function startSetup() {
  downloading.value = true
  try {
    for (const id of requiredServices) {
      const def = servicesStore.getDefinition(id)
      if (def) {
        await servicesStore.setupService(id, def.defaultVersion)
      }
    }
    step.value = 3
  } catch (e) {
    toast.add({ title: 'Setup failed', description: String(e), color: 'error' })
  } finally {
    downloading.value = false
  }
}

function finish() {
  localStorage.setItem('voltenv_onboarded', 'true')
  router.push('/')
}

onMounted(async () => {
  await servicesStore.init()
})
</script>

<template>
  <div class="min-h-screen flex items-center justify-center bg-base p-4">
    <UCard class="max-w-md w-full">
      <template #header>
        <div class="flex items-center gap-2">
          <img src="/app-icon.png" class="size-8" />
          <h1 class="text-xl font-bold">Welcome to VoltEnv</h1>
        </div>
      </template>

      <div v-if="step === 1" class="space-y-4">
        <p class="text-sm text-muted">
          VoltEnv will help you set up a high-performance local development environment for PHP.
        </p>
        <UButton block color="primary" :loading="checkingPorts" @click="checkPorts">
          Get Started
        </UButton>
      </div>

      <div v-if="step === 2" class="space-y-4">
        <h2 class="font-medium">System Check</h2>
        <div class="space-y-2">
          <div v-for="p in portStatus" :key="p.port" class="flex items-center justify-between text-sm">
            <span>Port {{ p.port }}</span>
            <UBadge :color="p.available ? 'success' : 'error'" variant="subtle">
              {{ p.available ? 'Available' : 'In Use' }}
            </UBadge>
          </div>
        </div>
        <p class="text-xs text-muted">
          Note: If ports 80/443 are in use, VoltEnv will automatically use alternative ports.
        </p>
        <UButton block color="primary" :loading="downloading" @click="startSetup">
          Download & Install Binaries
        </UButton>
      </div>

      <div v-if="step === 3" class="space-y-4 text-center">
        <div class="flex justify-center">
          <UIcon name="i-lucide-check-circle" class="size-16 text-success" />
        </div>
        <h2 class="text-xl font-bold">Everything is Ready!</h2>
        <p class="text-sm text-muted">
          Nginx, PHP, and MySQL have been installed and configured.
        </p>
        <UButton block color="primary" @click="finish">
          Launch Dashboard
        </UButton>
      </div>

      <template #footer>
        <div class="flex justify-between items-center text-[10px] text-muted uppercase tracking-wider">
          <span>Step {{ step }} of 3</span>
          <span v-if="downloading">Installing...</span>
        </div>
      </template>
    </UCard>
  </div>
</template>
