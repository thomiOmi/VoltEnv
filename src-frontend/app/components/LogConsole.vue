<script setup lang="ts">
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { ServiceLogPayload } from '#shared/types/events'

const props = defineProps<{
  serviceId: string
  version: string
}>()

const servicesStore = useServicesStore()
const logManager = useLogManagerStore()

const logs = ref<ServiceLogPayload[]>([])
const container = ref<HTMLElement | null>(null)

const key = computed(() => `${props.serviceId}:${props.version}`)
const storeLogs = computed(() => logManager.logs[key.value] ?? [])

const displayLogs = computed(() => {
  const map = new Map<string, ServiceLogPayload>()
  for (const log of logs.value) {
    map.set(`${log.timestamp}:${log.message}`, log)
  }
  for (const log of storeLogs.value) {
    map.set(`${log.timestamp}:${log.message}`, log)
  }
  return Array.from(map.values()).sort(
    (a, b) => new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime(),
  )
})

const serviceName = computed(() => {
  const s = servicesStore.services.find(s => s.id === props.serviceId)
  return s?.name ?? props.serviceId
})

let unlisten: UnlistenFn | null = null

onMounted(() => {
  listen<ServiceLogPayload>('service-log', (event) => {
    const { service_id, version } = event.payload
    if (service_id === props.serviceId && version === props.version) {
      logs.value.push(event.payload)
    }
  }).then((fn) => {
    unlisten = fn
  })
})

onUnmounted(() => {
  unlisten?.()
})

watch(
  () => displayLogs.value.length,
  async () => {
    await nextTick()
    if (container.value) {
      container.value.scrollTop = container.value.scrollHeight
    }
  },
)
</script>

<template>
  <div
    class="rounded-xl border border-default bg-elevated/50 overflow-hidden flex flex-col"
  >
    <div class="flex items-center justify-between px-4 py-2.5 border-b border-default bg-elevated">
      <div class="flex items-center gap-2.5 min-w-0">
        <div class="size-2 rounded-full bg-success shadow-sm shadow-success/30 shrink-0" />
        <span class="text-sm font-semibold text-highlighted truncate">{{ serviceName }}</span>
        <span class="text-xs text-dimmed font-mono truncate">{{ version }}</span>
        <span class="text-xs text-dimmed font-mono shrink-0">
          ({{ displayLogs.length }} lines)
        </span>
      </div>
      <UButton
        color="neutral"
        variant="ghost"
        size="xs"
        icon="i-lucide-eraser"
        title="Clear Logs"
        @click="logManager.clearLogs(serviceId, version); logs = []"
      />
    </div>

    <div
      ref="container"
      class="overflow-y-auto p-3 space-y-0.5 font-mono text-xs leading-relaxed"
      style="max-height: 320px; min-height: 120px;"
    >
      <div v-if="displayLogs.length === 0" class="text-dimmed select-none italic">
        Waiting for logs…
      </div>
      <div
        v-for="(log, idx) in displayLogs"
        :key="idx"
        class="whitespace-pre-wrap break-all"
        :class="log.is_error ? 'text-error' : 'text-muted'"
      >
        <span class="text-dimmed">[{{ log.timestamp }}]</span>
        {{ log.message }}
      </div>
    </div>
  </div>
</template>
