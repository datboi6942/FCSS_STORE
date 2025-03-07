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
  }
});