<script>
  import { onMount } from 'svelte';
  import { apiCall } from '../api.js';
  import { auth } from '../stores/auth.js';
  
  let products = [];
  let isLoading = true;
  let error = null;
  let isAdmin = false;
  
  // For new product form (admin only)
  let showForm = false;
  let newProduct = {
    name: '',
    description: '',
    price: '',
    available: true
  };
  
  onMount(async () => {
    await fetchProducts();
    checkAdminStatus();
  });
  
  function checkAdminStatus() {
    const unsubscribe = auth.subscribe(authData => {
      if (authData.isAuthenticated) {
        fetchUserRole(authData.token);
      }
    });
    
    return unsubscribe;
  }
  
  async function fetchUserRole(token) {
    try {
      const response = await apiCall('/auth/profile');
      
      if (response.ok) {
        const userData = await response.json();
        isAdmin = userData.role === 'admin';
      }
    } catch (error) {
      console.error('Failed to fetch user profile', error);
    }
  }
  
  async function fetchProducts() {
    try {
      isLoading = true;
      const response = await apiCall('/products');
      
      if (response.ok) {
        products = await response.json();
      } else {
        error = 'Failed to load products';
      }
    } catch (err) {
      error = err.message || 'Failed to fetch products';
      console.error(error);
    } finally {
      isLoading = false;
    }
  }
  
  async function addProduct() {
    if (!newProduct.name || !newProduct.description || !newProduct.price) {
      error = 'Please fill out all required fields';
      return;
    }
    
    try {
      const response = await apiCall('/products', {
        method: 'POST',
        body: JSON.stringify(newProduct)
      });
      
      if (response.ok) {
        // Reset form
        newProduct = {
          name: '',
          description: '',
          price: '',
          available: true
        };
        showForm = false;
        
        // Refresh product list
        await fetchProducts();
      } else {
        error = 'Failed to add product';
      }
    } catch (err) {
      error = err.message || 'Error adding product';
      console.error(error);
    }
  }
  
  async function createOrder(productId) {
    try {
      const authValue = get(auth);
      if (!authValue.isAuthenticated) {
        alert('Please log in to place an order');
        return;
      }
      
      const response = await apiCall('/order', {
        method: 'POST',
        body: JSON.stringify({
          product_id: productId
        })
      });
      
      if (response.ok) {
        const data = await response.json();
        alert(`Order created successfully! Order ID: ${data.order_id}`);
      } else {
        const errorData = await response.json();
        alert(`Failed to create order: ${errorData.error || 'Unknown error'}`);
      }
    } catch (error) {
      console.error('Error creating order:', error);
      alert('Failed to create order');
    }
  }
</script>

<div class="product-list">
  <h1>Products</h1>
  
  {#if isLoading}
    <div class="loading">Loading products...</div>
  {:else if error}
    <div class="error-message">{error}</div>
  {:else}
    <div class="products-grid">
      {#each products as product}
        <div class="product-card">
          <h3>{product.name}</h3>
          <p class="description">{product.description}</p>
          <p class="price">${product.price}</p>
          <div class="status">
            {#if product.available}
              <span class="in-stock">In Stock</span>
            {:else}
              <span class="out-of-stock">Out of Stock</span>
            {/if}
          </div>
          <button class="buy-button" on:click={() => createOrder(product.id)}>
            Order Now
          </button>
        </div>
      {/each}
    </div>
    
    {#if isAdmin}
      {#if showForm}
        <div class="admin-form">
          <h2>Add New Product</h2>
          <form on:submit|preventDefault={addProduct}>
            <div class="form-group">
              <label for="name">Product Name</label>
              <input id="name" bind:value={newProduct.name} required />
            </div>
            
            <div class="form-group">
              <label for="description">Description</label>
              <textarea id="description" bind:value={newProduct.description} required></textarea>
            </div>
            
            <div class="form-group">
              <label for="price">Price</label>
              <input id="price" type="number" step="0.01" bind:value={newProduct.price} required />
            </div>
            
            <div class="form-group">
              <label>
                <input type="checkbox" bind:checked={newProduct.available} />
                Available in Stock
              </label>
            </div>
            
            <div class="form-actions">
              <button type="submit" class="submit-button">Add Product</button>
              <button type="button" class="cancel-button" on:click={() => showForm = false}>Cancel</button>
            </div>
          </form>
        </div>
      {:else}
        <button class="add-product-button" on:click={() => showForm = true}>
          Add New Product
        </button>
      {/if}
    {/if}
  {/if}
</div>

<style>
  .product-list {
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem 1rem;
  }
  
  h1 {
    margin-bottom: 2rem;
    color: #333;
  }
  
  .loading {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 200px;
    font-size: 1.2rem;
    color: #666;
  }
  
  .error-message {
    background-color: #ffebee;
    color: #c62828;
    padding: 1rem;
    border-radius: 4px;
    margin-bottom: 2rem;
  }
  
  .products-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 2rem;
  }
  
  .product-card {
    background-color: white;
    border-radius: 8px;
    overflow: hidden;
    box-shadow: 0 2px 8px rgba(0,0,0,0.1);
    transition: transform 0.2s ease-in-out;
  }
  
  .product-card:hover {
    transform: translateY(-5px);
    box-shadow: 0 6px 12px rgba(0,0,0,0.15);
  }
  
  .product-card h3 {
    margin: 1.5rem 1.5rem 0.5rem;
    color: #333;
  }
  
  .description {
    margin: 0 1.5rem 1rem;
    color: #666;
    font-size: 0.9rem;
  }
  
  .price {
    margin: 0 1.5rem;
    font-size: 1.25rem;
    font-weight: bold;
    color: #2196F3;
  }
  
  .status {
    margin: 0.5rem 1.5rem;
  }
  
  .in-stock {
    color: #4CAF50;
    font-size: 0.875rem;
  }
  
  .out-of-stock {
    color: #F44336;
    font-size: 0.875rem;
  }
  
  .buy-button {
    display: block;
    width: calc(100% - 3rem);
    margin: 1rem 1.5rem 1.5rem;
    padding: 0.75rem 0;
    background-color: #2196F3;
    color: white;
    border: none;
    border-radius: 4px;
    font-weight: bold;
    cursor: pointer;
    transition: background-color 0.2s;
  }
  
  .buy-button:hover {
    background-color: #1976D2;
  }
  
  .admin-form {
    margin-top: 3rem;
    background-color: #f9f9f9;
    padding: 2rem;
    border-radius: 8px;
  }
  
  .admin-form h2 {
    margin-top: 0;
    margin-bottom: 1.5rem;
  }
  
  .form-group {
    margin-bottom: 1.5rem;
  }
  
  .form-group label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: bold;
  }
  
  input, textarea {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 1rem;
  }
  
  textarea {
    min-height: 100px;
    resize: vertical;
  }
  
  .form-actions {
    display: flex;
    gap: 1rem;
  }
  
  .submit-button {
    padding: 0.75rem 1.5rem;
    background-color: #4CAF50;
    color: white;
    border: none;
    border-radius: 4px;
    font-weight: bold;
    cursor: pointer;
  }
  
  .cancel-button {
    padding: 0.75rem 1.5rem;
    background-color: #f5f5f5;
    color: #333;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-weight: bold;
    cursor: pointer;
  }
  
  .add-product-button {
    display: block;
    margin: 2rem auto 0;
    padding: 0.75rem 1.5rem;
    background-color: #2196F3;
    color: white;
    border: none;
    border-radius: 4px;
    font-weight: bold;
    cursor: pointer;
  }
</style> 