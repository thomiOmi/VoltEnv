// https://nuxt.com/docs/api/configuration/nuxt-config
// https://v2.tauri.app/start/frontend/nuxt/
export default defineNuxtConfig({
  compatibilityDate: '2026-05-26',
  devtools: {
    enabled: true
  },

  css: ['~/assets/css/main.css'],

  // Enable SSG (Tauri doesn't support server-based solutions)
  ssr: false,

  modules: ['@nuxt/ui', '@pinia/nuxt', '@nuxt/eslint', '@vueuse/nuxt'],

  vite: {
    // Better support for Tauri CLI output
    clearScreen: false,
    // Enable environment variables
    envPrefix: ['VITE_', 'TAURI_'],
    server: {
      // Tauri requires a consistent port
      strictPort: true,
    },
  },

  // Avoids file watcher errors with Tauri's Rust source
  ignore: ['**/src-tauri/**'],

  eslint: {
    config: {
      stylistic: true,
      formatters: true,
    }
  }
})