<script lang="ts">
  import authStore from '../stores/auth.js';
  export let setView: (view: string) => void;

  function navigate(view) {
    setView(view);
  }
</script>

<nav class="navbar">
  <div class="brand">
    <button on:click={() => navigate('home')}>Secure Store</button>
  </div>
  
  <div class="nav-links">
    <button on:click={() => navigate('home')}>Home</button>
    <button on:click={() => navigate('products')}>Products</button>
    <button on:click={() => navigate('orders')}>Orders</button>
    <button on:click={() => navigate('chat')}>Chat</button>
    <button on:click={() => navigate('payment')}>Payment</button>
    <button on:click={() => navigate('login')}>Login</button>
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
</style>