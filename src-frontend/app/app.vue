<script setup lang="ts">
const servicesStore = useServicesStore()

onMounted(() => {
  servicesStore.init()
})
</script>

<template>
  <div class="min-h-screen bg-slate-950 text-slate-100">
    <div class="max-w-6xl mx-auto px-6 py-8">
      <!-- Header -->
      <header class="flex items-center justify-between mb-10">
        <div class="flex items-center gap-3">
          <div
            class="size-10 rounded-xl bg-gradient-to-br from-amber-400 to-orange-500
                   flex items-center justify-center text-slate-950 font-extrabold text-lg
                   shadow-lg shadow-orange-500/20"
          >
            V
          </div>
          <div>
            <h1 class="text-2xl font-bold bg-gradient-to-r from-amber-400 to-orange-500 bg-clip-text text-transparent">
              VoltEnv
            </h1>
            <p class="text-xs text-slate-500">
              Local Development Environment Manager
            </p>
          </div>
        </div>
        <UButton
          color="neutral"
          variant="ghost"
          square
          icon="i-lucide-refresh-cw"
          @click="servicesStore.fetchServicesStatus()"
        />
      </header>

      <!-- 3-Column Grid -->
      <div class="grid grid-cols-1 lg:grid-cols-3 gap-6">
        <!-- Col 1: Sidebar -->
        <div class="lg:col-span-1 space-y-4">
          <div class="rounded-xl border border-slate-800 bg-slate-900/50 backdrop-blur-sm p-5">
            <h2 class="text-sm font-semibold text-slate-400 uppercase tracking-wider mb-4">
              System Overview
            </h2>
            <div class="space-y-4">
              <div class="flex items-center justify-between">
                <span class="text-sm text-slate-400">Total Services</span>
                <span class="text-2xl font-bold text-slate-100">{{ servicesStore.services.length }}</span>
              </div>
              <div class="flex items-center justify-between">
                <span class="text-sm text-slate-400">Running</span>
                <span class="text-2xl font-bold text-emerald-400">{{ servicesStore.services.filter(s => s.status === 'Running').length }}</span>
              </div>
              <div class="flex items-center justify-between">
                <span class="text-sm text-slate-400">Stopped</span>
                <span class="text-2xl font-bold text-rose-400">{{ servicesStore.services.filter(s => s.status === 'Stopped').length }}</span>
              </div>
              <hr class="border-slate-800">
              <div>
                <p class="text-xs text-slate-500 mb-1">
                  Data Directory
                </p>
                <code class="text-xs text-slate-300 bg-slate-800/50 px-2 py-1 rounded block truncate">~/.voltenv/</code>
              </div>
            </div>
          </div>
        </div>

        <!-- Cols 2-3: Service Cards -->
        <div class="lg:col-span-2 space-y-4">
          <div class="flex items-center justify-between">
            <h2 class="text-sm font-semibold text-slate-400 uppercase tracking-wider">
              Local Services
            </h2>
            <span class="text-xs text-slate-500">{{ servicesStore.services.length }} service(s) configured</span>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div
              v-for="service in servicesStore.services"
              :key="service.id"
              class="rounded-xl border border-slate-800 bg-slate-900/50 backdrop-blur-sm
                     hover:border-slate-700 transition-all duration-300 overflow-hidden"
            >
              <!-- Header -->
              <div class="p-4 pb-2">
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-2">
                    <div
                      class="size-2.5 rounded-full"
                      :class="service.status === 'Running'
                        ? 'bg-emerald-400 animate-pulse shadow-lg shadow-emerald-400/30'
                        : 'bg-rose-500/50'"
                    />
                    <span class="font-semibold text-sm text-slate-100">{{ service.name }}</span>
                  </div>
                  <span
                    class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-md text-xs font-medium"
                    :class="service.status === 'Running'
                      ? 'bg-emerald-500/10 text-emerald-400 border border-emerald-500/20'
                      : 'bg-rose-500/10 text-rose-400 border border-rose-500/20'"
                  >
                    <span
                      class="size-1.5 rounded-full"
                      :class="service.status === 'Running' ? 'bg-emerald-400 animate-pulse' : 'bg-rose-400'"
                    />
                    {{ service.status === 'Running' ? 'Running' : 'Stopped' }}
                  </span>
                </div>
              </div>

              <!-- Body -->
              <div class="px-4 pb-3 space-y-1.5 text-sm">
                <div class="flex justify-between">
                  <span class="text-slate-500">Port</span>
                  <span class="text-slate-300 font-mono">{{ service.port }}</span>
                </div>
              </div>

              <!-- Footer -->
              <div class="px-4 pb-4 pt-1">
                <UButton
                  v-if="service.status === 'Stopped'"
                  color="success"
                  variant="soft"
                  block
                  icon="i-lucide-play"
                  :loading="servicesStore.loadingStates[service.id]"
                  @click="servicesStore.startService(service.id)"
                >
                  Start
                </UButton>
                <UButton
                  v-if="service.status === 'Running'"
                  color="error"
                  variant="soft"
                  block
                  icon="i-lucide-square-stop"
                  :loading="servicesStore.loadingStates[service.id]"
                  @click="servicesStore.stopService(service.id)"
                >
                  Stop
                </UButton>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Footer -->
      <footer class="mt-16 pt-6 border-t border-slate-800 text-center">
        <p class="text-xs text-slate-600">
          VoltEnv v0.1.0 &mdash; Blazing Fast Local Development Environment
        </p>
      </footer>
    </div>
  </div>
</template>

<style>
body {
  margin: 0;
  background-color: #020617;
}
</style>
