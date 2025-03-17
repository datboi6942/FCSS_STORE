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
      '/api': 'http://localhost:5000',
      '/cart': 'http://localhost:5000',
      '/monero': 'http://localhost:5000',
      '/ws': {
        target: 'ws://localhost:5000',
        ws: true
      }
    }
  }
});