<script lang="ts">
  import { writable } from 'svelte/store';
  import NavBar from './components/NavBar.svelte';
  import Home from './components/Home.svelte';
  import Products from './components/Products.svelte';
  import Orders from './components/Orders.svelte';
  import Chat from './components/Chat.svelte';
  import Payment from './components/Payment.svelte';
  import Login from './components/Login.svelte';
  import { Router, Route } from "svelte-routing";
  import { onMount } from 'svelte';
  import { auth } from './stores/auth.js';
  
  // Import missing components
  import Nav from './components/Nav.svelte';
  import ProtectedRoute from './components/ProtectedRoute.svelte';
  import UserProfile from './components/UserProfile.svelte';
  import ProductList from './components/Products.svelte'; // Using Products component as ProductList
  import Cart from './components/Cart.svelte';

  // Dynamically import AdminPanel
  import { fade } from 'svelte/transition';
  let AdminPanel;
  
  let serverReady = false;
  
  // Check if server is ready before initializing app components
  async function checkServerReadiness() {
    const maxRetries = 5;
    const retryDelay = 2000;
    
    for (let i = 0; i < maxRetries; i++) {
      try {
        const response = await fetch('http://localhost:5000/health', {
          // Longer timeout for initial connection
          signal: AbortSignal.timeout(5000)
        });
        
        if (response.ok) {
          console.log("Server is ready");
          serverReady = true;
          return true;
        }
      } catch (err) {
        console.log(`Server not ready (attempt ${i+1}/${maxRetries}): ${err.message}`);
      }
      
      // Wait before retrying
      await new Promise(resolve => setTimeout(resolve, retryDelay));
    }
    
    console.warn("Could not establish server connection after multiple attempts");
    // Continue anyway with offline mode
    serverReady = true;
    return false;
  }
  
  onMount(async () => {
    await checkServerReadiness();
    // Dynamically load the AdminPanel component
    const module = await import('./components/AdminPanel.svelte');
    AdminPanel = module.default;
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
</script>

<Router>
  <Nav />
  
  <div class="container">
    <Route path="/" component={ProductList} />
    <Route path="/login" component={Login} />
    <Route path="/products" component={ProductList} />
    
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
    
    <Route path="/cart" component={Cart} />
  </div>
</Router>

<main>
  <NavBar {setView} />
  
  {#if $currentView === 'home'}
    <Home {setView} />
  {:else if $currentView === 'products'}
    <Products />
  {:else if $currentView === 'orders'}
    <Orders />
  {:else if $currentView === 'chat'}
    <Chat />
  {:else if $currentView === 'payment'}
    <Payment />
  {:else if $currentView === 'login'}
    <Login />
  {/if}
</main>

<footer>
  © 2025 Secure Store. All rights reserved.
</footer>

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
</style>

{#if serverReady}
  <!-- Your app content here -->
{:else}
  <div class="loading-container">
    <p>Connecting to server...</p>
    <!-- Add loading spinner here -->
  </div>
{/if}