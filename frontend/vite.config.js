import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import sveltePreprocess from 'svelte-preprocess';

export default defineConfig({
  plugins: [
    svelte({
      preprocess: sveltePreprocess()
    })
  ],
  build: {
    rollupOptions: {
      input: {
        main: 'src/main.ts'
      }
    }
  },
  resolve: {
    alias: {
      $lib: '/src/lib',
      $components: '/src/components'
    }
  },
  optimizeDeps: {
    include: ['qrcode']
  },
  server: {
    proxy: {
      '/api': {
        target: 'http://192.168.6.53:8080',
        changeOrigin: true,
        rewrite: (path) => path.replace(/^\/api/, '')
      },
      '/cart': 'http://192.168.6.53:8080',
      '/monero': 'http://192.168.6.53:8080',
      '/ws': {
        target: 'ws://192.168.6.53:8080',
        ws: true
      }
    }
  }
});