<script>
  import { cart, cartTotal } from '../../stores/cart.js';
  import { goto } from '$app/navigation';
  
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
    if ($cart.items.length === 0) return;
    
    // Call your Rust backend
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
        goto('/checkout/success');
      }
    })
    .catch(error => {
      console.error('Checkout error:', error);
    });
  }
</script>

<svelte:head>
  <title>Shopping Cart</title>
</svelte:head>

<div class="cart-page">
  <h1>Your Shopping Cart</h1>
  
  {#if $cart.items.length === 0}
    <div class="empty-cart">
      <p>Your cart is empty</p>
      <button class="btn continue-btn" on:click={continueShopping}>
        Start Shopping
      </button>
    </div>
  {:else}
    <div class="cart-container">
      <div class="cart-items">
        <table>
          <thead>
            <tr>
              <th>Product</th>
              <th>Price</th>
              <th>Quantity</th>
              <th>Total</th>
              <th></th>
            </tr>
          </thead>
          <tbody>
            {#each $cart.items as item (item.id)}
              <tr class="cart-item">
                <td class="product-cell">
                  <img src={item.image} alt={item.name} class="product-image">
                  <span class="product-name">{item.name}</span>
                </td>
                <td>${item.price.toFixed(2)}</td>
                <td>
                  <div class="quantity-controls">
                    <button on:click={() => updateQuantity(item.id, item.quantity - 1)}>-</button>
                    <span>{item.quantity}</span>
                    <button on:click={() => updateQuantity(item.id, item.quantity + 1)}>+</button>
                  </div>
                </td>
                <td>${(item.price * item.quantity).toFixed(2)}</td>
                <td>
                  <button class="remove-btn" on:click={() => removeItem(item.id)}>×</button>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      </div>
      
      <div class="cart-summary">
        <h2>Order Summary</h2>
        <div class="summary-row">
          <span>Subtotal:</span>
          <span>${$cartTotal.toFixed(2)}</span>
        </div>
        <div class="summary-row">
          <span>Shipping:</span>
          <span>$0.00</span>
        </div>
        <div class="summary-row total">
          <span>Total:</span>
          <span>${$cartTotal.toFixed(2)}</span>
        </div>
        <button class="btn checkout-btn" on:click={checkout}>
          Proceed to Checkout
        </button>
        <button class="btn continue-btn" on:click={continueShopping}>
          Continue Shopping
        </button>
      </div>
    </div>
  {/if}
</div>

<style>
  .cart-page {
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem;
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
</style> 