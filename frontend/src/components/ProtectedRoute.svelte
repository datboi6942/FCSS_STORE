<script>
  import { onMount } from 'svelte';
  import { navigate } from 'svelte-routing';
  import { auth } from '../stores/auth.js';
  
  export let requiredRole = null; // optional: 'admin', 'user', etc.
  
  let isAuthenticated = false;
  let userRole = '';
  let isLoading = true;
  
  onMount(() => {
    console.log("ProtectedRoute: Initial auth state", $auth);
    
    // Special handling for admin tokens in localStorage
    const token = localStorage.getItem('jwt');
    if (token && token.startsWith('admin-token-')) {
      console.log("Admin token detected in localStorage");
      // Make sure auth store reflects admin status
      auth.update(state => ({
        ...state,
        isAuthenticated: true,
        token: token,
        user: {
          id: 'admin-user',
          username: 'admin',
          role: 'admin'
        },
        isAdmin: true
      }));
    }
    
    // Subscribe to auth store changes
    const unsubscribe = auth.subscribe(value => {
      console.log("ProtectedRoute: Auth state updated", value);
      isAuthenticated = value.isAuthenticated;
      userRole = value.user?.role || '';
      
      if (!isAuthenticated) {
        console.log("ProtectedRoute: Not authenticated, redirecting to login");
        navigate('/login', { replace: true });
      } else if (requiredRole && userRole !== requiredRole && !value.isAdmin) {
        console.log(`ProtectedRoute: Required role ${requiredRole} not met, got ${userRole}`);
        navigate('/unauthorized', { replace: true });
      }
      
      isLoading = false;
    });
    
    return unsubscribe;
  });
</script>

{#if isLoading}
  <div class="loading">Loading...</div>
{:else if isAuthenticated && (!requiredRole || userRole === requiredRole || $auth.isAdmin)}
  <slot></slot>
{/if}

<style>
  .loading {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 200px;
    font-size: 1.2rem;
    color: #666;
  }
</style> 
