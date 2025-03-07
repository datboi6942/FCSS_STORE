<script>
  import { onMount } from 'svelte';
  
  let orderId = '';
  let order = null;
  let loading = false;
  let error = null;
  
  async function checkOrderStatus() {
    if (!orderId) {
      error = "Please enter an order ID";
      return;
    }
    
    loading = true;
    error = null;
    
    try {
      const response = await fetch(`http://localhost:5000/orders/status/${orderId}`);
      if (!response.ok) {
        throw new Error(response.status === 404 ? "Order not found" : "Failed to fetch order");
      }
      
      order = await response.json();
    } catch (err) {
      error = err.message;
      order = null;
    } finally {
      loading = false;
    }
  }
</script>

<div class="order-status">
  <h2>Check Order Status</h2>
  
  <div class="search-form">
    <input 
      type="text" 
      bind:value={orderId} 
      placeholder="Enter your order ID"
    />
    <button on:click={checkOrderStatus} disabled={loading}>
      {loading ? 'Checking...' : 'Check Status'}
    </button>
  </div>
  
  {#if error}
    <div class="error">
      {error}
    </div>
  {/if}
  
  {#if order}
    <div class="order-details">
      <h3>Order Details</h3>
      <div class="status">
        Status: <span class="status-badge {order.status.toLowerCase()}">{order.status}</span>
      </div>
      
      <div class="shipping-info">
        <h4>Shipping Information</h4>
        <p>{order.shipping_info.name}</p>
        <p>{order.shipping_info.address}</p>
        <p>{order.shipping_info.city}, {order.shipping_info.state} {order.shipping_info.zip}</p>
        <p>{order.shipping_info.country}</p>
      </div>
      
      <div class="order-info">
        <p>Order ID: {order.id}</p>
        <p>Total Amount: ${order.total_amount.toFixed(2)}</p>
        <p>Created: {new Date(order.created_at * 1000).toLocaleString()}</p>
        <p>Last Updated: {new Date(order.updated_at * 1000).toLocaleString()}</p>
      </div>
    </div>
  {/if}
</div>

<style>
  .order-status {
    max-width: 800px;
    margin: 2rem auto;
    padding: 2rem;
    background: white;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  }
  
  .search-form {
    display: flex;
    gap: 1rem;
    margin-bottom: 2rem;
  }
  
  input {
    flex: 1;
    padding: 0.75rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 1rem;
  }
  
  button {
    padding: 0.75rem 1.5rem;
    background: #4CAF50;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }
  
  button:disabled {
    background: #ccc;
  }
  
  .error {
    color: #e53935;
    margin-bottom: 1rem;
  }
  
  .status-badge {
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    font-weight: bold;
  }
  
  .pending { background: #FFF3E0; color: #E65100; }
  .awaitingpayment { background: #E3F2FD; color: #1565C0; }
  .paid { background: #E8F5E9; color: #2E7D32; }
  .shipped { background: #F3E5F5; color: #7B1FA2; }
  .delivered { background: #E8EAF6; color: #3949AB; }
  .completed { background: #E0F2F1; color: #00695C; }
  .cancelled { background: #FFEBEE; color: #C62828; }
</style> 