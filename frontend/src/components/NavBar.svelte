<script lang="ts">
  import authStore from '../stores/auth.js';
  import { Link } from "svelte-routing";
  import CartIcon from "./CartIcon.svelte";
  export let setView: (view: string) => void;
  import { tick } from 'svelte';

  // Flag to prevent rapid-fire clicks on the Products link
  let productsDisabled = false;

  function handleProductsClick(e: MouseEvent) {
    if (productsDisabled) {
      // Prevent navigation if already clicked recently
      e.preventDefault();
      return;
    }
    productsDisabled = true;
    // Re-enable navigation after 1 second
    setTimeout(() => {
      productsDisabled = false;
    }, 1000);
  }
</script>

<nav class="navbar">
  <div class="brand">
    <Link to="/">FCSS STORE</Link>
  </div>
  
  <div class="nav-links">
    <Link to="/">Home</Link>
    <Link to="/products" on:click={handleProductsClick}>Products</Link>
    <Link to="/cart">Cart</Link>
    <button on:click={() => setView('orders')}>Orders</button>
    <button on:click={() => setView('chat')}>Chat</button>
    <button on:click={() => setView('payment')}>Payment</button>
    <button on:click={() => setView('login')}>Login</button>
  </div>
  
  <div class="nav-actions">
    <CartIcon />
  </div>
</nav>

<style>
  .navbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    background-color: #2196F3;
    color: white;
  }
  
  .brand button {
    font-size: 1.5rem;
    font-weight: bold;
    background: none;
    border: none;
    color: white;
    cursor: pointer;
  }
  
  .nav-links {
    display: flex;
    gap: 1rem;
  }
  
  .nav-links button {
    background: none;
    border: none;
    color: white;
    cursor: pointer;
    padding: 0.5rem;
    border-radius: 4px;
    transition: background-color 0.2s;
  }
  
  .nav-links button:hover {
    background-color: rgba(255, 255, 255, 0.1);
  }
  
  .nav-actions {
    display: flex;
    align-items: center;
  }
  
  a {
    color: inherit;
    text-decoration: none;
  }
</style>