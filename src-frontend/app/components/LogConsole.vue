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

function handleClear() {
  logManager.clearLogs(props.serviceId, props.version)
}

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
    class="rounded-xl border border-slate-800 bg-slate-950 overflow-hidden
           flex flex-col"
  >
    <!-- Header -->
    <div
      class="flex items-center justify-between px-4 py-2.5
             border-b border-slate-800 bg-slate-900/80"
    >
      <div class="flex items-center gap-2.5">
        <div
          class="size-2 rounded-full bg-emerald-500 shadow-sm shadow-emerald-500/30"
        />
        <span class="text-sm font-semibold text-slate-200">{{ serviceName }}</span>
        <span class="text-xs text-slate-500 font-mono">{{ version }}</span>
        <span class="text-xs text-slate-600 font-mono">
          ({{ logs.length }} lines)
        </span>
      </div>
      <UButton
        color="neutral"
        variant="ghost"
        size="xs"
        icon="i-lucide-eraser"
        title="Clear Logs"
        @click="handleClear"
      />
    </div>

    <!-- Log Area -->
    <div
      ref="container"
      class="overflow-y-auto p-3 space-y-0.5 font-mono text-xs leading-relaxed
             bg-slate-950"
      style="max-height: 320px; min-height: 120px;"
    >
      <div v-if="logs.length === 0" class="text-slate-600 select-none italic">
        Waiting for logs…
      </div>
      <div
        v-for="(log, idx) in logs"
        :key="idx"
        class="whitespace-pre-wrap break-all"
        :class="log.is_error ? 'text-red-400' : 'text-slate-300'"
      >
        <span class="text-slate-500">[{{ log.timestamp }}]</span>
        {{ log.message }}
      </div>
    </div>
  </div>
</template>
