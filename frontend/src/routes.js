export default [
  // ... other routes
  {
    path: '/checkout/monero',
    component: () => import('./routes/checkout/monero/+page.svelte')
  }
]; 