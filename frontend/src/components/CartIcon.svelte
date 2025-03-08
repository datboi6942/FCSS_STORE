<script>
  import { createEventDispatcher } from 'svelte';
  import { slide } from 'svelte/transition';
  import { cartItems, cartTotal, cart } from '../stores/cart.js';
  
  const dispatch = createEventDispatcher();
  
  let isDrawerOpen = false;
  
  export let count = 0;
  
  $: itemCount = count || $cartItems.reduce((sum, item) => sum + item.quantity, 0);
  
  function toggleDrawer() {
    isDrawerOpen = !isDrawerOpen;
  }
  
  function removeFromCart(itemId) {
    cartItems.update(items => items.filter(item => item.id !== itemId));
  }
  
  function updateQuantity(itemId, newQuantity) {
    if (newQuantity < 1) return;
    cartItems.update(items =>
      items.map(item =>
        item.id === itemId ? { ...item, quantity: newQuantity } : item
      )
    );
  }
  
  function proceedToCheckout() {
    if ($cartItems.length === 0) {
      alert('Your cart is empty!');
      return;
    }
    isDrawerOpen = false;
    dispatch('showShipping');
  }
</script>

<div class="cart-icon">
  <svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
    <circle cx="9" cy="21" r="1"></circle>
    <circle cx="20" cy="21" r="1"></circle>
    <path d="M1 1h4l2.68 13.39a2 2 0 0 0 2 1.61h9.72a2 2 0 0 0 2-1.61L23 6H6"></path>
  </svg>
  {#if itemCount > 0}
    <span class="cart-count">{itemCount}</span>
  {/if}
</div>

<style>
  .cart-icon {
    position: relative;
  }
  
  .cart-button {
    background: none;
    border: none;
    color: white;
    cursor: pointer;
    font-size: 1rem;
    padding: 8px;
  }
  
  .cart-drawer {
    position: fixed;
    top: 0;
    right: 0;
    width: 400px;
    height: 100vh;
    background: white;
    box-shadow: -2px 0 5px rgba(0,0,0,0.2);
    padding: 20px;
    z-index: 1000;
    display: flex;
    flex-direction: column;
  }
  
  .cart-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
  }
  
  .cart-items {
    flex: 1;
    overflow-y: auto;
    padding-bottom: 120px;
  }
  
  .cart-item {
    display: flex;
    padding: 15px;
    border-bottom: 1px solid #eee;
    align-items: flex-start;
    gap: 15px;
  }
  
  .cart-item img {
    width: 60px;
    height: 60px;
    object-fit: cover;
    border-radius: 4px;
  }
  
  .item-details {
    flex: 1;
    min-width: 0;
    padding-right: 10px;
  }
  
  .item-details h4 {
    margin: 0 0 5px 0;
    font-size: 1rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  
  .item-price {
    font-weight: bold;
    color: #2ecc71;
  }
  
  .quantity-controls {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 5px;
  }
  
  .quantity-controls button {
    background: #f5f5f5;
    border: none;
    border-radius: 4px;
    padding: 4px 8px;
    cursor: pointer;
  }
  
  .quantity-controls button:hover {
    background: #e0e0e0;
  }
  
  .cart-footer {
    position: fixed;
    bottom: 0;
    right: 0;
    width: 400px;
    padding: 20px;
    background: white;
    border-top: 1px solid #eee;
    box-shadow: 0 -2px 5px rgba(0,0,0,0.1);
    z-index: 1001;
  }
  
  .cart-total {
    display: flex;
    justify-content: space-between;
    margin-bottom: 10px;
    font-weight: bold;
  }
  
  .checkout-button {
    width: 100%;
    padding: 12px;
    background: #2ecc71;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .checkout-button:hover {
    background: #27ae60;
  }
  
  .empty-cart {
    text-align: center;
    color: #666;
    padding: 20px;
  }
  
  .item-title {
    font-size: 1.1rem;
    font-weight: bold;
    margin: 0 0 4px 0;
    color: #333;
    white-space: normal;
    overflow-wrap: break-word;
    word-wrap: break-word;
    line-height: 1.2;
  }
  
  .item-description {
    font-size: 0.9rem;
    color: #666;
    margin: 0 0 8px 0;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }
  
  .remove-button {
    background: none;
    border: none;
    color: #999;
    cursor: pointer;
    font-size: 1.2rem;
    padding: 4px 8px;
    align-self: flex-start;
  }
  
  .remove-button:hover {
    color: #ff4444;
  }
</style> 