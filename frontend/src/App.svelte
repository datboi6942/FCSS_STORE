<script lang="ts">
  import { config } from './config.js';  // Import config at the top
  import { writable } from 'svelte/store';
  import NavBar from './components/NavBar.svelte';
  import Home from './components/Home.svelte';
  import Products from './components/Products.svelte';
  import Orders from './components/Orders.svelte';
  import Chat from './components/Chat.svelte';
  import Login from './components/Login.svelte';
  import { Router, Route } from "svelte-routing";
  import { onMount } from 'svelte';
  import { auth } from './stores/auth.js';
  import { cart, cartTotal } from './stores/cart.js';
  
  // Import missing components
  import Nav from './components/Nav.svelte';
  import ProtectedRoute from './components/ProtectedRoute.svelte';
  import UserProfile from './components/UserProfile.svelte';
  import ProductList from './components/Products.svelte';
  import Cart from './components/Cart.svelte';
  import CartDrawer from './components/CartDrawer.svelte';
  import ShippingFormOverlay from './components/ShippingFormOverlay.svelte';
  import MoneroCheckout from './routes/MoneroCheckout.svelte';
  import AdminProducts from './components/AdminProducts.svelte';
  import AdminUsers from './components/AdminUsers.svelte';
  import OrderValidator from './components/OrderValidator.svelte';
  
  // Dynamically import AdminPanel
  import { fade } from 'svelte/transition';
  let AdminPanel;
  
  let serverReady = false;
  let showShippingForm = false;
  let isCartOpen = false;
  
  // Add api import
  import { api } from './services/api.js';
  
  // Update checkServerReadiness function
  async function checkServerReadiness() {
    // Check if we have a cached result from a recent health check (15 seconds)
    const cached = sessionStorage.getItem('serverReadyCached');
    const cachedTimestamp = sessionStorage.getItem('serverReadyTimestamp');
    if (cached && cachedTimestamp) {
      const diff = Date.now() - parseInt(cachedTimestamp);
      if (diff < 15000) { // use cached result if within 15 seconds
        serverReady = cached === 'true';
        console.log("Using cached health check result:", serverReady);
        return serverReady;
      }
    }

    const maxRetries = 3;
    const baseDelay = 2000;
    let isReady = false;
    
    for (let i = 0; i < maxRetries; i++) {
      try {
        const result = await api.health.check();
        console.log(`Server is ready (attempt ${i + 1})`);
        isReady = true;
        break;
      } catch (err) {
        console.log(`Health check attempt ${i + 1} failed: ${err}`);
      }
      // Increase delay with each retry
      await new Promise(resolve => setTimeout(resolve, baseDelay * (i + 1)));
    }
    
    if (!isReady) {
      console.warn("Health check failed repeatedly – entering offline mode.");
      // Optionally, you can display an alert or fallback here
    }

    // Cache the result with the current timestamp
    sessionStorage.setItem('serverReadyCached', isReady.toString());
    sessionStorage.setItem('serverReadyTimestamp', Date.now().toString());

    serverReady = isReady;
    return isReady;
  }
  
  onMount(async () => {
    console.log("App component mounted, checking authentication...");
    
    // Check server first to ensure it's up
    await checkServerReadiness();
    
    // Try to restore auth from storage first
    const token = localStorage.getItem('jwt') || sessionStorage.getItem('jwt');
    const savedUser = localStorage.getItem('user');
    
    if (token && savedUser) {
      try {
        const userData = JSON.parse(savedUser);
        auth.update(state => ({
          ...state,
          isAuthenticated: true,
          token,
          user: userData,
          isAdmin: userData.role === 'admin'
        }));
      } catch (e) {
        console.error("Error restoring auth state:", e);
      }
    }
    
    // Then verify with backend
    const isAuthenticated = await auth.checkAuth();
    console.log("Authentication check result:", isAuthenticated);
    
    // Add event listener for showShipping
    const handleShowShipping = () => {
      console.log("showShipping event received in App.svelte");
      showShippingForm = true;
    };
    
    document.addEventListener('showShipping', handleShowShipping);
    
    // Cleanup
    return () => {
      document.removeEventListener('showShipping', handleShowShipping);
    };
    
    // Check if we need to restore auth from backup token
    const tokenBackup = localStorage.getItem('auth_token_backup');
    if (tokenBackup && (!$auth || !$auth.token !== tokenBackup)) {
      console.log("Restoring authentication from backup token in App");
      auth.update(state => ({
        ...state,
        isAuthenticated: true,
        token: tokenBackup
      }));
      
      // Check auth state with backend
      try {
        const userData = await api.auth.profile();
        
        // Update auth with user data
        auth.update(state => ({
          ...state,
          user: userData,
          isAdmin: userData.role === 'admin'
        }));
        console.log("Successfully restored user profile from token");
      } catch (e) {
        console.error("Error restoring auth from token:", e);
        localStorage.removeItem('auth_token_backup');
      }
    }

    // Add event handler for page unloads to save auth state
    const handleBeforeUnload = () => {
      if ($auth && $auth.isAuthenticated && $auth.token) {
        localStorage.setItem('auth_token_backup', $auth.token);
        if ($auth.user) {
          localStorage.setItem('auth_user_backup', JSON.stringify($auth.user));
        }
      }
    };
    
    window.addEventListener('beforeunload', handleBeforeUnload);
    
    return () => {
      window.removeEventListener('beforeunload', handleBeforeUnload);
      // Other cleanup...
    };
  });
  
  // For compatibility with the old navigation system
  const currentView = writable('home');
  
  function setView(view) {
    currentView.set(view);
  }

  // Check if user is authenticated on page load
  onMount(() => {
    // Check for both regular JWT and backup token
    const token = localStorage.getItem('jwt') || localStorage.getItem('auth_token_backup');
    if (token) {
      console.log("Restoring authentication from token:", token.substring(0, 10) + "...");
      
      // Set auth state immediately with the token
      auth.update(state => ({
        ...state,
        isAuthenticated: true,
        token: token
      }));
      
      // Verify token with backend
      fetch(config.api.profile, {
        headers: {
          'Authorization': `Bearer ${token}`
        }
      }).then(response => {
        if (response.ok) {
          return response.json();
        } else {
          throw new Error('Invalid token');
        }
      }).then(data => {
        console.log("User profile loaded successfully:", data.username);
        auth.update(state => ({
          ...state,
          user: data,
          isAdmin: data.role === 'admin'
        }));
        
        // If this was from a backup token, promote it to be the main JWT
        if (localStorage.getItem('auth_token_backup') === token) {
          localStorage.setItem('jwt', token);
        }
      }).catch(error => {
        console.error('Auth check failed', error);
        // Only clear if it's not the backup token during checkout
        if (!window.location.pathname.includes('/checkout/')) {
          auth.logout();
        }
      });
    }
  });

  function handleToggleCart() {
    isCartOpen = !isCartOpen;
  }

  import OrderStatus from './components/OrderStatus.svelte';
  import AdminOrders from './components/AdminOrders.svelte';

  // Add a new function to handle shipping form display
  function handleShowShipping() {
    console.log("Showing shipping form");
    showShippingForm = true;
  }

  // Update handleShippingSubmit
  async function handleShippingSubmit(event) {
    const checkoutData = {
      ...event.detail,
      user_id: getCurrentUserId() || 'guest',
      items: $cart,
    };
    
    console.log('Submitting checkout data:', checkoutData);
    
    try {
      const data = await api.monero.checkout(checkoutData);
      
      console.log('Checkout response:', data);
      
      if (data.success) {
        cart.clear();
        showShippingForm = false;
        window.location.href = `/monero/checkout/${data.order_id}`;
      } else {
        alert('Checkout failed: ' + (data.error || 'Unknown error'));
      }
    } catch (error) {
      console.error('Checkout error:', error);
      alert('Error during checkout: ' + error.message);
    }
  }
  
  // Helper function to get current user ID
  function getCurrentUserId() {
    try {
      return $auth.user ? $auth.user.id : null;
    } catch (e) {
      return null;
    }
  }
