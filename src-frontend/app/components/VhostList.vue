<script setup lang="ts">
const api = useServiceApi()
const toast = useToast()
const vhosts = ref<any[]>([])
const loading = ref(false)
const runningComposer = ref<string | null>(null)

async function loadVhosts() {
  loading.value = true
  try {
    vhosts.value = await api.listVhosts()
  } finally {
    loading.value = false
  }
}

async function handleDelete(domain: string) {
  if (!confirm(`Delete vhost for ${domain}?`)) return
  try {
    await api.deleteVhost(domain)
    await loadVhosts()
    toast.add({ title: 'Vhost deleted', color: 'success' })
  } catch (e) {
    toast.add({ title: 'Delete failed', description: String(e), color: 'error' })
  }
}

async function runComposer(path: string, args: string[]) {
  runningComposer.value = `${path}-${args.join(' ')}`
  try {
    const output = await api.runComposerCommand(path, args)
    toast.add({
        title: 'Composer Success',
        description: output.substring(0, 100) + '...',
        color: 'success'
    })
    console.log('Composer Output:', output)
  } catch (e) {
    toast.add({ title: 'Composer Error', description: String(e), color: 'error' })
  } finally {
    runningComposer.value = null
  }
}

onMounted(loadVhosts)
</script>

<template>
  <div class="space-y-2">
    <div v-if="vhosts.length === 0" class="text-sm text-muted py-4 text-center">
      No projects created yet.
    </div>

    <div
      v-for="v in vhosts"
      :key="v.domain"
      class="flex flex-col p-3 rounded border border-default bg-elevated/50 gap-2"
    >
      <div class="flex items-center justify-between">
        <div>
          <div class="flex items-center gap-2">
            <span class="font-medium text-sm">{{ v.domain }}</span>
            <UBadge v-if="v.ssl" color="success" size="xs" variant="subtle">HTTPS</UBadge>
          </div>
          <p class="text-xs text-muted truncate max-w-xs" :title="v.root">{{ v.root }}</p>
        </div>
        <div class="flex items-center gap-1">
          <UButton
            variant="ghost"
            color="neutral"
            icon="i-lucide-external-link"
            size="xs"
            @click="window.open(`http${v.ssl ? 's' : ''}://${v.domain}`, '_blank')"
          />
          <UButton
            variant="ghost"
            color="error"
            icon="i-lucide-trash-2"
            size="xs"
            @click="handleDelete(v.domain)"
          />
        </div>
      </div>

      <div class="flex items-center gap-2 mt-1 border-t border-default pt-2">
        <span class="text-[10px] uppercase font-bold text-muted mr-auto">Composer</span>
        <UButton
          variant="outline"
          size="xs"
          label="Install"
          :loading="runningComposer === `${v.root}-install`"
          @click="runComposer(v.root, ['install'])"
        />
        <UButton
          variant="outline"
          size="xs"
          label="Update"
          :loading="runningComposer === `${v.root}-update`"
          @click="runComposer(v.root, ['update'])"
        />
      </div>
    </div>
  </div>
</template>
