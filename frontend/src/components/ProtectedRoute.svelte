<script>
  import { onMount } from 'svelte';
  import { navigate } from 'svelte-routing';
  import { auth } from '../stores/auth.js';
  import { apiCall } from '../api.js';
  
  export let requiredRole = null; // optional: 'admin', 'user', etc.
  
  let isAuthenticated = false;
  let userRole = '';
  let isLoading = true;
  
  onMount(async () => {
    const unsubscribe = auth.subscribe(value => {
      isAuthenticated = value.isAuthenticated;
      
      if (!isAuthenticated) {
        navigate('/login', { replace: true });
        return;
      }
      
      // If authenticated, verify token with backend
      verifyAuthentication();
    });
    
    return unsubscribe;
  });
  
  async function verifyAuthentication() {
    try {
      // Check token validity by making a request to /auth/profile
      const response = await apiCall('/auth/profile');
      
      if (response.ok) {
        const data = await response.json();
        userRole = data.role;
        
        // Check role requirement if specified
        if (requiredRole && userRole !== requiredRole) {
          navigate('/unauthorized', { replace: true });
        }
      } else {
        // Token invalid, redirect to login
        auth.logout();
        navigate('/login', { replace: true });
      }
    } catch (error) {
      console.error('Auth verification failed', error);
      auth.logout();
      navigate('/login', { replace: true });
    } finally {
      isLoading = false;
    }
  }
</script>

{#if isLoading}
  <div class="loading">Verifying authentication...</div>
{:else if isAuthenticated && (!requiredRole || userRole === requiredRole)}
  <slot />
{/if}

<style>
  .loading {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 100vh;
    font-size: 1.2rem;
  }
</style> 