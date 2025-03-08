<script>
  import { onMount } from 'svelte';
  import { navigate } from 'svelte-routing';
  import { auth } from '../stores/auth.js';
  import { apiCall } from '../api.js';
  
  export let requiredRole = null; // optional: 'admin', 'user', etc.
  
  let isAuthenticated = false;
  let userRole = '';
  let isLoading = true;
  let isAdmin = false;
  
  // Check if connection is from localhost
  const isLocalhost = 
    window.location.hostname === 'localhost' || 
    window.location.hostname === '127.0.0.1';
  
  onMount(async () => {
    const unsubscribe = auth.subscribe(value => {
      console.log("ProtectedRoute: Auth state updated", value);
      isAuthenticated = value.isAuthenticated;
      
      if (!isAuthenticated) {
        console.log("ProtectedRoute: Not authenticated, redirecting to login");
        navigate('/login', { replace: true });
        return;
      }
      
      // Check for admin token directly if admin token pattern is detected
      if (value.token && value.token.startsWith('admin-token')) {
        console.log("ProtectedRoute: Admin token detected");
        userRole = 'admin';
        isAdmin = true;
        isLoading = false;
        
        // Check if admin route and localhost
        if (requiredRole === 'admin' && !isLocalhost) {
          console.log("ProtectedRoute: Admin attempt from non-localhost");
          navigate('/unauthorized', { replace: true });
        }
        return;
      }
      
      // Regular token - verify with backend
      verifyAuthentication();
    });
    
    return unsubscribe;
  });
  
  async function verifyAuthentication() {
    try {
      console.log("ProtectedRoute: Verifying authentication with backend");
      // Check token validity by making a request to /auth/profile
      const response = await apiCall('/auth/profile');
      
      if (response.ok) {
        const data = await response.json();
        userRole = data.role;
        isAdmin = userRole === 'admin';
        
        console.log("ProtectedRoute: User verified, role:", userRole);
        
        // For admin role, additionally check if access is from localhost
        if (requiredRole === 'admin') {
          if (userRole !== 'admin' || !isLocalhost) {
            console.log("ProtectedRoute: Admin access denied");
            // Silently redirect without indicating why
            navigate('/unauthorized', { replace: true });
            return;
          }
        }
        // For other roles, just check role requirement if specified
        else if (requiredRole && userRole !== requiredRole) {
          console.log("ProtectedRoute: Required role not met");
          navigate('/unauthorized', { replace: true });
        }
      } else {
        console.log("ProtectedRoute: Token invalid");
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
{:else if isAuthenticated && (
  (requiredRole === 'admin' && isAdmin && isLocalhost) || 
  (requiredRole !== 'admin' && (!requiredRole || userRole === requiredRole))
)}
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
