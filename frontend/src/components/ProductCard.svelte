<script>
  import { onMount } from 'svelte';
  import { navigate } from 'svelte-routing';
  import { auth } from '../stores/auth.js';
  import { createEventDispatcher } from 'svelte';
  
  export let product = {};
  
  let user = null;
  let processingPurchase = false;
  let purchaseError = null;
  let showPaymentModal = false;
  let paymentInfo = null;
  const API_BASE_URL = 'http://localhost:5000';
  let adding = false;
  let error = null;
  let addSuccess = false;
  const dispatch = createEventDispatcher();
  
  // Track if we're in offline/demo mode
  let offlineMode = false;
  
  onMount(() => {
    const unsubscribe = auth.subscribe(authData => {
      user = authData.user;
    });
    
    // Check if product is from fallback data
    if (product.id && product.id.startsWith('fb')) {
      console.log("Fallback product detected, enabling offline mode");
      offlineMode = true;
    }
    
    return unsubscribe;
  });
  
  async function handlePurchase() {
    if (!user) {
      navigate('/login');
      return;
    }
    
    processingPurchase = true;
    purchaseError = null;
    
    try {
      // Handle offline mode with demo checkout
      if (offlineMode) {
        // Simulate server response delay
        await new Promise(resolve => setTimeout(resolve, 800));
        
        // Create dummy payment info
        paymentInfo = {
          order_id: `demo-${Date.now()}`,
          payment_info: {
            amount: product.price,
            currency: "USD",
            crypto_address: "0xDemoAddress1234567890AbCdEf",
            payment_methods: ["BTC", "ETH", "USDC"]
          }
        };
        
        showPaymentModal = true;
        return;
      }
      
      // Regular online mode
      const response = await fetch('http://localhost:5000/products/purchase', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${$auth.token}`
        },
        body: JSON.stringify({
          user_id: user.id,
          product_id: product.id
        })
      });
      
      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.error || 'Failed to process purchase');
      }
      
      paymentInfo = await response.json();
      showPaymentModal = true;
    } catch (error) {
      purchaseError = error.message;
    } finally {
      processingPurchase = false;
    }
  }
  
  async function confirmPayment() {
    try {
      // Create a demo transaction hash
      const txHash = `0x${Math.random().toString(16).substr(2, 40)}`;
      
      // If in offline mode, simulate successful payment
      if (offlineMode) {
        await new Promise(resolve => setTimeout(resolve, 1000));
        showPaymentModal = false;
        alert('Demo payment successful! This is a simulation.');
        return;
      }
      
      // Regular online mode
      const response = await fetch('http://localhost:5000/payment/crypto/confirm', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${$auth.token}`
        },
        body: JSON.stringify({
          order_id: paymentInfo.order_id,
          amount: paymentInfo.payment_info.amount,
          currency: paymentInfo.payment_info.currency,
          transaction_hash: txHash
        })
      });
      
      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.error || 'Failed to confirm payment');
      }
      
      showPaymentModal = false;
      alert('Payment successful! Your order has been placed.');
      navigate('/orders');
    } catch (error) {
      purchaseError = error.message;
    }
  }
  
  async function addToCart() {
    if (!$auth.isAuthenticated) {
      alert('Please log in to add items to your cart');
      return;
    }
    
    try {
      adding = true;
      error = null;
      
      // Handle offline mode with simulated cart add
      if (offlineMode) {
        await new Promise(resolve => setTimeout(resolve, 600));
        
        // Get current demo cart or create a new one
        let demoCart;
        try {
          const storedCart = localStorage.getItem('demoCart');
          demoCart = storedCart ? JSON.parse(storedCart) : { 
            id: "demo-cart", 
            user_id: "offline-user",
            items: [],
            created_at: new Date().toISOString()
          };
        } catch (err) {
          console.error("Error parsing demo cart:", err);
          demoCart = { 
            id: "demo-cart", 
            user_id: "offline-user",
            items: [],
            created_at: new Date().toISOString()
          };
        }
        
        // Check if product already in cart
        const existingItemIndex = demoCart.items.findIndex(item => 
          item.product_id === product.id
        );
        
        if (existingItemIndex >= 0) {
          // Increment quantity
          demoCart.items[existingItemIndex].quantity += 1;
        } else {
          // Add new item
          demoCart.items.push({
            product_id: product.id,
            name: product.name,
            price: product.price,
            quantity: 1
          });
        }
        
        // Save updated cart to localStorage
        localStorage.setItem('demoCart', JSON.stringify(demoCart));
        console.log("Updated demo cart in localStorage:", demoCart);
        
        // Update cart badge UI
        try {
          const cartBadgeElem = document.querySelector('.cart-badge');
          if (cartBadgeElem) {
            const currentCount = parseInt(cartBadgeElem.textContent || '0');
            cartBadgeElem.textContent = (currentCount + 1).toString();
          } else {
            // If no badge exists yet, create one
            const cartLink = document.querySelector('.cart-link');
            if (cartLink) {
              const badge = document.createElement('span');
              badge.className = 'cart-badge';
              badge.textContent = '1';
              cartLink.appendChild(badge);
            }
          }
        } catch (err) {
          console.log("Could not update cart badge:", err);
        }
        
        addSuccess = true;
        
        // Also set a timeout to hide the success message
        setTimeout(() => {
          addSuccess = false;
        }, 3000);
        
        // Fire cart updated event
        dispatch('cartUpdated');
        
        // Also dispatch a window event for demo cart
        window.dispatchEvent(new CustomEvent('demo-add-to-cart', {
          detail: product
        }));
        
        return;
      }
      
      // Regular online mode
      const response = await fetch(`${API_BASE_URL}/cart/add`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': $auth.token ? `Bearer ${$auth.token}` : ''
        },
        body: JSON.stringify({
          user_id: $auth.user.id,
          product_id: product.id,
          quantity: 1
        })
      });
      
      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.error || 'Failed to add to cart');
      }
      
      addSuccess = true;
      
      // Update cart count directly by dispatching a simple event
      dispatch('cartUpdated');
      
      // Also set a timeout to hide the success message
      setTimeout(() => {
        addSuccess = false;
      }, 3000);
      
      // For extra reliability, try to directly update the cart badge
      try {
        const cartBadgeElem = document.querySelector('.cart-badge');
        if (cartBadgeElem) {
          const currentCount = parseInt(cartBadgeElem.textContent || '0');
          cartBadgeElem.textContent = (currentCount + 1).toString();
        }
      } catch (err) {
        console.log("Could not directly update cart badge");
      }
      
    } catch (err) {
      console.error('Error adding to cart:', err);
      error = err.message;
    } finally {
      adding = false;
    }
  }
