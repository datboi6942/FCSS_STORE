<script lang="ts">
    import { onMount } from 'svelte';
    import { auth } from '../stores/auth.js';
    import ProductCard from './ProductCard.svelte';
    import { cartItems } from '../stores/cart.js';
    import { fade } from 'svelte/transition';
    
    // Direct API URL
    const API_URL = 'http://localhost:5000/products';
    let products = [];
    let newProduct = { name: '', description: '', price: 0, available: true };
    let loading = true;
    let error = null;
    let offlineMode = false;
    let serverStatus = "unknown"; // "online", "offline", or "unknown"
  
    // Fallback products when API fails
    const fallbackProducts = [
      { 
        id: "fb1", 
        name: "Deluxe Crypto Hardware Wallet", 
        description: "Secure your cryptocurrency with military-grade encryption and biometric protection.", 
        price: 149.99, 
        available: true 
      },
      { 
        id: "fb2", 
        name: "Ultra-Secure Password Manager", 
        description: "Store all your passwords with end-to-end encryption and zero-knowledge architecture.", 
        price: 79.99, 
        available: true 
      },
      { 
        id: "fb3", 
        name: "Privacy VPN Premium", 
        description: "Browse the internet with complete anonymity and no activity logs.", 
        price: 99.99, 
        available: true 
      }
    ];
  
    // Queue for pending product additions
    let pendingProducts = [];
    let syncInProgress = false;
  
    // Load pending products from localStorage
    function loadPendingProducts() {
      try {
        const stored = localStorage.getItem('pendingProducts');
        if (stored) {
          pendingProducts = JSON.parse(stored);
          console.log(`Loaded ${pendingProducts.length} pending products from storage`);
        }
      } catch (err) {
        console.error("Error loading pending products:", err);
        pendingProducts = [];
      }
    }
  
    // Save pending products to localStorage
    function savePendingProducts() {
      localStorage.setItem('pendingProducts', JSON.stringify(pendingProducts));
      console.log(`Saved ${pendingProducts.length} pending products to storage`);
    }
  
    // Add a product to the pending queue
    function addToPendingQueue(product) {
      // Add timestamp and ID if not present
      const productToQueue = {
        ...product,
        id: product.id || `pending-${Date.now()}`,
        queued_at: new Date().toISOString()
      };
      
      pendingProducts.push(productToQueue);
      savePendingProducts();
      
      // Also add to local products array for immediate display
      products = [...products, productToQueue];
    }
  
    // Try to sync pending products with server
    async function syncPendingProducts() {
      if (syncInProgress || pendingProducts.length === 0) return;
      
      syncInProgress = true;
      console.log(`Attempting to sync ${pendingProducts.length} pending products`);
      
      // Keep track of products that were successfully synced
      const syncedProducts = [];
      
      for (const product of pendingProducts) {
        try {
          console.log(`Syncing product: ${product.name}`);
          
          // Remove any temporary IDs or metadata
          const { id, queued_at, ...productData } = product;
          
          const response = await fetch(API_URL, {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
              'Authorization': $auth.token ? `Bearer ${$auth.token}` : ''
            },
            body: JSON.stringify(productData)
          });
          
          if (response.ok) {
            console.log(`Successfully synced product: ${product.name}`);
            syncedProducts.push(product);
          } else {
            console.error(`Failed to sync product: ${product.name}`);
          }
        } catch (err) {
          console.error(`Error syncing product ${product.name}:`, err);
          // Continue with next product
        }
      }
      
      // Remove synced products from pending queue
      if (syncedProducts.length > 0) {
        pendingProducts = pendingProducts.filter(p => 
          !syncedProducts.some(sp => sp.id === p.id)
        );
        savePendingProducts();
        
        // Reload products from server to get proper IDs
        await fetchProducts();
        
        alert(`Successfully synchronized ${syncedProducts.length} products with the server.`);
      }
      
      syncInProgress = false;
    }
  
    // Simplified cart update handler
    function handleCartUpdated() {
      console.log("Cart updated in Products");
    }
  
    // Add a more robust fetchProducts function with retries
    async function fetchProducts() {
      let retries = 3;
      let lastError = null;
      
      while (retries > 0) {
        try {
          loading = true;
          error = null;
          
          const controller = new AbortController();
          const timeoutId = setTimeout(() => controller.abort(), 8000);
          
          console.log(`Fetching products (attempt ${4-retries}/3)...`);
          
          const response = await fetch(API_URL, {
            signal: controller.signal,
            headers: {
              'Authorization': $auth.token ? `Bearer ${$auth.token}` : ''
            }
          });
          
          clearTimeout(timeoutId);
          
          if (!response.ok) {
            throw new Error(`Server returned ${response.status}`);
          }
          
          const data = await response.json();
          console.log(`Got products: ${data.length}`);
          
          // Success - update products and exit retry loop
          products = data;
          loading = false;
          
          // Check server status after successful fetch
          checkServerStatus();
          
          return;
        } catch (err) {
          lastError = err;
          console.warn(`Fetch attempt failed (${retries} retries left): ${err.message}`);
          retries--;
          
          // Wait before retrying
          if (retries > 0) {
            await new Promise(resolve => setTimeout(resolve, 1000));
          }
        }
      }
      
      // If we get here, all retries failed
      console.error("Error loading products:", lastError);
      error = `Failed to load products: ${lastError?.message}`;
      loading = false;
      
      // Fall back to offline mode
      useOfflineMode();
    }
  
    // Call fetchProducts directly on mount
    onMount(() => {
      console.log("Products component mounted");
      loadPendingProducts();
      setTimeout(fetchProducts, 100);
      
      // Check server status immediately
      checkServerStatus();
      
      // Set up periodic server status check
      const statusInterval = setInterval(checkServerStatus, 10000); // Every 10 seconds
      
      // Set up connectivity check interval
      const checkInterval = setInterval(() => {
        if (offlineMode && pendingProducts.length > 0) {
          // Try to connect to the server periodically
          fetch(API_URL, { method: 'HEAD', timeout: 2000 })
            .then(() => {
              console.log("Server connectivity restored");
              offlineMode = false;
              syncPendingProducts();
            })
            .catch(() => {
              // Still offline
            });
        }
      }, 30000); // Check every 30 seconds
      
      return () => {
        clearInterval(checkInterval);
        clearInterval(statusInterval);
      };
    });
  
    // Admin function to add new product
    async function addProduct() {
      try {
        console.log("Adding new product:", newProduct);
        
        // Clear any previous errors
        error = null;
        
        // Validate input
        if (!newProduct.name || !newProduct.description || newProduct.price <= 0) {
          error = "Please fill out all fields with valid values";
          return;
        }
        
        // Create a copy of the product to avoid reference issues
        const productToAdd = { ...newProduct };
        
        // For offline mode, queue the product
        if (offlineMode) {
          console.log("Offline mode - adding product to pending queue");
          addToPendingQueue(productToAdd);
          alert("Product saved to queue. It will be synchronized with the server when connection is restored.");
          
          // Reset form
          newProduct = { name: '', description: '', price: 0, available: true };
          return;
        }
        
        // For online mode, try the API with a timeout
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), 5000);
        
        try {
          const response = await fetch(API_URL, {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
              'Authorization': $auth.token ? `Bearer ${$auth.token}` : ''
            },
            body: JSON.stringify(productToAdd),
            signal: controller.signal
          });
          
          clearTimeout(timeoutId);
          
          if (!response.ok) {
            throw new Error('Failed to add product');
          }
          
          // If successful, get the product data from the response
          try {
            const serverProduct = await response.json();
            // Add the server-assigned ID and other properties
            products = [...products, serverProduct];
          } catch (err) {
            // If we can't parse the response, just add the product with a temp ID
            const tempId = Date.now().toString();
            products = [...products, {...productToAdd, id: tempId}];
          }
          
          // Reset form
          newProduct = { name: '', description: '', price: 0, available: true };
          
          alert("Product added successfully!");
        } catch (err) {
          if (err.name === 'AbortError') {
            // If server request times out, add to pending queue
            console.log("Server timeout, adding product to pending queue");
            offlineMode = true;
            addToPendingQueue(productToAdd);
            alert("Server connection timed out. Product saved to queue and will be synchronized when connection is restored.");
            
            // Reset form
            newProduct = { name: '', description: '', price: 0, available: true };
          } else {
            throw err;
          }
        }
      } catch (err) {
        error = err.message;
        console.error("Error adding product:", err);
      }
    }
  
    // Add a function to check if a product is from the database or local
    function isProductInDatabase(product) {
      // Products from the database will have a real ID, not a pending- or local- prefix
      return product.id && !product.id.startsWith('pending-') && !product.id.startsWith('local-') && !product.id.startsWith('fb');
    }
  
    // Get all product IDs from database to verify which products exist
    async function verifyDatabaseProducts() {
      try {
        const controller = new AbortController();
        const timeoutId = setTimeout(() => controller.abort(), 5000);
        
        console.log("Verifying products in database...");
        
        const response = await fetch(`${API_URL}/ids`, {
          signal: controller.signal,
          headers: {
            'Authorization': $auth.token ? `Bearer ${$auth.token}` : ''
          }
        }).catch(err => {
          if (err.name === 'AbortError') {
            throw new Error('Request timed out');
          }
          throw err;
        });
        
        clearTimeout(timeoutId);
        
        if (!response.ok) {
          throw new Error(`Server error: ${response.status}`);
        }
        
        const dbIds = await response.json();
        console.log("Database product IDs:", dbIds);
        
        // Update products to show which are in the database
        products = products.map(product => ({
          ...product,
          inDatabase: dbIds.includes(product.id)
        }));
        
        return dbIds;
      } catch (err) {
        console.error("Error verifying database products:", err);
        return null;
      }
    }
  
    // Update the checkServerStatus function to work around CORS limitations
    async function checkServerStatus() {
      try {
        console.log("Starting server status check without CORS checks...");
        
        // Instead of directly checking server status, use the products that we already fetched
        // If products were fetched successfully, the server must be online
        if (products.length > 0 && products.some(p => !p.id.startsWith('fb'))) {
          // We have at least one non-fallback product, server must be online
          serverStatus = "online";
          console.log("Server is online (detected from product data)");
          
          // If we can infer the server is online, we should also update offline mode
          if (offlineMode) {
            console.log("Server connection restored, enabling online mode");
            offlineMode = false;
            
            // Try to sync pending products if we have any
            if (pendingProducts.length > 0) {
              setTimeout(() => syncPendingProducts(), 1000);
            }
          }
          
          return true;
        }
        
        // No real products found, server might be offline
        serverStatus = "offline";
        console.log("Server appears to be offline (no real products found)");
        return false;
        
      } catch (err) {
        serverStatus = "offline";
        console.log("Server status check error:", err.message);
        return false;
      }
    }

    async function checkProductsServer() {
      // Check if a cached health status exists (valid for 15 seconds)
      const cached = sessionStorage.getItem('productsHealthStatus');
      const cachedTimestamp = sessionStorage.getItem('productsHealthTimestamp');
      if (cached && cachedTimestamp) {
        const diff = Date.now() - parseInt(cachedTimestamp, 10);
        if (diff < 15000) {
          console.log("Using cached product server status:", cached);
          return cached === 'true';
        }
      }

      try {
        const response = await fetch('http://localhost:5000/health', {
          signal: AbortSignal.timeout(5000)
        });
        const online = response.ok;

        // Cache the result with the current timestamp
        sessionStorage.setItem('productsHealthStatus', online.toString());
        sessionStorage.setItem('productsHealthTimestamp', Date.now().toString());

        return online;
      } catch (err) {
        console.error("Products health check error:", err);
        sessionStorage.setItem('productsHealthStatus', 'false');
        sessionStorage.setItem('productsHealthTimestamp', Date.now().toString());
        return false;
      }
    }

    function addToCart(product) {
      cartItems.update(items => {
        const existingItem = items.find(item => item.id === product.id);
        if (existingItem) {
          return items.map(item => 
            item.id === product.id 
              ? {...item, quantity: item.quantity + 1}
              : item
          );
        }
        return [...items, { ...product, quantity: 1 }];
      });
    }

    // Add this function to handle "Buy Now"
    function handleBuyNow(product) {
      // Clear the cart first
      cartItems.set([]);
      // Add this product to cart
      cartItems.update(items => [...items, { ...product, quantity: 1 }]);
      // Dispatch custom event to show shipping form
      const event = new CustomEvent('showShipping', {
        bubbles: true,
        composed: true
      });
      document.dispatchEvent(event);
    }
