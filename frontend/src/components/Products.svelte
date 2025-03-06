<script lang="ts">
    import { onMount } from 'svelte';
    import { auth } from '../stores/auth.js';
    import { apiCall } from '../api.js';
    
    let products = [];
    let newProduct = { name: '', description: '', price: 0, available: true };
    let loading = true;
    let error = null;
  
    async function fetchProducts() {
      try {
        loading = true;
        const res = await apiCall('/products');
        if (!res.ok) throw new Error('Failed to fetch products');
        products = await res.json();
        error = null;
      } catch (err) {
        error = err.message;
        console.error(err);
      } finally {
        loading = false;
      }
    }
  
    async function addProduct() {
      try {
        const res = await apiCall('/products', {
          method: 'POST',
          body: JSON.stringify(newProduct)
        });
        
        if (!res.ok) throw new Error('Failed to add product');
        
        await fetchProducts();
        newProduct = { name: '', description: '', price: 0, available: true };
      } catch (err) {
        error = err.message;
        console.error(err);
      }
    }
  
    onMount(() => {
      fetchProducts();
    });
</script>

<div class="products-container">
  <h2>Our Products</h2>
  
  {#if loading}
    <p>Loading products...</p>
  {:else if error}
    <div class="error-message">
      <p>{error}</p>
      <button on:click={fetchProducts}>Try Again</button>
    </div>
  {:else if products.length === 0}
    <p>No products available.</p>
  {:else}
    <div class="product-grid">
      {#each products as product}
        <div class="product-card">
          <h3>{product.name}</h3>
          <p>{product.description}</p>
          <p class="price">${product.price}</p>
          <p class="status">{product.available ? 'In Stock' : 'Out of Stock'}</p>
        </div>
      {/each}
    </div>
  {/if}
  
  <!-- Only show for admin users -->
  {#if $authStore.isAdmin}
    <div class="admin-section">
      <h3>Add New Product</h3>
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
        
        <button type="submit">Add Product</button>
      </form>
    </div>
  {/if}
</div>

<style>
  .products-container {
    max-width: 1000px;
    margin: 0 auto;
  }
  
  .product-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(250px, 1fr));
    gap: 1.5rem;
    margin: 2rem 0;
  }
  
  .product-card {
    background: white;
    border-radius: 8px;
    padding: 1.5rem;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    transition: transform 0.2s, box-shadow 0.2s;
  }
  
  .product-card:hover {
    transform: translateY(-5px);
    box-shadow: 0 5px 10px rgba(0, 0, 0, 0.1);
  }
  
  .price {
    font-weight: bold;
    color: #1e90ff;
    font-size: 1.2rem;
  }
  
  .status {
    display: inline-block;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
    font-size: 0.8rem;
    background: #e6f7ff;
    color: #1e90ff;
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
  
  .error-message {
    color: #e74c3c;
    background: #fadbd8;
    padding: 1rem;
    border-radius: 4px;
    margin: 1rem 0;
  }
</style>