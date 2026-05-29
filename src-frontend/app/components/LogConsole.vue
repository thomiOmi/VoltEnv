<script setup lang="ts">
const props = defineProps<{
  serviceId: string
  version: string
}>()

const logManager = useLogManagerStore()
const servicesStore = useServicesStore()

const logs = computed(() => {
  const key = `${props.serviceId}:${props.version}`
  return logManager.logs[key] ?? []
})

const serviceName = computed(() => {
  const s = servicesStore.services.find(s => s.id === props.serviceId)
  return s?.name ?? props.serviceId
})

const container = ref<HTMLElement | null>(null)

watch(
  () => logs.value.length,
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
    <!-- Header -->
    <div class="flex items-center justify-between px-4 py-2.5 border-b border-default bg-elevated">
      <div class="flex items-center gap-2.5 min-w-0">
        <div class="size-2 rounded-full bg-success shadow-sm shadow-success/30 shrink-0" />
        <span class="text-sm font-semibold text-highlighted truncate">{{ serviceName }}</span>
        <span class="text-xs text-dimmed font-mono truncate">{{ version }}</span>
        <span class="text-xs text-dimmed font-mono shrink-0">
          ({{ logs.length }} lines)
        </span>
      </div>
      <UButton
        color="neutral"
        variant="ghost"
        size="xs"
        icon="i-lucide-eraser"
        title="Clear Logs"
        @click="logManager.clearLogs(serviceId, version)"
      />
    </div>

    <!-- Log Area -->
    <div
      ref="container"
      class="overflow-y-auto p-3 space-y-0.5 font-mono text-xs leading-relaxed bg-inverted"
      style="max-height: 320px; min-height: 120px;"
    >
      <div v-if="logs.length === 0" class="text-dimmed select-none italic">
        Waiting for logs…
      </div>
      <div
        v-for="(log, idx) in logs"
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
