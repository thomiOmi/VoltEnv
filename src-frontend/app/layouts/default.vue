<script setup lang="ts">
import type { NavigationMenuItem } from '@nuxt/ui'

const open = ref(false)

const links = [[{
  label: 'Services',
  icon: 'i-lucide-server',
  to: '/',
  onSelect: () => { open.value = false },
}, {
  label: 'Settings',
  icon: 'i-lucide-settings',
  to: '/settings',
  onSelect: () => { open.value = false },
}], [{
  label: 'Source',
  icon: 'i-simple-icons-github',
  to: 'https://github.com/thomiOmi/voltenv',
  target: '_blank',
}]] satisfies NavigationMenuItem[][]

const groups = computed(() => [{
  id: 'links',
  label: 'Go to',
  items: links.flat(),
}])
</script>

<template>
  <UDashboardGroup unit="rem">
    <UDashboardSidebar
      id="default"
      v-model:open="open"
      collapsible
      resizable
      class="bg-elevated/25"
      :ui="{ footer: 'lg:border-t lg:border-default' }"
    >
      <template #header="{ collapsed }">
        <div class="inline-flex w-full items-center gap-3 py-2">
          <div
            class="size-8 rounded-xl bg-linear-to-br from-amber-400 to-orange-500 flex items-center justify-center text-slate-950 font-extrabold shrink-0"
          >
            V
          </div>
          <div v-if="!collapsed" class="flex flex-col min-w-0">
            <span class="text-sm font-medium text-highlighted truncate">VoltEnv</span>
            <span class="text-sm text-dimmed truncate">Environment Manager</span>
          </div>
        </div>
      </template>

      <template #default="{ collapsed }">
        <UDashboardSearchButton :collapsed="collapsed" class="bg-transparent ring-default" />

        <UNavigationMenu
          :collapsed="collapsed"
          :items="links[0]"
          orientation="vertical"
          tooltip
          popover
        />

        <UNavigationMenu
          :collapsed="collapsed"
          :items="links[1]"
          orientation="vertical"
          tooltip
          class="mt-auto"
        />
      </template>
    </UDashboardSidebar>

    <UDashboardSearch :groups="groups" />

    <slot />
  </UDashboardGroup>
</template>
