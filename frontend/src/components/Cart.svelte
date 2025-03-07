<script>
  import { onMount } from 'svelte';
  import { auth } from '../stores/auth.js';
  import { navigate } from 'svelte-routing';
  import { writable } from 'svelte/store';

  // Update API base URL
  const API_BASE_URL = 'http://localhost:5000';

  let cart = null;
  let loading = true;
  let error = null;
  let loadingTimeout = null;
  
  // Track if we're in offline/demo mode
  let offlineMode = false;
  
  // Demo cart for offline mode
  let demoCart = {
    id: "demo-cart",
    user_id: "offline-user",
    items: [],
    created_at: new Date().toISOString()
  };

  // Create a store for the checkout stage to ensure reactivity
  const checkoutStageStore = writable('cart');

  let checkoutStage = 'cart'; // 'cart', 'shipping', or 'payment'
  let shippingInfo = {
    name: "",
    address: "",
    city: "",
    state: "",
    zip: "",
    country: "",
    email: ""
  };

  onMount(async () => {
    console.log("Cart component mounted");
    
    // Immediately check if demo mode is needed for admin users
    if ($auth.isAdmin) {
      console.log("Admin user detected, enabling offline mode");
      offlineMode = true;
    }
    
    // Load demo cart from localStorage if it exists
    const storedDemoCart = localStorage.getItem('demoCart');
    if (storedDemoCart) {
      try {
        demoCart = JSON.parse(storedDemoCart);
        console.log("Loaded demo cart from storage:", demoCart);
        
        // If we're in offline mode, immediately set the cart
        if (offlineMode) {
          cart = demoCart;
          loading = false;
        }
      } catch (err) {
        console.error("Error parsing demo cart:", err);
      }
    }
    
    // Add timeout to prevent infinite loading
    loadingTimeout = setTimeout(() => {
      if (loading) {
        loading = false;
        error = "Loading cart timed out. Using offline mode.";
        console.error("Cart loading timed out");
        offlineMode = true;
        cart = demoCart;
      }
    }, 5000);
    
    await loadCart();
    clearTimeout(loadingTimeout);
  });

  async function loadCart() {
    try {
      loading = true;
      error = null;

      // Get user ID from auth store
      if (!$auth.isAuthenticated || !$auth.user) {
        console.log("User not authenticated, can't load cart");
        loading = false;
        return;
      }
      
      const userId = $auth.user.id || 'admin-user';
      console.log("Loading cart for user:", userId);
      
      // For admin users, create a dummy cart if one doesn't exist
      if ($auth.isAdmin) {
        console.log("Admin user detected, creating admin cart");
        cart = demoCart;
        offlineMode = true;
        loading = false;
        return;
      }
      
      // Try to fetch from server with a timeout
      const controller = new AbortController();
      const timeoutId = setTimeout(() => controller.abort(), 3000);
      
      try {
        const response = await fetch(`${API_BASE_URL}/cart/${userId}`, {
          signal: controller.signal,
          headers: {
            'Authorization': $auth.token ? `Bearer ${$auth.token}` : '',
          }
        });
        
        clearTimeout(timeoutId);
        console.log("Cart response status:", response.status);
        
        if (!response.ok) {
          // If cart doesn't exist for user, create an empty one
          if (response.status === 404) {
            console.log("Cart not found, creating empty cart");
            cart = {
              id: "new-cart-" + Date.now(),
              user_id: userId,
              items: [],
              created_at: new Date().toISOString()
            };
            loading = false;
            return;
          }
          throw new Error(`Failed to load cart: ${response.status}`);
        }
        
        const data = await response.json();
        console.log("Cart data loaded:", data);
        cart = data;
        
      } catch (err) {
        if (err.name === 'AbortError') {
          console.log("Cart request timed out, using offline mode");
          offlineMode = true;
          cart = demoCart;
        } else {
          throw err;
        }
      }
      
    } catch (err) {
      console.error('Error loading cart:', err);
      error = "Could not connect to server. Using offline mode.";
      offlineMode = true;
      cart = demoCart;
    } finally {
      loading = false;
    }
  }

  function calculateTotal() {
    if (!cart || !cart.items || !cart.items.length) return 0;
    return cart.items.reduce((sum, item) => sum + (item.price * item.quantity), 0);
  }
  
  async function removeItem(cartId, index) {
    if (offlineMode) {
      // In offline mode, just remove from local cart
      demoCart.items.splice(index, 1);
      cart = {...demoCart};
      localStorage.setItem('demoCart', JSON.stringify(demoCart));
      return;
    }
    
    try {
      const item = cart.items[index];
      const response = await fetch(`${API_BASE_URL}/cart/remove`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': $auth.token ? `Bearer ${$auth.token}` : ''
        },
        body: JSON.stringify({
          cart_id: cartId,
          product_id: item.product_id
        })
      });
      
      if (!response.ok) {
        throw new Error('Failed to remove item');
      }
      
      // Update local cart immediately for better UX
      cart.items.splice(index, 1);
      cart = {...cart};
      
    } catch (err) {
      console.error('Error removing item:', err);
      error = err.message;
    }
  }
  
  function startCheckout() {
    if (!cart || !cart.items || cart.items.length === 0) {
      alert('Your cart is empty!');
      return;
    }
    console.log("Starting checkout, changing stage to shipping");
    checkoutStageStore.set('shipping');
    // For debugging, directly set the local variable too
    checkoutStage = 'shipping';
  }
  
  function submitShippingInfo() {
    console.log("Submitting shipping info", shippingInfo);
    
    // Validate form
    if (!shippingInfo.name || !shippingInfo.address || !shippingInfo.city || 
        !shippingInfo.state || !shippingInfo.zip || !shippingInfo.country || 
        !shippingInfo.email) {
      alert('Please fill in all shipping information fields.');
      return;
    }
    
    // Email validation
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
    if (!emailRegex.test(shippingInfo.email)) {
      alert('Please enter a valid email address.');
      return;
    }
    
    // Store shipping info in localStorage so it persists
    localStorage.setItem('shippingInfo', JSON.stringify(shippingInfo));
    
    // Proceed with checkout
    processPayment();
  }
  
  async function processPayment() {
    try {
      // Get user ID
      const userId = getCurrentUserId();
      console.log("Checkout with user ID:", userId);
      
      // Calculate total amount
      const totalAmount = calculateTotal();
      
      // Prepare checkout data
      const checkoutData = {
        items: cart.items,
        shipping_info: shippingInfo, // Use the filled in shipping info
        user_id: userId,
        total: totalAmount
      };
      
      console.log("Sending checkout data:", checkoutData);
      
      if (offlineMode) {
        // In offline/demo mode, create a mock response
        console.log("Offline mode, creating mock order");
        
        const mockOrderId = "demo-order-" + Math.random().toString(36).substring(2, 10);
        const mockResponse = {
          success: true,
          order_id: mockOrderId,
          payment: {
            payment_id: "pay-" + Math.random().toString(36).substring(2, 10),
            address: "44AFFq5kSiGBoZ4NMDwYtN18obc8AemS33DBLWs3H7otXft3XjrpDtQGv7SqSsaBYBb98uNbr2VBBEt7f2wfn3RVGQBEP3A",
            amount_xmr: "0.25",
            status: "Pending"
          },
          message: "Please send Monero to the provided address"
        };
        
        // Store the mock response
        localStorage.setItem('checkoutData', JSON.stringify(mockResponse));
        localStorage.setItem('current_order_id', mockOrderId);
        
        // Navigate to checkout page
        window.location.href = `/checkout/monero?order_id=${mockOrderId}`;
        return;
      }
      
      // Send real checkout request
      const response = await fetch(`${API_BASE_URL}/monero/checkout/monero`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': $auth.token ? `Bearer ${$auth.token}` : ''
        },
        body: JSON.stringify(checkoutData)
      });
      
      if (!response.ok) {
        throw new Error(`Server responded with status: ${response.status}`);
      }
      
      const data = await response.json();
      console.log("Checkout response:", data);
      
      if (data.success) {
        // Store checkout data in localStorage
        localStorage.setItem('checkoutData', JSON.stringify(data));
        localStorage.setItem('current_order_id', data.order_id);
        
        // Navigate to checkout page with order ID in URL
        window.location.href = `/checkout/monero?order_id=${data.order_id}`;
      } else {
        alert('Checkout failed: ' + (data.error || 'Unknown error'));
      }
    } catch (error) {
      console.error('Error during checkout:', error);
      
      // Fallback to demo checkout if server is unavailable
      if (error.message.includes('Failed to fetch') || error.message.includes('NetworkError')) {
        console.log("Network error, falling back to demo checkout");
        offlineMode = true;
        processPayment(); // Retry in offline mode
      } else {
        alert('An error occurred during checkout: ' + error.message);
      }
    }
  }
  
  function goToHomepage() {
    navigate('/products');
  }
  
  // Also update the demo-add-to-cart event handler to be more robust
  const handleDemoAddToCart = (event) => {
    console.log("Demo add to cart event received:", event.detail);
    
    if (!offlineMode) {
      offlineMode = true; // Enable offline mode when demo products are added
    }
    
    const product = event.detail;
    if (!product) return;
    
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
    
    // Update cart and localStorage
    cart = {...demoCart};
    localStorage.setItem('demoCart', JSON.stringify(demoCart));
    console.log("Updated cart after demo add:", cart);
  };
  
  // Set up the event listener properly
  onMount(() => {
    window.addEventListener('demo-add-to-cart', handleDemoAddToCart);
    
    return () => {
      window.removeEventListener('demo-add-to-cart', handleDemoAddToCart);
    };
  });

  function getCurrentUserId() {
    try {
      return $auth.user ? $auth.user.id : 'guest-user';
    } catch (e) {
      console.error('Error getting current user ID:', e);
      return 'guest-user';
    }
  }

  function backToCart() {
    checkoutStage = 'cart';
  }

  // Update the showShippingForm function
  function showShippingForm() {
    console.log("Show shipping form called");
    if (!cart || !cart.items || cart.items.length === 0) {
      alert('Your cart is empty!');
      return;
    }
    
    // Set stage to shipping
    checkoutStage = 'shipping';
    checkoutStageStore.set('shipping');
    
    // Force a UI update
    cart = {...cart};
    
    // Better debug output
    console.log("CHECKOUT STAGE CHANGED TO:", checkoutStage);
    console.log("CHECKOUT STORE VALUE:", $checkoutStageStore);
    
    // Check for the modal element with the correct class
    setTimeout(() => {
      const modalElement = document.querySelector('.modal-overlay');
      console.log("Modal overlay element found:", modalElement !== null);
      
      if (!modalElement) {
        console.error("CRITICAL: Modal overlay not found in DOM!");
        // Force show the modal by updating the DOM directly as a fallback
        const container = document.querySelector('.cart-container');
        if (container) {
          // This is a last resort to force-add the modal
          console.log("Attempting to force-show modal...");
          const modalHTML = `
            <div class="modal-overlay" style="position:fixed; top:0; left:0; right:0; bottom:0; background-color:rgba(0,0,0,0.75); display:flex; justify-content:center; align-items:center; z-index:1000;">
              <div class="modal-content" style="background-color:white; padding:20px; border-radius:5px; max-width:500px; width:100%;">
                <h2 style="text-align:center;">Enter Shipping Information</h2>
                <p>If you're seeing this emergency modal, the normal form isn't displaying correctly.</p>
                <button onclick="window.location.reload()">Reload Page</button>
              </div>
            </div>
          `;
          container.insertAdjacentHTML('beforeend', modalHTML);
        }
      }
    }, 100);
  }
