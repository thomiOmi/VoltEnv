<script setup lang="ts">
const route = useRoute()
const router = useRouter()
const servicesStore = useServicesStore()

const id = computed(() => route.params.id as string)

const service = computed(() =>
  servicesStore.services.find(s => s.id === id.value),
)

const isRunning = computed(() => service.value?.status === 'Running')
const isStarting = computed(() => service.value?.status === 'Starting')

const otherVersions = computed(() => {
  const versions = servicesStore.versionsFor(id.value)
  return versions.filter(v => v !== service.value?.version)
})

async function handleStart() {
  if (service.value) {
    await servicesStore.startService(service.value.id)
  }
}

async function handleStop() {
  if (service.value) {
    await servicesStore.stopService(service.value.id)
  }
}

async function handleSoftStop() {
  if (service.value) {
    await servicesStore.softStopService(service.value.id)
  }
}

async function handleForceStop() {
  if (service.value) {
    await servicesStore.forceStopService(service.value.id)
  }
}

async function handleSwitch(version: string) {
  if (service.value) {
    await servicesStore.switchServiceVersion(service.value.id, version)
  }
}

const catalogInfo = computed(() =>
  servicesStore.catalog.find(s => s.id === id.value),
)

async function handleDownloadTemplates() {
  const info = catalogInfo.value
  if (!info) return

  const manifest: DownloadManifest = {
    assets: [
      {
        name: `${info.id}-${info.version}.zip`,
        url: info.downloadUrl,
        destinationSubdir: info.version,
        sha256: info.sha256 ?? null,
        pgpSignatureUrl: info.pgpSignatureUrl ?? null,
        extract: true,
      },
    ],
  }

  await servicesStore.downloadTemplates(info.id, manifest)
}
</script>

