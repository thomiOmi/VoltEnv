// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  // Nuxt 4 compatibility
  future: {
    compatibilityVersion: 4,
  },

  // SSR must be false for Tauri SSG
  ssr: false,

  // Output must be export for Tauri SSG
  // @ts-ignore
  output: 'export',

  // Dev server port for Tauri
  devServer: {
    port: 1420,
  },

  modules: [
    '@nuxt/ui',
    '@pinia/nuxt',
  ],

  nitro: {
    output: {
      publicDir: '../dist'
    }
  },

  // Path mapping
  srcDir: 'app/',
})
