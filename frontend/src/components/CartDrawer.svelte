<script>
  import { cart, cartTotal } from '../stores/cart.js';
  import { fly } from 'svelte/transition';
  
  function closeCart() {
    cart.toggleCart();
  }
  
  function removeItem(id) {
    cart.removeItem(id);
  }
  
  function updateQuantity(id, newQuantity) {
    if (newQuantity > 0) {
      cart.updateQuantity(id, newQuantity);
    } else {
      cart.removeItem(id);
    }
  }
  
  function checkout() {
    // Send cart data to your Rust backend
    fetch('/api/checkout', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json'
      },
      body: JSON.stringify($cart.items)
    })
    .then(response => response.json())
    .then(data => {
      if (data.success) {
        cart.clearCart();
        closeCart();
        // Redirect to confirmation page or show success message
      }
    })
    .catch(error => {
      console.error('Checkout error:', error);
      // Show error message
    });
  }
</script>

{#if $cart.isOpen}
  <div class="cart-overlay" on:click={closeCart}>
    <div 
      class="cart-drawer" 
      on:click|stopPropagation 
      transition:fly={{ x: 300, duration: 300 }}
    >
      <div class="cart-header">
        <h2>Your Cart</h2>
        <button class="close-btn" on:click={closeCart}>×</button>
      </div>
      
      {#if $cart.items.length === 0}
        <div class="empty-cart">
          <p>Your cart is empty</p>
          <button class="continue-btn" on:click={closeCart}>
            Continue Shopping
          </button>
        </div>
      {:else}
        <div class="cart-items">
          {#each $cart.items as item (item.id)}
            <div class="cart-item">
              <img src={item.image} alt={item.name} class="item-image" />
              <div class="item-details">
                <h3>{item.name}</h3>
                <p class="item-price">${item.price.toFixed(2)}</p>
              </div>
              <div class="quantity-controls">
                <button on:click={() => updateQuantity(item.id, item.quantity - 1)}>-</button>
                <span>{item.quantity}</span>
                <button on:click={() => updateQuantity(item.id, item.quantity + 1)}>+</button>
              </div>
              <p class="item-total">${(item.price * item.quantity).toFixed(2)}</p>
              <button class="remove-btn" on:click={() => removeItem(item.id)}>×</button>
            </div>
          {/each}
        </div>
        
        <div class="cart-footer">
          <div class="cart-total">
            <span>Total:</span>
            <span>${$cartTotal.toFixed(2)}</span>
          </div>
          <button class="checkout-btn" on:click={checkout}>
            Proceed to Checkout
          </button>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .cart-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-color: rgba(0, 0, 0, 0.5);
    z-index: 1000;
    display: flex;
    justify-content: flex-end;
  }
  
  .cart-drawer {
    background-color: white;
    width: 400px;
    max-width: 90%;
    height: 100%;
    display: flex;
    flex-direction: column;
    box-shadow: -2px 0 10px rgba(0, 0, 0, 0.1);
  }
  
  .cart-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1rem;
    border-bottom: 1px solid #eaeaea;
  }
  
  .close-btn {
    background: none;
    border: none;
    font-size: 24px;
    cursor: pointer;
  }
  
  .cart-items {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
  }
  
  .cart-item {
    display: flex;
    align-items: center;
    margin-bottom: 1rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid #eaeaea;
  }
  
  .item-image {
    width: 50px;
    height: 50px;
    object-fit: cover;
    border-radius: 4px;
    margin-right: 1rem;
  }
  
  .item-details {
    flex: 1;
  }
  
  .item-price {
    color: #666;
  }
  
  .quantity-controls {
    display: flex;
    align-items: center;
    margin: 0 1rem;
  }
  
  .quantity-controls button {
    width: 24px;
    height: 24px;
    background-color: #f0f0f0;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .quantity-controls span {
    margin: 0 0.5rem;
  }
  
  .remove-btn {
    background: none;
    border: none;
    font-size: 18px;
    color: #999;
    cursor: pointer;
  }
  
  .item-total {
    margin-left: auto;
    font-weight: bold;
  }
  
  .cart-footer {
    padding: 1rem;
    border-top: 1px solid #eaeaea;
  }
  
  .cart-total {
    display: flex;
    justify-content: space-between;
    font-weight: bold;
    margin-bottom: 1rem;
    font-size: 1.2rem;
  }
  
  .checkout-btn {
    width: 100%;
    padding: 0.75rem;
    background-color: #ff3e00;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .empty-cart {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    padding: 2rem;
    text-align: center;
  }
  
  .continue-btn {
    margin-top: 1rem;
    padding: 0.5rem 1rem;
    background-color: #ff3e00;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }
</style> 