</script>

<div class="products-container">
  <div class="products-header">
    <h1>Our Products</h1>
    <div class="header-actions">
      <div class="server-status {serverStatus}">
        Server: 
        {#if serverStatus === "online"}
          <span class="status-indicator online">🟢 Online</span>
        {:else if serverStatus === "offline"}
          <span class="status-indicator offline">🔴 Offline</span>
        {:else}
          <span class="status-indicator unknown">⚪ Checking...</span>
        {/if}
      </div>
      <button class="refresh-btn" on:click={fetchProducts}>
        Refresh Products
      </button>
    </div>
  </div>
  
  <!-- Only show debug info in development mode -->
  {#if window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1'}
    <div class="debug-info">
      <p>Loading: {loading}</p>
      <p>Error: {error}</p>
      <p>Products count: {products.length}</p>
      <p>Offline mode: {offlineMode ? 'Yes' : 'No'}</p>
      <p>Pending products: {pendingProducts.length}</p>
    </div>
  {/if}
  
  {#if loading}
    <div class="loading">Loading products...</div>
  {:else if error && products.length === 0}
    <div class="error">
      <p>Error: {error}</p>
      <button on:click={() => fetchProducts()}>Retry</button>
    </div>
  {:else if products.length === 0}
    <div class="no-products">No products available.</div>
  {:else}
    <!-- Show a nicer notification when using fallback products -->
    {#if error}
      <div class="fallback-notice">
        <p>Note: {error}</p>
        <button class="retry-btn" on:click={() => fetchProducts()}>Try again</button>
      </div>
    {/if}
    
    <div class="products-grid">
      {#each products as product (product.id)}
        <div class="product-card" transition:fade>
          <img src={product.image || '/placeholder.png'} alt={product.name} />
          <h3>{product.name}</h3>
          <p>{product.description}</p>
          <div class="price">${product.price.toFixed(2)}</div>
          <div class="button-group">
            <!-- Update the button to use handleBuyNow -->
            <button 
              class="buy-now" 
              on:click={() => handleBuyNow(product)}
            >
              Buy Now
            </button>
            <button 
              class="add-to-cart" 
              on:click={() => addToCart(product)}
            >
              Add to Cart
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
  
  <!-- Add a sync button if there are pending products -->
  {#if pendingProducts.length > 0}
    <div class="sync-banner">
      <p>You have {pendingProducts.length} product(s) waiting to be synchronized with the server.</p>
      <button 
        class="sync-button" 
        on:click={syncPendingProducts} 
        disabled={syncInProgress || offlineMode}
      >
        {#if syncInProgress}
          Syncing...
        {:else if offlineMode}
          Offline (Sync Later)
        {:else}
          Sync Now
        {/if}
      </button>
    </div>
  {/if}
  
  <!-- Only show for admin users -->
  {#if $auth.isAdmin}
    <div class="admin-section">
      <h3>Add New Product</h3>
      
      <!-- Add debug info for admin -->
      {#if window.location.hostname === 'localhost' || window.location.hostname === '127.0.0.1'}
        <div class="admin-debug">
          <p>Admin mode: {$auth.isAdmin ? 'Active' : 'Inactive'}</p>
          <p>Token: {$auth.token ? 'Present' : 'Missing'}</p>
          <p>Offline mode: {offlineMode ? 'Yes' : 'No'}</p>
        </div>
      {/if}
      
      {#if offlineMode}
        <div class="admin-offline-notice">
          <p>You are in demo mode. New products will only be saved locally.</p>
        </div>
      {/if}
      
      <form on:submit|preventDefault={addProduct}>
        <div class="form-group">
          <label for="product-name">Name:</label>
          <input id="product-name" bind:value={newProduct.name} required />
        </div>
        
        <div class="form-group">
          <label for="product-desc">Description:</label>
          <textarea id="product-desc" bind:value={newProduct.description} required></textarea>
        </div>
        
        <div class="form-group">
          <label for="product-price">Price:</label>
          <input type="number" id="product-price" bind:value={newProduct.price} min="0" step="0.01" required />
        </div>
        
        <div class="form-group">
          <label>
            <input type="checkbox" bind:checked={newProduct.available} />
            Available
          </label>
        </div>
        
        <div class="admin-actions">
          <button class="verify-btn" on:click={verifyDatabaseProducts}>
            Verify Database Products
          </button>
        </div>
        
        <button type="submit">Add Product</button>
      </form>
      
      {#if error}
        <div class="admin-error">
          {error}
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .products-container {
    max-width: 1200px;
    margin: 0 auto;
    padding: 20px;
  }
  
  .debug-info {
    background: #f5f5f5;
    padding: 10px;
    border-radius: 5px;
    margin-bottom: 20px;
    font-family: monospace;
  }
  
  .loading, .error, .no-products {
    text-align: center;
    padding: 30px;
    background: #f8f9fa;
    border-radius: 5px;
    margin: 20px 0;
  }
  
  .error {
    color: #e74c3c;
    background: #fdecea;
  }
  
  .products-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
    gap: 20px;
    padding: 20px;
  }
  
  .admin-section {
    background: #f8f9fa;
    padding: 1.5rem;
    border-radius: 8px;
    margin-top: 2rem;
    border-left: 4px solid #1e90ff;
  }
  
  .form-group {
    margin-bottom: 1rem;
  }
  
  label {
    display: block;
    margin-bottom: 0.5rem;
  }
  
  input, textarea {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid #ddd;
    border-radius: 4px;
  }
  
  textarea {
    min-height: 100px;
    resize: vertical;
  }
  
  button {
    background: #1e90ff;
    color: white;
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: 4px;
    cursor: pointer;
  }
  
  button:hover {
    background: #167edb;
  }
  
  @media (max-width: 768px) {
    .products-grid {
      grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
    }
  }
  
  .fallback-notice {
    background: #fff3cd;
    border-left: 4px solid #ffc107;
    padding: 10px 15px;
    margin-bottom: 20px;
    border-radius: 4px;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  
  .retry-btn {
    background: #ffc107;
    color: #212529;
    border: none;
    padding: 6px 12px;
    border-radius: 4px;
    cursor: pointer;
    font-size: 14px;
  }
  
  .retry-btn:hover {
    background: #e0a800;
  }
  
  .admin-debug {
    background: #f8f9fa;
    padding: 8px 12px;
    border-radius: 4px;
    margin-bottom: 15px;
    font-family: monospace;
    font-size: 12px;
  }
  
  .admin-offline-notice {
    background: #fff3cd;
    border-left: 4px solid #ffc107;
    padding: 10px 15px;
    margin-bottom: 15px;
    border-radius: 0 4px 4px 0;
  }
  
  .admin-error {
    color: #721c24;
    background-color: #f8d7da;
    border: 1px solid #f5c6cb;
    padding: 10px;
    border-radius: 4px;
    margin-top: 15px;
  }
  
  .sync-banner {
    background: #e3f2fd;
    border-left: 4px solid #2196f3;
    padding: 10px 15px;
    margin: 20px 0;
    border-radius: 0 4px 4px 0;
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  
  .sync-button {
    background: #2196f3;
    color: white;
    border: none;
    padding: 8px 15px;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .sync-button:hover:not(:disabled) {
    background: #1976d2;
  }
  
  .sync-button:disabled {
    background: #bbdefb;
    cursor: not-allowed;
  }
  
  .product-card {
    background: white;
    border-radius: 8px;
    padding: 15px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    transition: transform 0.2s;
  }
  
  .product-card:hover {
    transform: translateY(-5px);
  }
  
  .product-card img {
    width: 100%;
    height: 200px;
    object-fit: cover;
    border-radius: 4px;
  }
  
  .product-card h3 {
    margin: 10px 0;
    color: #333;
  }
  
  .price {
    font-size: 1.2em;
    font-weight: bold;
    color: #2ecc71;
    margin: 10px 0;
  }
  
  .button-group {
    display: flex;
    gap: 10px;
    margin-top: 10px;
  }
  
  .buy-now {
    background-color: #2ecc71;
    color: white;
  }
  
  .buy-now:hover {
    background-color: #27ae60;
  }
  
  .add-to-cart {
    background-color: #3498db;
    color: white;
  }
  
  .add-to-cart:hover {
    background-color: #2980b9;
  }
  
  .products-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
  }
  
  .header-actions {
    display: flex;
    gap: 15px;
    align-items: center;
  }
  
  .server-status {
    font-size: 14px;
    display: flex;
    align-items: center;
    gap: 5px;
  }
  
  .status-indicator {
    font-weight: bold;
    padding: 4px 8px;
    border-radius: 4px;
  }
  
  .status-indicator.online {
    background-color: #d4edda;
    color: #155724;
  }
  
  .status-indicator.offline {
    background-color: #f8d7da;
    color: #721c24;
  }
  
  .status-indicator.unknown {
    background-color: #f8f9fa;
    color: #6c757d;
  }
  
  .refresh-btn {
    padding: 8px 12px;
    background-color: #007bff;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .refresh-btn:hover {
    background-color: #0069d9;
  }
</style>