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
    // ace-builds は最小化後も 560 kB（gzip: 157 kB）あるため閾値を引き上げる
    chunkSizeWarningLimit: 600,
    rollupOptions: {
      output: {
        // 大きなライブラリを個別ベンダーチャンクに分離して index チャンクを縮小する
        manualChunks(id) {
          if (id.includes('node_modules/ace-builds')) return 'vendor-ace';
          if (id.includes('node_modules/katex'))     return 'vendor-katex';
          if (id.includes('node_modules/prismjs'))   return 'vendor-prism';
          if (id.includes('node_modules/marked'))    return 'vendor-marked';
        },
      },
    },
  },
})
