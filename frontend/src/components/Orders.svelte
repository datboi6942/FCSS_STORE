<script>
  import { onMount } from 'svelte';
  import { auth } from '../stores/auth.js';
  import { navigate } from 'svelte-routing';
  
  let orders = [];
  let userOrders = [];
  let lookupOrderId = '';
  let lookupOrder = null;
  let lookupError = null;
  let loading = false;
  let submitClicked = false;
  let testOrderId = null;
  let debugData = null;
  
  onMount(async () => {
    if ($auth.isAuthenticated) {
      await fetchUserOrders();
    }
  });
  
  async function fetchUserOrders() {
    try {
      loading = true;
      const response = await fetch('http://localhost:5000/orders/my-orders', {
        headers: {
          'Authorization': `Bearer ${$auth.token}`
        }
      });
      
      if (!response.ok) {
        throw new Error('Failed to fetch your orders');
      }
      
      const data = await response.json();
      userOrders = data.success ? data.orders : [];
    } catch (error) {
      console.error('Error fetching orders:', error);
    } finally {
      loading = false;
    }
  }
  
  async function performOrderLookup() {
    submitClicked = true;
    lookupOrder = null;
    lookupError = null;
    
    if (!lookupOrderId) {
      lookupError = 'Please enter an order ID';
      return;
    }
    
    try {
      loading = true;
      const response = await fetch(`http://localhost:5000/orders/status/${lookupOrderId}`, {
        method: 'GET',
        headers: {
          'Accept': 'application/json'
        }
      });
      
      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.error || 'Order not found');
      }
      
      const data = await response.json();
      
      if (!data.success) {
        throw new Error(data.error || 'Failed to look up order');
      }
      
      lookupOrder = data.order;
      console.log('Order found:', lookupOrder);
    } catch (error) {
      lookupError = error.message;
      console.error('Order lookup error:', error);
    } finally {
      loading = false;
    }
  }
  
  function formatDate(timestamp) {
    if (!timestamp) return 'N/A';
    return new Date(timestamp * 1000).toLocaleString();
  }
  
  function getStatusClass(status) {
    if (!status) return 'status-unknown';
    
    switch (status.toLowerCase()) {
      case 'pending': return 'status-pending';
      case 'processing': return 'status-processing';
      case 'shipped': return 'status-shipped';
      case 'delivered': return 'status-delivered';
      case 'cancelled': return 'status-cancelled';
      default: return 'status-unknown';
    }
  }
  
  function getStatusStep(status) {
    if (!status) return 0;
    
    switch (status.toLowerCase()) {
      case 'pending': return 1;
      case 'processing': return 2;
      case 'shipped': return 3;
      case 'delivered': return 4;
      case 'cancelled': return -1; // Special case
      default: return 0;
    }
  }
  
  async function createTestOrder() {
    try {
      const response = await fetch('http://localhost:5000/orders/create-test', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${$auth.token}`
        }
      });
      
      if (!response.ok) {
        throw new Error('Failed to create test order');
      }
      
      const data = await response.json();
      testOrderId = data.id;
      console.log('Test order created:', testOrderId);
    } catch (error) {
      console.error('Error creating test order:', error);
    }
  }
  
  async function debugOrders() {
    try {
      const response = await fetch('http://localhost:5000/orders/debug-orders', {
        headers: {
          'Authorization': `Bearer ${$auth.token}`
        }
      });
      
      if (!response.ok) {
        throw new Error('Failed to debug orders');
      }
      
      debugData = await response.json();
      console.log('Debug data:', debugData);
    } catch (error) {
      console.error('Error during debug:', error);
    }
  }
</script>

<div class="orders-container">
  <h1>Orders</h1>
  
  <div class="test-order-section">
    <button class="create-test-btn" on:click={createTestOrder}>Create Test Order</button>
    {#if testOrderId}
      <div class="success-message">
        <p>Test order created! Order ID: <strong>{testOrderId}</strong></p>
      </div>
    {/if}
  </div>
  
  <div class="debug-section">
    <button class="debug-btn" on:click={debugOrders}>Debug Orders Database</button>
    {#if debugData}
      <div class="debug-info">
        <h4>Debug Information</h4>
        <p>Found {debugData.count} orders in database</p>
        {#if debugData.orders && debugData.orders.length > 0}
          <div class="debug-orders">
            {#each debugData.orders as order}
              <div class="debug-order">
                <p><strong>ID:</strong> {order.id}</p>
                <p><strong>Status:</strong> {order.status}</p>
                <p><strong>Created:</strong> {formatDate(order.created_at)}</p>
                
                <button class="lookup-btn" on:click={() => {
                  lookupOrderId = order.id;
                  performOrderLookup();
                }}>Look Up This Order</button>
              </div>
            {/each}
          </div>
        {:else}
          <p>No orders found in database!</p>
        {/if}
      </div>
    {/if}
  </div>
  
  <div class="order-lookup-section">
    <h2>Track Your Order</h2>
    <p>Enter your order ID to check the status of your order. <strong>No login required!</strong></p>
    
    <div class="lookup-form">
      <input 
        type="text" 
        bind:value={lookupOrderId} 
        placeholder="Enter your order ID"
        class:error={submitClicked && !lookupOrderId}
      />
      <button on:click={performOrderLookup} disabled={loading}>
        {loading ? 'Looking up...' : 'Track Order'}
      </button>
    </div>
    
    {#if lookupError}
      <div class="error-message">
        <strong>Error:</strong> {lookupError}
      </div>
    {/if}
    
    {#if lookupOrder}
      <div class="order-details-card">
        <div class="order-header">
          <h3>Order #{lookupOrder.id}</h3>
          <span class="order-date">Placed on: {formatDate(lookupOrder.created_at)}</span>
        </div>
        
        <div class="order-status-tracker">
          <div class="progress-bar">
            <div class="progress" style="width: {getStatusStep(lookupOrder.status) * 25}%"></div>
          </div>
          
          <div class="status-steps">
            <div class="step {getStatusStep(lookupOrder.status) >= 1 ? 'active' : ''} {lookupOrder.status === 'Cancelled' ? 'cancelled' : ''}">
              <div class="step-icon">1</div>
              <div class="step-label">Order Received</div>
            </div>
            <div class="step {getStatusStep(lookupOrder.status) >= 2 ? 'active' : ''} {lookupOrder.status === 'Cancelled' ? 'cancelled' : ''}">
              <div class="step-icon">2</div>
              <div class="step-label">Processing</div>
            </div>
            <div class="step {getStatusStep(lookupOrder.status) >= 3 ? 'active' : ''} {lookupOrder.status === 'Cancelled' ? 'cancelled' : ''}">
              <div class="step-icon">3</div>
              <div class="step-label">Shipped</div>
            </div>
            <div class="step {getStatusStep(lookupOrder.status) >= 4 ? 'active' : ''} {lookupOrder.status === 'Cancelled' ? 'cancelled' : ''}">
              <div class="step-icon">4</div>
              <div class="step-label">Delivered</div>
            </div>
          </div>
        </div>
        
        {#if lookupOrder.status === 'Cancelled'}
          <div class="cancelled-notice">
            This order has been cancelled.
          </div>
        {/if}
        
        <div class="order-summary">
          <div class="summary-row">
            <span class="label">Status:</span>
            <span class="value status-badge {getStatusClass(lookupOrder.status)}">
              {lookupOrder.status || 'Unknown'}
            </span>
          </div>
          <div class="summary-row">
            <span class="label">Total Amount:</span>
            <span class="value">${lookupOrder.total_amount?.toFixed(2) || '0.00'}</span>
          </div>
          <div class="summary-row">
            <span class="label">Payment Method:</span>
            <span class="value">{lookupOrder.payment_id ? 'Monero (XMR)' : 'Unknown'}</span>
          </div>
        </div>
        
        {#if lookupOrder.shipping_name}
          <div class="shipping-info">
            <h4>Shipping Information</h4>
            <p><strong>Name:</strong> {lookupOrder.shipping_name}</p>
            <p><strong>Address:</strong> {lookupOrder.shipping_address}, {lookupOrder.shipping_city} {lookupOrder.shipping_state} {lookupOrder.shipping_zip}, {lookupOrder.shipping_country}</p>
          </div>
        {/if}
      </div>
    {/if}
  </div>
  
  {#if $auth.isAuthenticated}
    <div class="user-orders-section">
      <h2>Your Orders</h2>
      
      {#if loading}
        <div class="loading">Loading your orders...</div>
      {:else if userOrders.length === 0}
        <div class="no-orders">
          <p>You haven't placed any orders yet.</p>
          <a href="/products" class="shop-now-btn">Shop Now</a>
        </div>
      {:else}
        <div class="orders-list">
          {#each userOrders as order}
            <div class="order-card">
              <div class="order-header">
                <h3>Order #{order.id}</h3>
                <span class="order-date">{formatDate(order.created_at)}</span>
              </div>
              
              <div class="order-info">
                <div class="status-info">
                  <span class="status-badge {getStatusClass(order.status)}">
                    {order.status}
                  </span>
                  <span class="total-amount">${order.total_amount.toFixed(2)}</span>
                </div>
                
                <button class="view-details-btn" on:click={() => navigate(`/order/${order.id}`)}>
                  View Details
                </button>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .orders-container {
    max-width: 1000px;
    margin: 0 auto;
    padding: 20px;
  }
  
  h1, h2, h3, h4 {
    color: #333;
  }
  
  h1 {
    margin-bottom: 30px;
    font-size: 2rem;
  }
  
  h2 {
    margin-bottom: 20px;
    font-size: 1.5rem;
    border-bottom: 1px solid #eee;
    padding-bottom: 10px;
  }
  
  .order-lookup-section {
    background-color: white;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    padding: 25px;
    margin-bottom: 30px;
  }
  
  .lookup-form {
    display: flex;
    gap: 10px;
    margin-bottom: 20px;
  }
  
  input {
    flex: 1;
    padding: 12px;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 16px;
  }
  
  input.error {
    border-color: #dc3545;
  }
  
  button {
    background-color: #4CAF50;
    color: white;
    border: none;
    padding: 12px 20px;
    border-radius: 4px;
    cursor: pointer;
    font-weight: bold;
  }
  
  button:hover {
    background-color: #45a049;
  }
  
  button:disabled {
    background-color: #cccccc;
    cursor: not-allowed;
  }
  
  .error-message {
    color: #721c24;
    background-color: #f8d7da;
    border-left: 4px solid #f5c6cb;
    padding: 12px;
    margin-bottom: 20px;
    border-radius: 4px;
  }
  
  .order-details-card {
    background-color: #f8f9fa;
    border-radius: 8px;
    padding: 20px;
    margin-top: 20px;
  }
  
  .order-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
  }
  
  .order-header h3 {
    margin: 0;
    color: #333;
  }
  
  .order-date {
    color: #6c757d;
    font-size: 0.9em;
  }
  
  .order-status-tracker {
    margin-bottom: 30px;
  }
  
  .progress-bar {
    height: 8px;
    background-color: #e9ecef;
    border-radius: 4px;
    margin-bottom: 15px;
    position: relative;
    z-index: 1;
  }
  
  .progress {
    height: 100%;
    background-color: #28a745;
    border-radius: 4px;
    transition: width 0.3s ease;
  }
  
  .status-steps {
    display: flex;
    justify-content: space-between;
  }
  
  .step {
    text-align: center;
    flex: 1;
    position: relative;
    color: #6c757d;
  }
  
  .step.active {
    color: #28a745;
  }
  
  .step.cancelled {
    color: #dc3545;
  }
  
  .step-icon {
    width: 30px;
    height: 30px;
    border-radius: 50%;
    background-color: #e9ecef;
    display: flex;
    align-items: center;
    justify-content: center;
    margin: 0 auto 8px;
    font-weight: bold;
    border: 2px solid #6c757d;
  }
  
  .step.active .step-icon {
    background-color: #28a745;
    color: white;
    border-color: #28a745;
  }
  
  .step.cancelled .step-icon {
    background-color: #dc3545;
    color: white;
    border-color: #dc3545;
  }
  
  .step-label {
    font-size: 0.85em;
    font-weight: 500;
  }
  
  .cancelled-notice {
    background-color: #f8d7da;
    color: #721c24;
    padding: 10px;
    border-radius: 4px;
    margin-bottom: 20px;
    text-align: center;
    font-weight: bold;
  }
  
  .order-summary {
    background-color: white;
    border-radius: 4px;
    padding: 15px;
    margin-bottom: 20px;
  }
  
  .summary-row {
    display: flex;
    justify-content: space-between;
    padding: 8px 0;
    border-bottom: 1px solid #eee;
  }
  
  .summary-row:last-child {
    border-bottom: none;
  }
  
  .label {
    font-weight: bold;
    color: #495057;
  }
  
  .value {
    color: #212529;
  }
  
  .status-badge {
    display: inline-block;
    padding: 3px 10px;
    border-radius: 20px;
    font-size: 0.85em;
    font-weight: 500;
    text-align: center;
  }
  
  .status-pending {
    background-color: #fff3cd;
    color: #856404;
  }
  
  .status-processing {
    background-color: #d1ecf1;
    color: #0c5460;
  }
  
  .status-shipped {
    background-color: #d4edda;
    color: #155724;
  }
  
  .status-delivered {
    background-color: #cce5ff;
    color: #004085;
  }
  
  .status-cancelled {
    background-color: #f8d7da;
    color: #721c24;
  }
  
  .status-unknown {
    background-color: #e9ecef;
    color: #495057;
  }
  
  .shipping-info {
    background-color: white;
    border-radius: 4px;
    padding: 15px;
  }
  
  .shipping-info h4 {
    margin-top: 0;
    margin-bottom: 10px;
    color: #495057;
  }
  
  .shipping-info p {
    margin: 5px 0;
    color: #6c757d;
  }
  
  .user-orders-section {
    margin-top: 40px;
  }
  
  .no-orders {
    background-color: white;
    border-radius: 8px;
    padding: 30px;
    text-align: center;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
  }
  
  .shop-now-btn {
    display: inline-block;
    background-color: #007bff;
    color: white;
    padding: 10px 20px;
    border-radius: 4px;
    text-decoration: none;
    margin-top: 15px;
    font-weight: 500;
  }
  
  .orders-list {
    display: grid;
    gap: 20px;
  }
  
  .order-card {
    background-color: white;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    padding: 15px;
  }
  
  .order-info {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: 15px;
  }
  
  .status-info {
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
  
  .total-amount {
    font-weight: bold;
    color: #212529;
  }
  
  .view-details-btn {
    background-color: #6c757d;
    color: white;
    border: none;
    padding: 8px 15px;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .view-details-btn:hover {
    background-color: #5a6268;
  }
  
  .loading {
    padding: 30px;
    text-align: center;
    background-color: white;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    color: #6c757d;
  }
  
  .create-test-btn {
    margin-bottom: 20px;
    background-color: #007bff;
    color: white;
    border: none;
    padding: 10px 15px;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .create-test-btn:hover {
    background-color: #0069d9;
  }
  
  .test-order-section {
    margin-bottom: 20px;
  }
  
  .debug-section {
    margin-bottom: 30px;
    padding: 15px;
    background-color: #f8f9fa;
    border: 1px dashed #dee2e6;
    border-radius: 8px;
  }
  
  .debug-btn {
    background-color: #6c757d;
    color: white;
    border: none;
    padding: 8px 15px;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .debug-info {
    margin-top: 15px;
  }
  
  .debug-orders {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 15px;
    margin-top: 15px;
  }
  
  .debug-order {
    background-color: white;
    padding: 15px;
    border-radius: 8px;
    box-shadow: 0 1px 3px rgba(0,0,0,0.1);
  }
  
  .lookup-btn {
    background-color: #28a745;
    color: white;
    border: none;
    padding: 8px 15px;
    border-radius: 4px;
    cursor: pointer;
    margin-top: 10px;
    width: 100%;
  }
</style>