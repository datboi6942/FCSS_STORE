<script>
  import { auth } from '../stores/auth.js';
  import { onMount } from 'svelte';
  
  let isLoggedIn = false;
  let username = '';
  let isAdmin = false;
  
  // Check if connection is from localhost
  const isLocalhost = 
    window.location.hostname === 'localhost' || 
    window.location.hostname === '127.0.0.1';
  
  // Subscribe to auth store changes
  onMount(() => {
    const unsubscribe = auth.subscribe(authData => {
      console.log("Nav: Auth data updated", authData);
      isLoggedIn = authData.isAuthenticated;
      
      if (authData.isAuthenticated) {
        // First check if this is admin directly from the auth store
        if (authData.user && authData.user.role === 'admin') {
          console.log("Nav: Admin user detected from auth store");
          username = authData.user.username || 'Admin';
          isAdmin = true;
        } 
        // For non-admin or to double-check, fetch profile
        else if (authData.token) {
          fetchUserProfile(authData.token);
        }
      }
    });
    
    return unsubscribe;
  });
  
  async function fetchUserProfile(token) {
    console.log("Nav: Fetching user profile with token", token);
    
    try {
      // Special case for admin token
      if (token.startsWith('admin-token')) {
        console.log("Nav: Admin token detected, skipping server validation");
        username = 'Administrator';
        isAdmin = true;
        return;
      }
      
      // Regular token - fetch from server
      const response = await fetch('http://localhost:8443/auth/profile', {
        headers: {
          'Authorization': `Bearer ${token}`
        }
      });
      
      if (response.ok) {
        const userData = await response.json();
        username = userData.username || 'User';
        isAdmin = userData.role === 'admin';
        console.log("Nav: User profile fetched", username, isAdmin);
      }
    } catch (error) {
      console.error('Failed to fetch user profile', error);
    }
  }
  
  function logout() {
    auth.logout();
  }
</script>

<nav>
  <div class="container">
    <a href="/" class="logo">Secure Store</a>
    
    <div class="nav-links">
      <a href="/">Home</a>
      <a href="/products">Products</a>
      
      {#if isLoggedIn}
        <a href="/orders">My Orders</a>
        
        <!-- Only show admin link if user is admin AND on localhost -->
        {#if isAdmin && isLocalhost}
          <a href="/admin">Admin Panel</a>
        {/if}
        
        <a href="/profile">{username}</a>
        <button on:click={logout}>Logout</button>
      {:else}
        <a href="/login">Login</a>
      {/if}
    </div>
  </div>
</nav>

<style>
  nav {
    background-color: #333;
    color: white;
    padding: 1rem 0;
  }
  
  .container {
    max-width: 1200px;
    margin: 0 auto;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  
  .logo {
    font-size: 1.5rem;
    font-weight: bold;
    color: white;
    text-decoration: none;
  }
  
  .nav-links {
    display: flex;
    gap: 1.5rem;
    align-items: center;
  }
  
  a {
    color: white;
    text-decoration: none;
  }
  
  a:hover {
    text-decoration: underline;
  }
  
  button {
    background-color: #f44336;
    color: white;
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    cursor: pointer;
  }
</style> 