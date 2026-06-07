<script setup lang="ts">
import { h } from 'vue'
import type { ServiceDefinition } from '#shared/types/service'

const servicesStore = useServicesStore()

async function handleSetup(id: string, version: string) {
  await servicesStore.setupService(id, version)
}

const columns = [
  {
    accessorKey: 'name',
    header: 'Service',
    cell: ({ row }: { row: { original: ServiceDefinition } }) =>
      h('div', { class: 'flex items-center gap-2' }, [
        h('span', { class: 'font-medium' }, row.original.name),
      ]),
  },
  {
    accessorKey: 'defaultVersion',
    header: 'Version',
  },
  {
    accessorKey: 'port',
    header: 'Port',
  },
  {
    id: 'status',
    header: 'Status',
    cell: ({ row }: { row: { original: ServiceDefinition } }) => {
      const svc = row.original
      const isInstalled = servicesStore.isInstalled(svc.id)
      return h(
        'UBadge',
        { color: isInstalled ? 'success' : 'neutral', variant: 'subtle', size: 'sm' },
        { default: () => isInstalled ? 'Installed' : 'Not installed' },
      )
    },
  },
  {
    id: 'actions',
    header: '',
    cell: ({ row }: { row: { original: ServiceDefinition } }) => {
      const svc = row.original
      const loading = servicesStore.isLoading(svc.id)
      const progress = (servicesStore.downloadProgress as Record<string, number | undefined>)[svc.id]
      const installProgress = (servicesStore.installProgress as Record<string, number | undefined>)[svc.id]
      const activeProgress = installProgress ?? progress

      return h('div', { class: 'flex items-center gap-2 justify-end' }, [
        activeProgress !== undefined
          ? h('UProgress', { value: activeProgress, class: 'w-20', color: 'primary', size: 'sm' })
          : null,
        h(
          'UButton',
          {
            color: 'primary',
            variant: 'soft',
            size: 'sm',
            loading,
            disabled: loading,
            onClick: () => handleSetup(svc.id, svc.defaultVersion),
          },
          { default: () => 'Setup' },
        ),
      ])
    },
  },
]

const data = computed(() => servicesStore.allDefinitions)
</script>

<template>
  <div>
    <h3 class="text-lg font-semibold mb-3">
      Available Services
    </h3>
    <UTable :data="data" :columns="columns" class="flex-1">
      <template #empty>
        <div class="flex flex-col items-center gap-2 py-8 text-muted">
          <UIcon name="i-lucide-package-plus" class="size-8" />
          <p>No services in catalog</p>
        </div>
      </template>
    </UTable>
  </div>
</template>
