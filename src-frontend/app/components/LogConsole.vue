<script setup lang="ts">
import { useLogManagerStore } from '~/stores/logManager'

const props = defineProps<{
  serviceId: string
  version: string
}>()

const logManager = useLogManagerStore()

const container = ref<HTMLElement | null>(null)

const displayLogs = computed(() => logManager.getLogs(props.serviceId, props.version))

watch(
  () => displayLogs.value.length,
  async () => {
    await nextTick()
    if (container.value) {
      container.value.scrollTop = container.value.scrollHeight
    }
  },
)

function handleClear() {
  logManager.clearLogs(props.serviceId, props.version)
}
</script>

<template>
  <div class="rounded-xl border border-default bg-elevated/50 overflow-hidden flex flex-col">
    <div class="flex items-center justify-between px-4 py-2.5 border-b border-default bg-elevated">
      <div class="flex items-center gap-2.5 min-w-0">
        <div class="size-2 rounded-full bg-success shadow-sm shadow-success/30 shrink-0" />
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
        aria-label="Clear logs"
        @click="handleClear"
      />
    </div>

    <div
      ref="container"
      class="overflow-y-auto p-3 space-y-0.5 font-mono text-xs leading-relaxed"
      style="max-height: 320px; min-height: 120px;"
      aria-live="polite"
      aria-atomic="false"
    >
      <div v-if="displayLogs.length === 0" class="text-dimmed select-none italic">
        Waiting for logs…
      </div>
      <div
        v-for="(log, idx) in displayLogs"
        :key="idx"
        class="whitespace-pre-wrap break-all"
        :class="log.isError ? 'text-error' : 'text-muted'"
      >
        <span class="text-dimmed">[{{ log.timestamp }}]</span>
        {{ log.message }}
      </div>
    </div>
  </div>
</template>