</script>

<div class="cart-container">
  <h1>Your Shopping Cart</h1>
  
  <!-- Debug info -->
  {#if window.location.hostname === 'localhost'}
    <div class="debug-info">
      <p>Loading: {loading}</p>
      <p>Error: {error ? error : 'None'}</p>
      <p>Offline Mode: {offlineMode ? 'Yes' : 'No'}</p>
      <p>Authentication: {$auth.isAuthenticated ? 'Yes' : 'No'}</p>
      <p>User ID: {$auth.user ? $auth.user.id : 'Not logged in'}</p>
      <p>Cart Items: {cart && cart.items ? cart.items.length : '0'}</p>
    </div>
  {/if}
  
  {#if offlineMode}
    <div class="offline-notice">
      <p>You are in demo mode. Cart functionality is simulated.</p>
    </div>
  {/if}
  
  {#if loading}
    <div class="loading">
      Loading your cart...
    </div>
  {:else if error && !cart}
    <div class="error">
      <p>Error: {error}</p>
      <button on:click={loadCart}>Try Again</button>
    </div>
  {:else if !$auth.isAuthenticated}
    <div class="not-logged-in">
      <p>Please log in to view your cart.</p>
      <a href="/login" class="login-link">Log In</a>
    </div>
  {:else if cart && cart.items && cart.items.length > 0}
    <div class="cart-items">
      <table>
        <thead>
          <tr>
            <th>Product</th>
            <th>Price</th>
            <th>Quantity</th>
            <th>Total</th>
            <th>Action</th>
          </tr>
        </thead>
        <tbody>
          {#each cart.items as item, index (item.product_id || index)}
            <tr>
              <td>{item.name}</td>
              <td>${parseFloat(item.price).toFixed(2)}</td>
              <td>{item.quantity}</td>
              <td>${(item.price * item.quantity).toFixed(2)}</td>
              <td>
                <button class="btn-remove" on:click={() => removeItem(cart.id, index)}>
                  Remove
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
        <tfoot>
          <tr>
            <td colspan="3" class="total-label">Total:</td>
            <td colspan="2" class="total-value">
              ${calculateTotal().toFixed(2)}
            </td>
          </tr>
        </tfoot>
      </table>

      <div class="cart-actions">
        <button class="btn-checkout" on:click={() => showShippingForm()}>
          {offlineMode ? 'Demo Checkout' : 'Proceed to Checkout'}
        </button>
        <button class="btn-continue" on:click={goToHomepage}>
          Continue Shopping
        </button>
      </div>
    </div>

    <!-- Add this debug display to show the current checkout stage -->
    {#if window.location.hostname === 'localhost'}
      <div class="debug-checkout-stage">
        <p>Current stage: {checkoutStage}</p>
        <p>Store stage: {$checkoutStageStore}</p>
        <button on:click={showShippingForm}>Force Show Shipping Form</button>
      </div>
    {/if}

    <!-- Make the shipping form simpler and more likely to render -->
    {#if checkoutStage === 'shipping'}
      <div class="modal-overlay">
        <div class="modal-content">
          <h2>Enter Shipping Information</h2>
          
          <form on:submit|preventDefault={submitShippingInfo}>
            <div class="form-group">
              <label for="name">Full Name</label>
              <input type="text" id="name" bind:value={shippingInfo.name} required>
            </div>
            
            <div class="form-group">
              <label for="email">Email</label>
              <input type="email" id="email" bind:value={shippingInfo.email} required>
            </div>
            
            <div class="form-group">
              <label for="address">Address</label>
              <input type="text" id="address" bind:value={shippingInfo.address} required>
            </div>
            
            <div class="form-group">
              <label for="city">City</label>
              <input type="text" id="city" bind:value={shippingInfo.city} required>
            </div>
            
            <div class="form-group">
              <label for="state">State/Province</label>
              <input type="text" id="state" bind:value={shippingInfo.state} required>
            </div>
            
            <div class="form-group">
              <label for="zip">Zip/Postal Code</label>
              <input type="text" id="zip" bind:value={shippingInfo.zip} required>
            </div>
            
            <div class="form-group">
              <label for="country">Country</label>
              <input type="text" id="country" bind:value={shippingInfo.country} required>
            </div>
            
            <div class="form-actions">
              <button type="button" on:click={() => checkoutStage = 'cart'}>
                Back to Cart
              </button>
              <button type="submit">
                Continue to Payment
              </button>
            </div>
          </form>
        </div>
      </div>
    {/if}
  {:else}
    <div class="empty-cart">
      <p>Your cart is empty.</p>
      <button class="btn-continue" on:click={goToHomepage}>
        Start Shopping
      </button>
    </div>
  {/if}

  <!-- Add this at the top after the h1 -->
  {#if checkoutStage === 'shipping'}
    <div class="stage-indicator">
      Currently in shipping stage
    </div>
  {/if}
</div>

<style>
  .offline-notice {
    background: #cce5ff;
    border-left: 4px solid #007bff;
    padding: 10px 15px;
    margin-bottom: 20px;
    border-radius: 4px;
  }
  
  .debug-info {
    background: #f8f9fa;
    padding: 10px;
    border-radius: 5px;
    margin-bottom: 20px;
    font-family: monospace;
    font-size: 12px;
  }
  
  .cart-container {
    max-width: 800px;
    margin: 0 auto;
    padding: 20px;
  }
  
  .not-logged-in {
    text-align: center;
    padding: 30px;
    background: #f8f9fa;
    border-radius: 4px;
    margin: 20px 0;
  }
  
  .login-link {
    display: inline-block;
    margin-top: 10px;
    background: #3498db;
    color: white;
    padding: 8px 16px;
    border-radius: 4px;
    text-decoration: none;
  }

  .cart-container {
    max-width: 800px;
    margin: 0 auto;
    padding: 20px;
  }

  h1 {
    text-align: center;
    margin-bottom: 30px;
  }

  .loading, .error, .empty-cart {
    text-align: center;
    padding: 20px;
    margin: 20px 0;
  }

  .error {
    color: #e74c3c;
    background-color: #fdecea;
    border: 1px solid #fadbd8;
    border-radius: 4px;
  }

  .cart-items table {
    width: 100%;
    border-collapse: collapse;
    margin-bottom: 20px;
  }

  .cart-items th, .cart-items td {
    padding: 12px;
    text-align: left;
    border-bottom: 1px solid #ddd;
  }

  .cart-items th {
    background-color: #f8f9fa;
    font-weight: bold;
  }

  .total-label {
    text-align: right;
    font-weight: bold;
  }

  .total-value {
    font-weight: bold;
    font-size: 1.2em;
  }

  .cart-actions {
    display: flex;
    justify-content: space-between;
    margin-top: 20px;
  }

  .btn-checkout, .btn-continue, .btn-remove {
    padding: 10px 15px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-weight: bold;
    transition: background-color 0.3s;
  }

  .btn-checkout {
    background-color: #2ecc71;
    color: white;
  }

  .btn-checkout:hover {
    background-color: #27ae60;
  }

  .btn-continue {
    background-color: #3498db;
    color: white;
  }

  .btn-continue:hover {
    background-color: #2980b9;
  }

  .btn-remove {
    background-color: #e74c3c;
    color: white;
    padding: 5px 10px;
    font-size: 0.9em;
  }

  .btn-remove:hover {
    background-color: #c0392b;
  }

  @media (max-width: 600px) {
    .cart-actions {
      flex-direction: column;
      gap: 10px;
    }
    
    .btn-checkout, .btn-continue {
      width: 100%;
    }
  }

  .modal-overlay {
    position: fixed !important;
    top: 0 !important;
    left: 0 !important;
    right: 0 !important;
    bottom: 0 !important;
    background-color: rgba(0, 0, 0, 0.85) !important;
    display: flex !important;
    justify-content: center !important;
    align-items: center !important;
    z-index: 9999 !important; /* Super high z-index */
  }
  
  .modal-content {
    background-color: white !important;
    padding: 25px !important;
    border-radius: 8px !important;
    max-width: 550px !important;
    width: 90% !important;
    box-shadow: 0 5px 20px rgba(0,0,0,0.3) !important;
    border: 2px solid #2ecc71 !important;
  }
  
  .debug-checkout-stage {
    margin-top: 20px;
    padding: 10px;
    background-color: #f8f9fa;
    border: 1px solid #ddd;
    border-radius: 4px;
  }

  /* Add these missing styles for the form elements */
  .form-group {
    margin-bottom: 15px;
  }
  
  .modal-content label {
    display: block;
    margin-bottom: 5px;
    font-weight: bold;
    color: #555;
  }
  
  .modal-content input {
    width: 100%;
    padding: 10px;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 16px;
  }
  
  .form-actions {
    display: flex;
    justify-content: space-between;
    margin-top: 20px;
  }
  
  .form-actions button {
    padding: 10px 15px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-weight: bold;
  }
  
  .form-actions button[type="submit"] {
    background-color: #2ecc71;
    color: white;
  }
  
  .form-actions button[type="button"] {
    background-color: #95a5a6;
    color: white;
  }
  
  .form-actions button[type="submit"]:hover {
    background-color: #27ae60;
  }
  
  .form-actions button[type="button"]:hover {
    background-color: #7f8c8d;
  }
  
  .modal-content h2 {
    margin-bottom: 20px;
    color: #333;
    text-align: center;
  }

  /* Add this for the stage indicator */
  .stage-indicator {
    background-color: #4CAF50;
    color: white;
    padding: 10px;
    margin-bottom: 15px;
    border-radius: 4px;
    text-align: center;
    font-weight: bold;
  }
</style> 