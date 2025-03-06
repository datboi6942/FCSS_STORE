<script>
  import { auth } from '../stores/auth.js';
  import { onMount } from 'svelte';
  
  let isLoggedIn = false;
  let username = '';
  let isAdmin = false;
  
  // Subscribe to auth store changes
  onMount(() => {
    const unsubscribe = auth.subscribe(authData => {
      isLoggedIn = authData.isAuthenticated;
      
      // If we have a token, fetch user profile
      if (authData.isAuthenticated) {
        fetchUserProfile(authData.token);
      }
    });
    
    return unsubscribe;
  });
  
  async function fetchUserProfile(token) {
    try {
      const response = await fetch('http://localhost:8443/auth/profile', {
        headers: {
          'Authorization': `Bearer ${token}`
        }
      });
      
      if (response.ok) {
        const userData = await response.json();
        username = userData.username || 'User';
        isAdmin = userData.role === 'admin';
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
        {#if isAdmin}
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