import { defineConfig } from 'vitest/config'
import vue from '@vitejs/plugin-vue'
import path from 'path'
import AutoImport from 'unplugin-auto-import/vite'

export default defineConfig({
  plugins: [
    vue(),
    AutoImport({
      imports: ['vue', 'pinia'],
      dts: false
    })
  ],
  test: {
    environment: 'jsdom',
    globals: true,
    alias: {
      '~': path.resolve(__dirname, './app'),
      '#shared': path.resolve(__dirname, './shared'),
    },
  },
})
