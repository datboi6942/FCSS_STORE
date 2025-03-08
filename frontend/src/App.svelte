<script lang="ts">
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
  import { cartItems, cartTotal, cart } from './stores/cart.js';
  
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
  
  // Check if server is ready before initializing app components
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
        const response = await fetch('http://localhost:5000/health', {
          // Use a 5000ms timeout
          signal: AbortSignal.timeout(5000)
        });
        if (response.ok) {
          console.log(`Server is ready (attempt ${i + 1})`);
          isReady = true;
          break;
        }
      } catch (err) {
        console.log(`Health check attempt ${i + 1} failed: ${err}`);
      }
      // Increase delay with each retry (exponential backoff)
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
    await checkServerReadiness();
    // Add event listener for Buy Now button
    document.addEventListener('showShipping', () => {
      showShippingForm = true;
    });
    
    // Cleanup
    return () => {
      document.removeEventListener('showShipping', () => {
        showShippingForm = true;
      });
    };
  });
  
  // For compatibility with the old navigation system
  const currentView = writable('home');
  
  function setView(view) {
    currentView.set(view);
  }

  // Check if user is authenticated on page load
  onMount(() => {
    const token = localStorage.getItem('jwt');
    if (token) {
      // Verify token with backend
      fetch('http://localhost:8443/auth/profile', {
        headers: {
          'Authorization': `Bearer ${token}`
        }
      }).then(response => {
        if (response.ok) {
          auth.login(token);
        } else {
          // Token invalid
          auth.logout();
        }
      }).catch(error => {
        console.error('Auth check failed', error);
        auth.logout();
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

  // Add this function to handle shipping form submission
  async function handleShippingSubmit(event) {
    const checkoutData = {
      ...event.detail,
      user_id: getCurrentUserId() || 'guest',
      items: $cartItems,
    };
    
    console.log('Submitting checkout data:', checkoutData);
    
    try {
      // Update this URL to match our backend route
      const response = await fetch('http://localhost:5000/monero/checkout', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Accept': 'application/json'
        },
        body: JSON.stringify(checkoutData)
      });
      
      console.log('Response status:', response.status);
      
      if (!response.ok) {
        const errorText = await response.text();
        console.error('Error response body:', errorText);
        throw new Error(`HTTP error! status: ${response.status}, response: ${errorText}`);
      }
      
      const data = await response.json();
      console.log('Checkout response:', data);
      
      if (data.success) {
        cartItems.set([]);
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
    onToggleCart={handleToggleCart}
    onShowShipping={handleShowShipping}
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
    <Route path="*" let:location>
      <div style="padding: 20px; border: 2px solid red;">
        <h3>Debug: Route Not Found</h3>
        <p>The current path is: {location.pathname}</p>
      </div>
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
  main {
    max-width: 1200px;
    margin: 0 auto;
    padding: 1rem;
  }
  
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

  /* Add styling for the order ID section */
  .order-id-section {
    margin-top: 20px;
    padding: 15px;
    background-color: #f8f9fa;
    border-radius: 5px;
    border-left: 4px solid #28a745;
  }
  
  .order-id-note {
    font-size: 0.85em;
    color: #6c757d;
    margin-top: 5px;
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