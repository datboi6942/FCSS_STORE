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
  
  // Add this to track cart items count
  let cartItemsCount = 0;
  const API_BASE_URL = 'http://localhost:5000';
  
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
      const response = await fetch('http://localhost:5000/auth/profile', {
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
  
  // Cart polling logic
  onMount(() => {
    // Initial cart fetch
    if ($auth.isAuthenticated) {
      fetchCartCount();
    }
    
    // Set up cart polling interval
    const cartInterval = setInterval(() => {
      if ($auth.isAuthenticated) {
        console.log("Polling cart count");
        fetchCartCount();
      }
    }, 5000);
    
    // Set up auth subscription for cart updates
    const authUnsubscribe = auth.subscribe(value => {
      if (value.isAuthenticated) {
        fetchCartCount();
      } else {
        cartItemsCount = 0;
      }
    });
    
    // Clean up on component unmount
    return () => {
      clearInterval(cartInterval);
      authUnsubscribe();
    };
  });
  
  async function fetchCartCount() {
    try {
      if (!$auth.user?.id) return;
      
      const response = await fetch(`${API_BASE_URL}/cart/${$auth.user.id}`, {
        headers: {
          'Authorization': $auth.token ? `Bearer ${$auth.token}` : '',
        }
      });
      
      if (response.ok) {
        const cart = await response.json();
        if (cart && cart.items) {
          cartItemsCount = cart.items.reduce((total, item) => total + item.quantity, 0);
        }
      }
    } catch (error) {
      console.error('Error fetching cart:', error);
    }
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
        <a href="/cart" class="cart-link">
          Cart
          {#if cartItemsCount > 0}
            <span class="cart-badge">{cartItemsCount}</span>
          {/if}
        </a>
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
  
  /* Cart Badge Styling */
  .cart-link {
    position: relative;
    padding-right: 8px;
  }
  
  .cart-badge {
    position: absolute;
    top: -10px;
    right: -10px;
    background-color: #e74c3c;
    color: white;
    border-radius: 50%;
    width: 20px;
    height: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 12px;
    font-weight: bold;
  }
</style> 