<script setup lang="ts">
const props = defineProps<{
  service: Service
}>()

const servicesStore = useServicesStore()

const isRunning = computed(() => props.service.status === 'Running')
const isStarting = computed(() => props.service.status === 'Starting')
const displayStatus = computed(() => {
  if (isRunning.value) return 'Running'
  if (isStarting.value) return 'Starting'
  if (typeof props.service.status === 'object' && 'Error' in props.service.status) {
    return 'Error'
  }
  return 'Stopped'
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
  <div
    class="rounded-xl border border-default bg-elevated/50 hover:border-accented transition-all duration-300 overflow-hidden"
  >
    <!-- Header -->
    <div class="p-4 pb-2">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2.5">
          <div
            class="size-2.5 rounded-full"
            :class="isRunning
              ? 'bg-success animate-pulse shadow-lg shadow-success/30'
              : 'bg-error/50'"
          />
          <span class="font-semibold text-sm text-highlighted">{{ service.name }}</span>
        </div>
        <UBadge
          :color="isRunning ? 'success' : isStarting ? 'warning' : 'neutral'"
          variant="soft"
          size="sm"
        >
          <template #leading>
            <div
              class="size-1.5 rounded-full"
              :class="isRunning ? 'bg-success animate-pulse' : 'bg-current'"
            />
          </template>
          {{ displayStatus }}
        </UBadge>
      </div>
    </div>

    <!-- Body -->
    <div class="px-4 pb-3 space-y-1.5 text-sm">
      <div class="flex justify-between">
        <span class="text-muted">Port</span>
        <span class="text-default font-mono">{{ service.port }}</span>
      </div>
      <div class="flex justify-between">
        <span class="text-muted">Version</span>
        <span class="text-default font-mono flex items-center gap-1.5">
          {{ service.version }}
          <UBadge
            v-if="servicesStore.isActiveVersion(service.id, service.version)"
            color="success"
            variant="subtle"
            size="xs"
          >
            Active OS
          </UBadge>
        </span>
      </div>
    </div>

    <!-- Footer actions -->
    <div class="px-4 pb-4 pt-1 flex flex-col gap-2">
      <!-- Start -->
      <UButton
        v-if="!isRunning"
        color="success"
        variant="soft"
        block
        icon="i-lucide-play"
        :loading="servicesStore.loadingStates[service.id]"
        @click="servicesStore.startService(service.id)"
      >
        Start
      </UButton>

      <!-- Stop -->
      <UButton
        v-if="isRunning"
        color="error"
        variant="soft"
        block
        icon="i-lucide-square-stop"
        :loading="servicesStore.loadingStates[service.id]"
        @click="servicesStore.stopService(service.id)"
      >
        Stop
      </UButton>

      <!-- Version switch -->
      <template v-if="!isRunning && hasMultipleVersions">
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
  </div>
</template>