</script>

<!-- Add an "offline mode" indicator for demo products -->
<div class="product-card {offlineMode ? 'demo-product' : ''}">
  {#if offlineMode}
    <div class="demo-badge">DEMO</div>
  {/if}
  
  <h3>{product.name}</h3>
  <p class="description">{product.description}</p>
  <p class="price">${product.price?.toFixed(2)}</p>
  
  <div class="actions">
    <button 
      class="buy-button" 
      on:click={handlePurchase} 
      disabled={processingPurchase || !product.available}
    >
      {#if processingPurchase}
        Processing...
      {:else if !product.available}
        Out of Stock
      {:else}
        Buy Now
      {/if}
    </button>
    
    {#if product.available}
      <button 
        class="add-to-cart-btn" 
        on:click={addToCart} 
        disabled={adding}
      >
        {#if adding}
          Adding...
        {:else}
          Add to Cart
        {/if}
      </button>
    {/if}
  </div>
  
  {#if addSuccess}
    <div class="success-message">
      Added to cart!
    </div>
  {/if}
  
  {#if error}
    <div class="error-message">
      {error}
    </div>
  {/if}
  
  {#if purchaseError}
    <p class="error">{purchaseError}</p>
  {/if}
</div>

{#if showPaymentModal}
  <div class="modal">
    <div class="modal-content">
      <h2>Complete Your Purchase</h2>
      <p>Please send {paymentInfo.payment_info.amount} {paymentInfo.payment_info.currency} to the following address:</p>
      <div class="crypto-address">
        {paymentInfo.payment_info.crypto_address}
      </div>
      <p>Accepted cryptocurrencies: {paymentInfo.payment_info.payment_methods.join(', ')}</p>
      
      <div class="modal-buttons">
        <button class="cancel-button" on:click={() => showPaymentModal = false}>
          Cancel
        </button>
        <button class="confirm-button" on:click={confirmPayment}>
          I've Sent the Payment
        </button>
      </div>
    </div>
  </div>
{/if}

<style>
  /* Add these new styles */
  .demo-product {
    border: 1px solid #b3d7ff;
    box-shadow: 0 2px 8px rgba(0, 123, 255, 0.15);
    position: relative;
  }
  
  .demo-badge {
    position: absolute;
    top: 10px;
    right: 10px;
    background: #007bff;
    color: white;
    padding: 3px 8px;
    border-radius: 4px;
    font-size: 12px;
    font-weight: bold;
  }
  
  .product-card {
    border: 1px solid #ddd;
    border-radius: 8px;
    padding: 16px;
    margin-bottom: 20px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    position: relative;
  }
  
  .description {
    color: #666;
  }
  
  .price {
    font-size: 1.2rem;
    font-weight: bold;
    color: #333;
  }
  
  .buy-button {
    background-color: #4CAF50;
    color: white;
    border: none;
    padding: 8px 16px;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .buy-button:disabled {
    background-color: #ccc;
    cursor: not-allowed;
  }
  
  .error {
    color: red;
    margin-top: 10px;
  }
  
  .modal {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-color: rgba(0,0,0,0.5);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
  }
  
  .modal-content {
    background-color: white;
    padding: 24px;
    border-radius: 8px;
    max-width: 500px;
    width: 100%;
  }
  
  .crypto-address {
    background-color: #f5f5f5;
    padding: 12px;
    border-radius: 4px;
    font-family: monospace;
    margin: 12px 0;
    word-break: break-all;
  }
  
  .modal-buttons {
    display: flex;
    justify-content: space-between;
    margin-top: 24px;
  }
  
  .cancel-button {
    background-color: #f44336;
    color: white;
    border: none;
    padding: 8px 16px;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .confirm-button {
    background-color: #4CAF50;
    color: white;
    border: none;
    padding: 8px 16px;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .actions {
    display: flex;
    justify-content: space-between;
  }
  
  .add-to-cart-btn {
    background-color: #3498db;
    color: white;
    border: none;
    padding: 8px 16px;
    border-radius: 4px;
    cursor: pointer;
    font-weight: bold;
    transition: background-color 0.3s;
  }
  
  .add-to-cart-btn:hover {
    background-color: #2980b9;
  }
  
  .add-to-cart-btn:disabled {
    background-color: #95a5a6;
    cursor: not-allowed;
  }
  
  .out-of-stock {
    color: #e74c3c;
    font-style: italic;
  }
  
  .success-message {
    background-color: #2ecc71;
    color: white;
    padding: 8px;
    border-radius: 4px;
    margin-top: 10px;
    text-align: center;
  }
  
  .error-message {
    background-color: #e74c3c;
    color: white;
    padding: 8px;
    border-radius: 4px;
    margin-top: 10px;
    text-align: center;
  }
</style>