</script>

<Router>
  <NavBar 
    {setView} 
    onToggleCart={() => isCartOpen = !isCartOpen}
    onShowShipping={showShippingForm}
  />
  
  <CartDrawer isOpen={isCartOpen} />
  
  <div class="container">
    <Route path="/" exact>
      <Home {setView} />
    </Route>
    <Route path="/login" component={Login} />
    <Route path="/products" component={ProductList} />
    <Route path="/orders" component={Orders} />
    <Route path="/chat" component={Chat} />
    <Route 
      path="/monero/checkout/:order_id" 
      let:params
    >
      <MoneroCheckout {params} />
    </Route>
    
    <Route path="/profile">
      <ProtectedRoute>
        <UserProfile />
      </ProtectedRoute>
    </Route>
    
    <Route path="/admin">
      <ProtectedRoute requiredRole="admin">
        {#if AdminPanel}
          <svelte:component this={AdminPanel} />
        {:else}
          <div class="loading">Loading admin panel...</div>
        {/if}
      </ProtectedRoute>
    </Route>
    
    <Route path="/unauthorized">
      <div class="unauthorized">
        <h1>Unauthorized Access</h1>
        <p>You don't have permission to access this page.</p>
        <a href="/">Return to Home</a>
      </div>
    </Route>
    
    <Route path="/order-status" component={OrderStatus} />
    <Route path="/admin/orders">
      <ProtectedRoute requiredRole="admin">
        <OrderValidator />
      </ProtectedRoute>
    </Route>
    <Route path="/admin/products">
      <ProtectedRoute requiredRole="admin">
        <AdminProducts />
      </ProtectedRoute>
    </Route>
    <Route path="/admin/users">
      <ProtectedRoute requiredRole="admin">
        <AdminUsers />
      </ProtectedRoute>
    </Route>
    <Route path="/checkout/monero">
      <MoneroCheckout />
    </Route>
    <Route path="/config">
      <ConfigTester />
    </Route>
    <Route path="*" let:location>
      {#if location && location.pathname}
        <div style="padding: 20px; border: 2px solid red;">
          <h3>Debug: Route Not Found</h3>
          <p>The current path is: {location.pathname}</p>
        </div>
      {:else}
        <div class="loading">Loading...</div>
      {/if}
    </Route>
  </div>
  
  <footer>
    © 2025 Secure Store. All rights reserved.
  </footer>
  
  <!-- Add the shipping form overlay directly in App.svelte so it's globally available -->
  {#if showShippingForm}
    <ShippingFormOverlay 
      on:close={() => showShippingForm = false}
      on:submit={handleShippingSubmit}
    />
  {/if}
</Router>

<style>
  footer {
    text-align: center;
    padding: 1rem;
    margin-top: 2rem;
    color: #666;
    font-size: 0.9rem;
  }

  :global(body) {
    margin: 0;
    padding: 0;
    font-family: Arial, sans-serif;
    background-color: #f9f9f9;
  }
  
  .container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 1rem;
  }
  
  .loading {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 200px;
  }
  
  .unauthorized {
    text-align: center;
    padding: 2rem;
    background-color: #fff9fa;
    border-radius: 8px;
    margin-top: 2rem;
  }
  
  .unauthorized a {
    display: inline-block;
    margin-top: 1rem;
    color: #2196F3;
    text-decoration: none;
  }
</style>

{#if serverReady}
  <!-- Your app content here -->
{:else}
  <div class="loading-container">
    <p>Connecting to server...</p>
    <!-- Add loading spinner here -->
  </div>
{/if}