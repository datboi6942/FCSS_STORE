<script lang="ts">
  import { Link } from "svelte-routing";
  import CartIcon from "./CartIcon.svelte";
  import { auth } from '../stores/auth.js';
  import { cartItems } from '../stores/cart.js';

  export let setView: (view: string) => void;
  export let onToggleCart = () => {};
  export let onShowShipping = () => {};

  // Flag to prevent rapid-fire clicks on the Products link
  let productsDisabled = false;

  function handleProductsClick(e: MouseEvent) {
    if (productsDisabled) {
      e.preventDefault();
      return;
    }
    productsDisabled = true;
    setTimeout(() => {
      productsDisabled = false;
    }, 1000);
  }

  function handleShowShipping() {
    onShowShipping();
  }

  // Calculate total items in cart
  $: cartItemCount = $cartItems.reduce((sum, item) => sum + item.quantity, 0);

  // Debug log to check auth state
  $: console.log('Auth state:', $auth);
  $: console.log('Is authenticated:', $auth?.isAuthenticated);
  $: console.log('Is admin:', $auth?.isAdmin);
</script>

<nav class="navbar">
  <div class="brand">
    <Link to="/">FCSS STORE</Link>
  </div>
  
  <div class="nav-links">
    <Link to="/">Home</Link>
    <Link to="/products" on:click={handleProductsClick}>Products</Link>
    
    {#if $auth && $auth.isAuthenticated}
      <Link to="/orders">Orders</Link>
      <Link to="/profile">Profile</Link>
      
      {#if $auth && $auth.isAdmin}
        <div class="admin-menu">
          <span class="admin-label">Admin ▾</span>
          <div class="admin-dropdown">
            <Link to="/admin/orders">Order Validator</Link>
            <Link to="/admin/products">Manage Products</Link>
            <Link to="/admin/users">Manage Users</Link>
          </div>
        </div>
      {/if}
    {/if}
  </div>
  
  <div class="nav-actions">
    {#if $auth && $auth.isAuthenticated}
      <button class="logout-btn" on:click={() => auth.logout()}>Logout</button>
    {:else}
      <Link to="/login">Login</Link>
    {/if}
    
    <button class="cart-btn" on:click={onToggleCart}>
      <CartIcon count={cartItemCount} />
    </button>
  </div>
</nav>

<style>
  .navbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    background-color: #2c3e50;
    color: white;
  }

  .brand :global(a) {
    font-size: 1.5rem;
    font-weight: bold;
    text-decoration: none;
    color: white;
  }

  .nav-links {
    display: flex;
    align-items: center;
    gap: 1.5rem;
  }

  .nav-actions {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  :global(a) {
    color: white;
    text-decoration: none;
    padding: 0.5rem;
    border-radius: 4px;
    transition: background-color 0.2s;
  }

  :global(a:hover) {
    background-color: #34495e;
  }

  .cart-btn {
    background: none;
    border: none;
    color: white;
    cursor: pointer;
    padding: 0.5rem;
    display: flex;
    align-items: center;
  }

  .logout-btn {
    background-color: #e74c3c;
    color: white;
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    cursor: pointer;
  }

  .admin-menu {
    position: relative;
    display: inline-block;
  }

  .admin-label {
    cursor: pointer;
    padding: 0.5rem;
    border-radius: 4px;
    background-color: #e74c3c;
  }

  .admin-menu:hover .admin-label {
    background-color: #c0392b;
  }

  .admin-dropdown {
    display: none;
    position: absolute;
    top: 100%;
    left: 0;
    background-color: #2c3e50;
    min-width: 160px;
    box-shadow: 0 8px 16px rgba(0,0,0,0.2);
    z-index: 1000;
    border-radius: 4px;
  }

  .admin-menu:hover .admin-dropdown {
    display: flex;
    flex-direction: column;
  }

  .admin-dropdown :global(a) {
    padding: 12px 16px;
    display: block;
    width: 100%;
    box-sizing: border-box;
  }

  .admin-dropdown :global(a:hover) {
    background-color: #34495e;
  }

  @media (max-width: 768px) {
    .nav-links {
      gap: 1rem;
    }

    :global(a), .admin-label {
      padding: 0.3rem;
      font-size: 0.9em;
    }
  }
</style>