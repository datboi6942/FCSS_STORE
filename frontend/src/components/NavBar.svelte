<script lang="ts">
  import authStore from '../stores/auth.js';
  export let setView: (view: string) => void;
</script>

<nav>
  <div class="nav-left">
    <button on:click={() => setView('home')}>Home</button>
    <button on:click={() => setView('products')}>Products</button>
    <button on:click={() => setView('orders')}>Orders</button>
    <button on:click={() => setView('chat')}>Chat</button>
    <button on:click={() => setView('payment')}>Payment</button>
  </div>
  
  <div class="nav-right">
    {#if $authStore.isAuthenticated}
      <span class="username">{$authStore.user.username}</span>
      {#if $authStore.isAdmin}
        <span class="admin-badge">Admin</span>
      {/if}
    {:else}
      <button on:click={() => setView('login')}>Login</button>
    {/if}
  </div>
</nav>

<style>
  nav {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 0;
    margin-bottom: 1rem;
  }
  
  .nav-left {
    display: flex;
    gap: 0.5rem;
  }
  
  button {
    background: none;
    color: #555;
    border: none;
    padding: 0.5rem 1rem;
    cursor: pointer;
    font-size: 1rem;
    border-radius: 4px;
    transition: background-color 0.2s, color 0.2s;
  }
  
  button:hover {
    background-color: #f0f0f0;
    color: #1e90ff;
  }
  
  .username {
    margin-right: 0.5rem;
    font-weight: 500;
  }
  
  .admin-badge {
    background-color: #1e90ff;
    color: white;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
    font-size: 0.8rem;
  }
</style>