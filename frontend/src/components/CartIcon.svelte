<script>
  import { cart, cartCount } from '../stores/cart.js';
  import { createEventDispatcher } from 'svelte';
  
  // Create a Svelte event dispatcher - this is the proper way to dispatch events in Svelte
  const dispatch = createEventDispatcher();
  
  // Add a local isOpen variable to control cart visibility
  let isCartOpen = false;
  
  // Handle undefined cart count
  $: displayCount = $cartCount || 0;
  
  // New function to toggle the cart drawer
  function toggleCart() {
    // Use the Svelte dispatch method to send the event to the parent
    dispatch('toggleCart');
  }
</script>

<div class="cart-icon" on:click={toggleCart}>
  <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <circle cx="9" cy="21" r="1"></circle>
    <circle cx="20" cy="21" r="1"></circle>
    <path d="M1 1h4l2.68 13.39a2 2 0 0 0 2 1.61h9.72a2 2 0 0 0 2-1.61L23 6H6"></path>
  </svg>
  
  {#if displayCount > 0}
    <span class="cart-count">{displayCount}</span>
  {/if}
</div>

<style>
  .cart-icon {
    position: relative;
    cursor: pointer;
    margin-left: 1rem;
  }
  
  .cart-count {
    position: absolute;
    top: -8px;
    right: -8px;
    background-color: #ff3e00;
    color: white;
    border-radius: 50%;
    width: 18px;
    height: 18px;
    font-size: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
  }
</style> 