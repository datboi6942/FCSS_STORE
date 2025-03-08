<script>
  import { onMount } from 'svelte';
  import { auth } from '../stores/auth.js';

  let products = [];
  let loading = true;
  let error = null;
  let newProduct = { name: '', description: '', price: 0, available: true };
  let editingProduct = null;
  let showForm = false;

  // Load products on mount
  onMount(fetchProducts);

  async function fetchProducts() {
    try {
      loading = true;
      const response = await fetch('http://localhost:5000/products', {
        headers: {
          'Authorization': `Bearer ${$auth.token}`
        }
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      products = await response.json();
      console.log('Fetched products:', products);
    } catch (e) {
      error = e.message;
      console.error('Error fetching products:', e);
    } finally {
      loading = false;
    }
  }

  async function handleSubmit() {
    try {
      const method = editingProduct ? 'PUT' : 'POST';
      const url = editingProduct 
        ? `http://localhost:5000/products/${editingProduct.id}` 
        : 'http://localhost:5000/products';
      
      const productData = editingProduct 
        ? { ...editingProduct, ...newProduct } 
        : newProduct;

      const response = await fetch(url, {
        method,
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${$auth.token}`
        },
        body: JSON.stringify(productData)
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      // Refresh products
      await fetchProducts();
      
      // Reset form
      newProduct = { name: '', description: '', price: 0, available: true };
      editingProduct = null;
      showForm = false;
    } catch (e) {
      error = e.message;
      console.error('Error saving product:', e);
    }
  }

  function editProduct(product) {
    editingProduct = product;
    newProduct = { 
      name: product.name, 
      description: product.description, 
      price: product.price,
      available: product.available 
    };
    showForm = true;
  }

  async function deleteProduct(id) {
    if (!confirm('Are you sure you want to delete this product?')) {
      return;
    }

    try {
      const response = await fetch(`http://localhost:5000/products/${id}`, {
        method: 'DELETE',
        headers: {
          'Authorization': `Bearer ${$auth.token}`
        }
      });

      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      // Refresh products
      await fetchProducts();
    } catch (e) {
      error = e.message;
      console.error('Error deleting product:', e);
    }
  }
</script>

<div class="admin-products">
  <h2>Manage Products</h2>
  
  <div class="actions">
    <button on:click={() => {
      editingProduct = null;
      newProduct = { name: '', description: '', price: 0, available: true };
      showForm = !showForm;
    }}>
      {showForm ? 'Cancel' : 'Add New Product'}
    </button>
    <button on:click={fetchProducts}>Refresh</button>
  </div>
  
  {#if error}
    <div class="error">
      Error: {error}
    </div>
  {/if}
  
  {#if showForm}
    <div class="product-form">
      <h3>{editingProduct ? 'Edit Product' : 'Add New Product'}</h3>
      <form on:submit|preventDefault={handleSubmit}>
        <div class="form-group">
          <label for="product-name">Name</label>
          <input id="product-name" bind:value={newProduct.name} required />
        </div>
        
        <div class="form-group">
          <label for="product-description">Description</label>
          <textarea id="product-description" bind:value={newProduct.description} required></textarea>
        </div>
        
        <div class="form-group">
          <label for="product-price">Price</label>
          <input type="number" id="product-price" bind:value={newProduct.price} min="0" step="0.01" required />
        </div>
        
        <div class="form-group checkbox">
          <label>
            <input type="checkbox" bind:checked={newProduct.available} />
            Available
          </label>
        </div>
        
        <button type="submit">{editingProduct ? 'Update Product' : 'Add Product'}</button>
      </form>
    </div>
  {/if}
  
  {#if loading}
    <div class="loading">Loading products...</div>
  {:else}
    <table class="products-table">
      <thead>
        <tr>
          <th>ID</th>
          <th>Name</th>
          <th>Description</th>
          <th>Price</th>
          <th>Available</th>
          <th>Actions</th>
        </tr>
      </thead>
      <tbody>
        {#each products as product (product.id)}
          <tr>
            <td>{product.id}</td>
            <td>{product.name}</td>
            <td class="description">{product.description}</td>
            <td>${product.price.toFixed(2)}</td>
            <td>
              <span class="status-badge status-{product.available ? 'available' : 'unavailable'}">
                {product.available ? 'Yes' : 'No'}
              </span>
            </td>
            <td class="actions">
              <button class="edit-btn" on:click={() => editProduct(product)}>Edit</button>
              <button class="delete-btn" on:click={() => deleteProduct(product.id)}>Delete</button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<style>
  .admin-products {
    padding: 20px;
  }

  .actions {
    display: flex;
    gap: 10px;
    margin-bottom: 20px;
  }

  .products-table {
    width: 100%;
    border-collapse: collapse;
    margin-top: 20px;
  }

  th, td {
    padding: 12px;
    text-align: left;
    border-bottom: 1px solid #ddd;
  }

  th {
    background-color: #f5f5f5;
    font-weight: bold;
  }

  .description {
    max-width: 300px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .status-badge {
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 0.9em;
  }

  .status-available {
    background-color: #d4edda;
    color: #155724;
  }

  .status-unavailable {
    background-color: #f8d7da;
    color: #721c24;
  }

  .actions button {
    padding: 4px 8px;
    margin-right: 5px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  .edit-btn {
    background-color: #3498db;
    color: white;
  }

  .delete-btn {
    background-color: #e74c3c;
    color: white;
  }

  .product-form {
    background-color: #f8f9fa;
    padding: 20px;
    border-radius: 8px;
    margin-bottom: 20px;
  }

  .form-group {
    margin-bottom: 15px;
  }

  .form-group label {
    display: block;
    margin-bottom: 5px;
    font-weight: bold;
  }

  .form-group input, .form-group textarea {
    width: 100%;
    padding: 8px;
    border: 1px solid #ddd;
    border-radius: 4px;
  }

  .form-group textarea {
    height: 100px;
    resize: vertical;
  }

  .checkbox {
    display: flex;
    align-items: center;
  }

  .checkbox input {
    width: auto;
    margin-right: 8px;
  }

  .error {
    color: #721c24;
    background-color: #f8d7da;
    padding: 10px;
    border-radius: 4px;
    margin-bottom: 15px;
  }

  .loading {
    text-align: center;
    padding: 20px;
  }
</style> 