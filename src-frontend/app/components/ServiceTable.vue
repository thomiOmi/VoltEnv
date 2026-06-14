<script setup lang="ts">
import { h, resolveComponent } from 'vue'
import type { ServiceDefinition } from '#shared/types/service'

const servicesStore = useServicesStore()

function formatMemory(bytes: number) {
  if (!bytes) return '0 MB'
  const mb = bytes / (1024 * 1024)
  if (mb > 1024) {
    return `${(mb / 1024).toFixed(1)} GB`
  }
  return `${mb.toFixed(0)} MB`
}

const columns = [
  {
    accessorKey: 'name',
    header: 'Service',
    cell: ({ row }: { row: { original: ServiceDefinition } }) =>
      h('div', { class: 'flex items-center gap-2' }, [
        h('span', { class: 'font-medium' }, row.original.name),
        h('span', { class: 'text-muted text-sm' }, `v${row.original.defaultVersion}`),
      ]),
  },
  {
    accessorKey: 'port',
    header: 'Port',
    cell: ({ row }: { row: { original: ServiceDefinition } }) => {
      const status = servicesStore.getStatus(row.original.id)
      return h('span', { class: 'font-mono text-xs' }, status?.port ? String(status.port) : String(row.original.port))
    },
  },
  {
    id: 'resources',
    header: 'Usage (CPU / RAM)',
    cell: ({ row }: { row: { original: ServiceDefinition } }) => {
      const usage = servicesStore.getUsage(row.original.id)
      if (!usage) return h('span', { class: 'text-dimmed text-xs' }, '-')

      return h('div', { class: 'flex items-center gap-3 font-mono text-[11px]' }, [
        h('div', { class: 'flex items-center gap-1' }, [
          h('span', { class: 'text-muted' }, 'CPU:'),
          h('span', { class: usage.cpu > 50 ? 'text-warning' : 'text-default' }, `${usage.cpu.toFixed(1)}%`)
        ]),
        h('div', { class: 'flex items-center gap-1' }, [
          h('span', { class: 'text-muted' }, 'RAM:'),
          h('span', {}, formatMemory(usage.memory))
        ])
      ])
    }
  },
  {
    accessorKey: 'status',
    header: 'Status',
    cell: ({ row }: { row: { original: ServiceDefinition } }) => {
      const status = servicesStore.getStatus(row.original.id)
      const s = status?.status ?? 'stopped'
      const color = s === 'running' ? 'success' : s === 'starting' ? 'warning' : 'neutral'
      const UBadge = resolveComponent('UBadge')
      return h(UBadge, { color, variant: 'subtle', size: 'sm', class: 'capitalize' }, { default: () => s })
    },
  },
  {
    id: 'actions',
    header: '',
    cell: ({ row }: { row: { original: ServiceDefinition } }) => {
      const id = row.original.id
      const isRun = servicesStore.isRunning(id)
      const isLoading = servicesStore.isLoading(id)
      const notInstalled = !servicesStore.isInstalled(id)
      const UButton = resolveComponent('UButton')

      return h('div', { class: 'flex items-center gap-1 justify-end' }, [
        notInstalled && !isRun
          ? h('span', { class: 'text-xs text-muted mr-1' }, 'Setup first')
          : h(
              UButton,
              {
                color: isRun ? 'error' : 'primary',
                variant: 'soft',
                size: 'sm',
                loading: isLoading,
                disabled: isLoading || (!isRun && notInstalled),
                ariaLabel: isRun ? `Stop ${row.original.name}` : `Start ${row.original.name}`,
                onClick: () => isRun ? servicesStore.stopService(id) : servicesStore.startService(id),
              },
              { default: () => isRun ? 'Stop' : 'Start' },
            ),
      ])
    },
  },
]

const data = computed(() => Array.from(servicesStore.definitions.values()))
</script>

<template>
  <UTable
    :data="data"
    :columns="columns"
    class="flex-1"
    :ui="{
      th: 'text-default font-medium text-xs',
      td: 'py-3',
    }"
  >
    <template #empty>
      <div class="flex flex-col items-center gap-2 py-8 text-muted">
        <UIcon name="i-lucide-box" class="size-8" />
        <p>No services found</p>
      </div>
    </template>
  </UTable>
</template>
