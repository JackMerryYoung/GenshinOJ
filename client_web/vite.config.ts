import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import topLevelAwait from 'vite-plugin-top-level-await';

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [react(), topLevelAwait()],
  build: {
    // Top-level await is an ES2022 feature; the default target (es2020) can't
    // transform the destructuring emitted by vite-plugin-top-level-await.
    target: 'es2022',
  },
  server: {
    watch: {
      usePolling: true,
    },
    hmr: true,
    host: '0.0.0.0',
    proxy: {
      '/wsapi': {
        target: 'ws://localhost:9983/ws',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/wsapi/, '')
      },
      '/avatar': {
        target: 'http://localhost:9983',
        changeOrigin: true
      }
    },
    allowedHosts: true
  },
})

