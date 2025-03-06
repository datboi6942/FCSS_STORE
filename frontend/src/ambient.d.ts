/// <reference types="svelte" />

declare global {
  namespace svelteHTML {
    interface HTMLAttributes<T> {
      // Add any custom attributes you need
      [key: string]: any;
    }
  }
}

export {}; 