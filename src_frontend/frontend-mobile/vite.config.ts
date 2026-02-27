import { fileURLToPath, URL } from 'node:url'

import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [vue()],
  base: "./",
  // プロジェクトルートの.envを取得
  envDir: "../../",
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  },
  build: {
    chunkSizeWarningLimit: 600,
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes('node_modules/ace-builds')) return 'vendor-ace';
          if (id.includes('node_modules/katex'))     return 'vendor-katex';
          if (id.includes('node_modules/prismjs'))   return 'vendor-prism';
          if (id.includes('node_modules/marked'))    return 'vendor-marked';
          if (id.includes('node_modules/mermaid'))   return 'vendor-mermaid';
        },
      },
    },
  },
})
