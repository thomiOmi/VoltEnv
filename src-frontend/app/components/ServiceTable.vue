<script setup lang="ts">
import { h } from 'vue'
import type { ServiceDefinition } from '#shared/types/service'

const servicesStore = useServicesStore()

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
      return h('span', {}, status?.port ? String(status.port) : String(row.original.port))
    },
  },
  {
    accessorKey: 'status',
    header: 'Status',
    cell: ({ row }: { row: { original: ServiceDefinition } }) => {
      const status = servicesStore.getStatus(row.original.id)
      const s = status?.status ?? 'stopped'
      const color = s === 'running' ? 'success' : s === 'starting' ? 'warning' : 'neutral'
      return h('UBadge', { color, variant: 'subtle', size: 'sm' }, s)
    },
  },
  {
    accessorKey: 'kind',
    header: 'Type',
    cell: ({ row }: { row: { original: ServiceDefinition } }) =>
      h('span', { class: 'text-muted text-sm' }, row.original.kind),
  },
  {
    id: 'actions',
    header: '',
    cell: ({ row }: { row: { original: ServiceDefinition } }) => {
      const id = row.original.id
      const isRun = servicesStore.isRunning(id)
      const isLoading = servicesStore.isLoading(id)
      const notInstalled = !servicesStore.isInstalled(id)

      return h('div', { class: 'flex items-center gap-1 justify-end' }, [
        notInstalled && !isRun
          ? h('span', { class: 'text-xs text-muted mr-1' }, 'Setup first')
          : h(
              'UButton',
              {
                color: isRun ? 'error' : 'primary',
                variant: 'soft',
                size: 'sm',
                loading: isLoading,
                disabled: isLoading || (!isRun && notInstalled),
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
      th: 'text-default font-medium',
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
