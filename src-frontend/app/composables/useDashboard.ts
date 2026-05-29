export const useDashboard = createSharedComposable(() => {
  const route = useRoute()
  const router = useRouter()
  const sidebarOpen = ref(false)

  defineShortcuts({
    'g-h': () => router.push('/'),
  })

  watch(() => route.fullPath, () => {
    sidebarOpen.value = false
  })

  return {
    sidebarOpen,
  }
})
