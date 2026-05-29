<script setup lang="ts">
const colorMode = useColorMode()

const color = computed(() => colorMode.value === 'dark' ? '#0f172a' : 'white')

useHead({
  meta: [
    { charset: 'utf-8' },
    { name: 'viewport', content: 'width=device-width, initial-scale=1' },
    { key: 'theme-color', name: 'theme-color', content: color },
  ],
  htmlAttrs: {
    lang: 'en',
  },
})

const title = 'VoltEnv'
const description = 'Local Development Environment Manager'

useSeoMeta({
  title,
  description,
  ogTitle: title,
  ogDescription: description,
})

const servicesStore = useServicesStore()
const logManagerStore = useLogManagerStore()

onMounted(() => {
  servicesStore.init()
  logManagerStore.initLogListener()
})
</script>

<template>
  <UApp>
    <NuxtLoadingIndicator />

    <NuxtLayout>
      <NuxtPage />
    </NuxtLayout>
  </UApp>
</template>
