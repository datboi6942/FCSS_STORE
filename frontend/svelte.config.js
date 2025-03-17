import { vitePreprocess } from '@sveltejs/vite-plugin-svelte'

/** @type {import('@sveltejs/kit').Config} */
export default {
  // Consult https://svelte.dev/docs#compile-time-svelte-preprocess
  // for more information about preprocessors
  preprocess: vitePreprocess(),
  
  // Simple compiler options - remove a11y which is causing issues
  compilerOptions: {
    // Silence any component warnings
    dev: true
  }
};