<template>
  <UDashboardPanel v-if="service" :id="`service-${service.id}`">
    <template #header>
      <UDashboardNavbar :title="service.name">
        <template #leading>
          <UButton
            color="neutral"
            variant="ghost"
            square
            icon="i-lucide-arrow-left"
            @click="router.back()"
          />
        </template>

        <template #right>
          <div class="flex items-center gap-2">
            <UButton
              color="success"
              variant="soft"
              size="sm"
              icon="i-lucide-play"
              :disabled="isRunning || isStarting"
              :loading="servicesStore.loadingStates[service.id]"
              @click="handleStart"
            >
              Start
            </UButton>
            <UButton
              color="error"
              variant="soft"
              size="sm"
              icon="i-lucide-square-stop"
              :disabled="!isRunning"
              :loading="servicesStore.loadingStates[service.id]"
              @click="handleStop"
            >
              Stop
            </UButton>
          </div>
        </template>
      </UDashboardNavbar>
    </template>

    <template #body>
      <div class="max-w-4xl mx-auto w-full p-6 space-y-6">
        <!-- Status Card -->
        <UCard variant="outline">
          <template #header>
            <span class="font-semibold text-sm text-highlighted">Status</span>
          </template>

          <div class="grid grid-cols-2 gap-6">
            <div class="space-y-3">
              <div class="flex justify-between text-sm">
                <span class="text-muted">Status</span>
                <UBadge
                  :color="isRunning ? 'success' : isStarting ? 'warning' : 'neutral'"
                  variant="soft"
                  size="sm"
                >
                  {{ isRunning ? 'Running' : isStarting ? 'Starting' : 'Stopped' }}
                </UBadge>
              </div>
              <div class="flex justify-between text-sm">
                <span class="text-muted">Port</span>
                <span class="font-mono text-default">{{ service.port }}</span>
              </div>
              <div class="flex justify-between text-sm">
                <span class="text-muted">Version</span>
                <span v-if="service.version" class="font-mono text-default">{{ service.version }}</span>
                <span v-else class="text-dimmed italic text-xs">Not installed</span>
              </div>
            </div>
            <div class="space-y-3">
              <div class="flex justify-between text-sm">
                <span class="text-muted">OS Active</span>
                <UBadge
                  v-if="servicesStore.isActiveVersion(service.id, service.version)"
                  color="success"
                  variant="subtle"
                  size="xs"
                >
                  Active
                </UBadge>
                <span v-else class="text-dimmed text-xs">No</span>
              </div>
              <div class="flex justify-between text-sm">
                <span class="text-muted">Available Versions</span>
                <span class="font-mono text-default">{{ service.version ? otherVersions.length + 1 : 0 }}</span>
              </div>
            </div>
          </div>
        </UCard>

        <!-- Advanced Actions -->
        <UCard variant="outline">
          <template #header>
            <span class="font-semibold text-sm text-highlighted">Advanced Actions</span>
          </template>

          <div class="flex flex-wrap gap-3">
            <UButton
              color="warning"
              variant="soft"
              size="sm"
              icon="i-lucide-power"
              :disabled="!isRunning"
              @click="handleSoftStop"
            >
              Soft Stop (SIGTERM)
            </UButton>
            <UButton
              color="error"
              variant="soft"
              size="sm"
              icon="i-lucide-octagon-x"
              :disabled="!isRunning"
              @click="handleForceStop"
            >
              Force Stop (SIGKILL)
            </UButton>
          </div>
        </UCard>

        <!-- Download Templates -->
        <UCard variant="outline">
          <template #header>
            <span class="font-semibold text-sm text-highlighted">Download Templates</span>
          </template>

          <p class="text-sm text-muted mb-4">
            Download, verify, and extract template assets for this service.
          </p>

          <UButton
            color="primary"
            variant="soft"
            size="sm"
            icon="i-lucide-download"
            :loading="servicesStore.loadingStates[service.id]"
            @click="handleDownloadTemplates"
          >
            Download & Extract
          </UButton>
        </UCard>

        <!-- Version Switching -->
        <UCard v-if="otherVersions.length > 0 && !isRunning && !isStarting" variant="outline">
          <template #header>
            <span class="font-semibold text-sm text-highlighted">Switch Version</span>
          </template>

          <div class="divide-y divide-default">
            <div
              v-for="ver in otherVersions"
              :key="ver"
              class="flex items-center justify-between py-2 first:pt-0 last:pb-0"
            >
              <div class="flex items-center gap-2">
                <span class="text-sm font-mono text-default">{{ ver }}</span>
                <UBadge
                  v-if="servicesStore.isActiveVersion(service.id, ver)"
                  color="success"
                  variant="subtle"
                  size="xs"
                >
                  OS Active
                </UBadge>
              </div>
              <UButton
                color="neutral"
                variant="ghost"
                size="xs"
                icon="i-lucide-arrow-left-right"
                :loading="servicesStore.switchingVersions[service.id]"
                @click="handleSwitch(ver)"
              >
                Switch
              </UButton>
            </div>
          </div>
        </UCard>

        <!-- Log Console -->
        <div v-if="service.version">
          <h2 class="text-sm font-semibold text-muted uppercase tracking-wider mb-3">
            Logs
          </h2>
          <LogConsole
            :service-id="service.id"
            :version="service.version"
          />
        </div>
      </div>
    </template>
  </UDashboardPanel>

  <UDashboardPanel v-else id="not-found">
    <template #header>
      <UDashboardNavbar title="Service Not Found">
        <template #leading>
          <UButton
            color="neutral"
            variant="ghost"
            square
            icon="i-lucide-arrow-left"
            @click="router.push('/')"
          />
        </template>
      </UDashboardNavbar>
    </template>

    <template #body>
      <div class="flex flex-col items-center justify-center h-full gap-4">
        <span class="i-lucide-server size-12 text-dimmed" />
        <p class="text-muted text-sm">
          Service "{{ id }}" not found.
        </p>
        <UButton color="neutral" variant="soft" to="/">
          Back to Services
        </UButton>
      </div>
    </template>
  </UDashboardPanel>
</template>
