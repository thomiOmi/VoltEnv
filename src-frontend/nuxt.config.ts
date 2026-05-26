// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  compatibilityDate: '2026-05-26',

  // Tauri v2 + Nuxt integration
  // https://v2.tauri.app/start/frontend/nuxt/

  // Enable SSG (Tauri doesn't support server-based solutions)
  ssr: false,

  modules: [
    '@nuxt/ui',
    '@pinia/nuxt',
  ],

  // Enables the development server to be discoverable by WebView2
  // on Windows (default listens on IPv6 ::1 only)
  devServer: {
    host: '0.0.0.0',
  },

  // Path mapping
  srcDir: 'app/',

  vite: {
    // Better support for Tauri CLI output
    clearScreen: false,
    // Enable environment variables
    envPrefix: ['VITE_', 'TAURI_'],
    server: {
      // Tauri requires a consistent port
      strictPort: true,
    },
    optimizeDeps: {
      // Prevent 504 (Outdated Optimize Dep) on Windows + Bun
      exclude: ['@iconify/vue'],
    },
  },

  // Avoids file watcher errors with Tauri's Rust source
  ignore: ['**/src-tauri/**'],
})
