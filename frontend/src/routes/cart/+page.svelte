<script>
  import { cart, cartTotal } from '../../stores/cart.js';
  import { goto } from '$app/navigation';
  import { onMount } from 'svelte';
  import MoneroCheckout from '../../components/MoneroCheckout.svelte';
  
  let checkoutStarted = false;
  let orderId = '';
  
  function updateQuantity(id, newQuantity) {
    if (newQuantity > 0) {
      cart.updateQuantity(id, newQuantity);
    } else {
      cart.removeItem(id);
    }
  }
  
  function removeItem(id) {
    cart.removeItem(id);
  }
  
  function continueShopping() {
    goto('/products');
  }
  
  function checkout() {
    if ($cart.length === 0) return;
    
    // Generate an order ID and proceed to Monero checkout
    orderId = 'order-' + Date.now();
    checkoutStarted = true;
    
    // In a real implementation, you would call the backend to create an order first
    // and then use the returned order ID for payment
  }
</script>

<svelte:head>
  <title>Shopping Cart</title>
</svelte:head>

<div class="cart-page">
  <h1>Your Shopping Cart</h1>
  
  {#if !checkoutStarted}
    {#if $cart.items.length > 0}
      <div class="cart-container">
        <div class="cart-items">
          {#each $cart.items as item}
            <div class="cart-item">
              <img src={item.image} alt={item.name} class="item-image">
              <div class="item-details">
                <h3>{item.name}</h3>
                <p class="item-price">${item.price.toFixed(2)}</p>
              </div>
              <div class="item-quantity">
                <button on:click={() => updateQuantity(item.id, item.quantity - 1)}>-</button>
                <span>{item.quantity}</span>
                <button on:click={() => updateQuantity(item.id, item.quantity + 1)}>+</button>
              </div>
              <div class="item-total">${(item.price * item.quantity).toFixed(2)}</div>
              <button class="remove-btn" on:click={() => removeItem(item.id)}>×</button>
            </div>
          {/each}
        </div>
        
        <div class="cart-summary">
          <h2>Order Summary</h2>
          
          <div class="summary-row">
            <span>Subtotal:</span>
            <span>${$cartTotal.toFixed(2)}</span>
          </div>
          
          <div class="summary-row">
            <span>Tax:</span>
            <span>${($cartTotal * 0.07).toFixed(2)}</span>
          </div>
          
          <div class="summary-row total">
            <span>Total:</span>
            <span>${($cartTotal * 1.07).toFixed(2)}</span>
          </div>
          
          <div class="payment-notice">
            <p><strong>Payment Method:</strong> Monero (XMR) cryptocurrency</p>
            <p class="payment-info">Payments are processed securely using Monero, offering complete privacy and security.</p>
          </div>
          
          <button class="btn checkout-btn" on:click={checkout}>Proceed to Checkout</button>
          <button class="btn continue-btn" on:click={continueShopping}>Continue Shopping</button>
        </div>
      </div>
    {:else}
      <div class="empty-cart">
        <p>Your cart is empty</p>
        <button class="btn continue-btn" on:click={continueShopping}>Start Shopping</button>
      </div>
    {/if}
  {:else}
    <div class="checkout-container">
      <MoneroCheckout {orderId} />
    </div>
  {/if}
</div>

<style>
  .cart-page {
    max-width: 1200px;
    margin: 0 auto;
    padding: 1rem;
  }
  
  h1 {
    margin-bottom: 2rem;
  }
  
  .cart-container {
    display: grid;
    grid-template-columns: 2fr 1fr;
    gap: 2rem;
  }
  
  table {
    width: 100%;
    border-collapse: collapse;
  }
  
  th {
    text-align: left;
    padding: 1rem;
    border-bottom: 1px solid #eaeaea;
  }
  
  td {
    padding: 1rem;
    border-bottom: 1px solid #eaeaea;
  }
  
  .product-cell {
    display: flex;
    align-items: center;
  }
  
  .product-image {
    width: 60px;
    height: 60px;
    object-fit: cover;
    margin-right: 1rem;
    border-radius: 4px;
  }
  
  .quantity-controls {
    display: flex;
    align-items: center;
  }
  
  .quantity-controls button {
    width: 30px;
    height: 30px;
    background-color: #f0f0f0;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .quantity-controls span {
    margin: 0 0.5rem;
    min-width: 30px;
    text-align: center;
  }
  
  .remove-btn {
    background: none;
    border: none;
    font-size: 18px;
    color: #999;
    cursor: pointer;
  }
  
  .cart-summary {
    background-color: #f9f9f9;
    padding: 1.5rem;
    border-radius: 8px;
  }
  
  .summary-row {
    display: flex;
    justify-content: space-between;
    margin-bottom: 1rem;
  }
  
  .total {
    font-weight: bold;
    font-size: 1.2rem;
    margin-top: 1rem;
    padding-top: 1rem;
    border-top: 1px solid #eaeaea;
  }
  
  .btn {
    width: 100%;
    padding: 0.75rem;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    margin-bottom: 0.5rem;
  }
  
  .checkout-btn {
    background-color: #ff3e00;
    color: white;
  }
  
  .continue-btn {
    background-color: white;
    border: 1px solid #aaa;
  }
  
  .empty-cart {
    text-align: center;
    padding: 3rem;
  }
  
  @media (max-width: 768px) {
    .cart-container {
      grid-template-columns: 1fr;
    }
  }
  
  .payment-notice {
    margin: 1.5rem 0;
    padding: 1rem;
    background-color: #f0f8ff;
    border-radius: 4px;
    border-left: 4px solid #2196f3;
  }
  
  .payment-info {
    font-size: 0.9rem;
    color: #666;
    margin-top: 0.5rem;
  }
  
  .checkout-container {
    margin-top: 2rem;
  }
</style> 