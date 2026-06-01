<script setup lang="ts">
const props = defineProps<{
  service: Service
}>()

const servicesStore = useServicesStore()

const isRunning = computed(() => props.service.status === 'Running')
const isStarting = computed(() => props.service.status === 'Starting')
const isNotInstalled = computed(() => {
  // Cross-reference with catalog's installedVersions (filesystem truth).
  // Even if the backend returns a non-empty version, if the disk scan
  // shows no installed binaries the service is definitively not installed.
  const catalogEntry = servicesStore.catalog.find(s => s.id === props.service.id)
  if (catalogEntry) {
    return (catalogEntry.installedVersions?.length ?? 0) === 0
  }
  return !props.service.version
})
const isError = computed(() => props.service.status !== null && typeof props.service.status === 'object' && 'Error' in props.service.status)

const statusLabel = computed(() => {
  if (isRunning.value) return 'Running'
  if (isStarting.value) return 'Starting'
  if (isError.value) return 'Error'
  return 'Stopped'
})

const badgeColor = computed(() => {
  if (isRunning.value) return 'success'
  if (isStarting.value) return 'warning'
  if (isError.value) return 'error'
  return 'neutral'
})

const errorMessage = computed(() => {
  if (props.service.status !== null && typeof props.service.status === 'object' && 'Error' in props.service.status) {
    return (props.service.status as { Error: string }).Error
  }
  return null
})

const hasMultipleVersions = computed(() => {
  const versions = servicesStore.versionsFor(props.service.id)
  return versions.length > 1
})

const otherVersions = computed(() => {
  return servicesStore
    .versionsFor(props.service.id)
    .filter(v => v !== props.service.version)
})
</script>

<template>
  <UCard variant="outline">
    <template #header>
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2.5">
          <div
            class="size-2.5 rounded-full"
            :class="isRunning
              ? 'bg-success animate-pulse shadow-lg shadow-success/30'
              : 'bg-error/50'"
          />
          <NuxtLink
            :to="`/services/${service.id}`"
            class="font-semibold text-sm text-highlighted hover:text-primary transition-colors"
          >
            {{ service.name }}
          </NuxtLink>
        </div>

        <div class="flex items-center gap-2">
          <UBadge
            :color="badgeColor"
            variant="soft"
            size="sm"
          >
            <template #leading>
              <div
                class="size-1.5 rounded-full"
                :class="isRunning ? 'bg-success animate-pulse' : 'bg-current'"
              />
            </template>
            {{ statusLabel }}
          </UBadge>
        </div>
      </div>
    </template>

    <div class="space-y-1.5 text-sm">
      <div class="flex justify-between">
        <span class="text-muted">Port</span>
        <span class="text-default font-mono">{{ service.port }}</span>
      </div>

      <div class="flex justify-between">
        <span class="text-muted">Version</span>
        <span class="text-default font-mono flex items-center gap-1.5">
          <template v-if="service.version">
            {{ service.version }}
            <UBadge
              v-if="servicesStore.isActiveVersion(service.id, service.version)"
              color="success"
              variant="subtle"
              size="xs"
            >
              Active OS
            </UBadge>
          </template>
          <span v-else class="text-dimmed italic text-xs">Not installed</span>
        </span>
      </div>

      <div v-if="errorMessage" class="mt-2">
        <UTooltip :text="errorMessage">
          <div class="flex items-center gap-1.5 text-xs text-error">
            <span class="i-lucide-alert-circle size-3.5 shrink-0" />
            <span class="truncate">{{ errorMessage }}</span>
          </div>
        </UTooltip>
      </div>
    </div>

    <template #footer>
      <div class="flex flex-col gap-2">
        <UButton
          color="success"
          variant="soft"
          block
          icon="i-lucide-play"
          :loading="servicesStore.loadingStates[service.id]"
          :disabled="isNotInstalled || isRunning || isStarting"
          @click="servicesStore.startService(service.id)"
        >
          Start
        </UButton>

        <UButton
          color="error"
          variant="soft"
          block
          icon="i-lucide-square-stop"
          :disabled="isNotInstalled || !isRunning"
          :loading="servicesStore.loadingStates[service.id]"
          @click="servicesStore.stopService(service.id)"
        >
          Stop
        </UButton>

        <template v-if="!isRunning && !isStarting && hasMultipleVersions">
          <div
            v-for="ver in otherVersions"
            :key="ver"
            class="flex items-center justify-between px-3 py-1.5 rounded-md bg-muted border border-default"
          >
            <span class="text-xs font-mono text-muted">{{ ver }}</span>
            <UButton
              color="neutral"
              variant="ghost"
              size="xs"
              icon="i-lucide-arrow-left-right"
              :loading="servicesStore.switchingVersions[service.id]"
              @click="servicesStore.switchServiceVersion(service.id, ver)"
            >
              Switch
            </UButton>
          </div>
        </template>
      </div>
    </template>
  </UCard>
</template>
