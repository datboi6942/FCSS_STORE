<script lang="ts">
  import { writable } from 'svelte/store';
  import NavBar from './components/NavBar.svelte';
  import Home from './components/Home.svelte';
  import Products from './components/Products.svelte';
  import Orders from './components/Orders.svelte';
  import Chat from './components/Chat.svelte';
  import Payment from './components/Payment.svelte';
  import Login from './components/Login.svelte';

  // A simple store to hold the current view
  const currentView = writable('home');
  
  function setView(view: string): void {
    currentView.set(view);
  }
</script>

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
</style>