<script>
  import { onMount } from 'svelte';
  import { auth } from '../stores/auth.js';
  import { navigate } from 'svelte-routing';

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
  
  function checkout() {
    if (offlineMode) {
      alert('This is a demo checkout. In a real app, you would proceed to payment.');
      return;
    }
    
    navigate('/checkout');
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
        <button class="btn-checkout" on:click={checkout}>
          {offlineMode ? 'Demo Checkout' : 'Proceed to Checkout'}
        </button>
        <button class="btn-continue" on:click={goToHomepage}>
          Continue Shopping
        </button>
      </div>
    </div>
  {:else}
    <div class="empty-cart">
      <p>Your cart is empty.</p>
      <button class="btn-continue" on:click={goToHomepage}>
        Start Shopping
      </button>
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
</style> 