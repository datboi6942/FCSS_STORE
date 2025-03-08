<script>
  import { onMount } from 'svelte';
  import { auth } from '../stores/auth.js';
  
  let orders = [];
  let loading = true;
  let error = null;
  let success = null;
  let selectedOrder = null;
  let showDetails = false;
  
  const orderStatuses = ['Pending', 'Processing', 'Shipped', 'Delivered', 'Cancelled'];
  
  // Load orders on mount
  onMount(fetchOrders);
  
  async function fetchOrders() {
    try {
      loading = true;
      error = null;
      success = null;
      
      console.log("Fetching orders with token:", $auth.token);
      const response = await fetch('http://localhost:5000/admin/orders', {
        headers: {
          'Authorization': `Bearer ${$auth.token}`
        }
      });
      
      if (!response.ok) {
        const errorText = await response.text();
        console.error("Error response:", errorText);
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      
      const data = await response.json();
      console.log("Orders data received:", data);
      
      // Handle both formats: array or {success: true, orders: [...]}
      if (Array.isArray(data)) {
        orders = data;
      } else if (data.success && Array.isArray(data.orders)) {
        orders = data.orders;
      } else {
        throw new Error('Invalid response format');
      }
      
      // Sort orders by creation date (newest first)
      orders.sort((a, b) => b.created_at - a.created_at);
      
      console.log('Fetched orders:', orders);
    } catch (e) {
      error = e.message;
      console.error('Error fetching orders:', e);
    } finally {
      loading = false;
    }
  }
  
  function viewOrderDetails(order) {
    selectedOrder = order;
    showDetails = true;
  }
  
  async function updateOrderStatus(orderId, newStatus) {
    try {
      loading = true;
      error = null;
      success = null;
      
      const response = await fetch(`http://localhost:5000/admin/orders/${orderId}/status`, {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${$auth.token}`
        },
        body: JSON.stringify({ status: newStatus })
      });
      
      if (!response.ok) {
        const errorText = await response.text();
        console.error("Error response:", errorText);
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      
      const data = await response.json();
      
      if (!data.success) {
        throw new Error(data.error || 'Operation failed');
      }
      
      // Show success message
      success = `Order ${orderId} status updated to ${newStatus}`;
      
      // Update the order status in the local array
      orders = orders.map(order => 
        order.id === orderId 
          ? { ...order, status: newStatus }
          : order
      );
      
      // Update selected order if it's the one we just modified
      if (selectedOrder && selectedOrder.id === orderId) {
        selectedOrder = { ...selectedOrder, status: newStatus };
      }
      
    } catch (e) {
      error = e.message;
      console.error('Error updating order status:', e);
    } finally {
      loading = false;
    }
  }
  
  function formatCurrency(amount) {
    return new Intl.NumberFormat('en-US', {
      style: 'currency',
      currency: 'USD'
    }).format(amount);
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
</script>

<div class="admin-orders">
  <h2>Manage Orders</h2>
  
  <div class="actions">
    <button class="refresh-btn" on:click={fetchOrders}>Refresh Orders</button>
  </div>
  
  {#if error}
    <div class="error-message">
      <strong>Error:</strong> {error}
    </div>
  {/if}
  
  {#if success}
    <div class="success-message">
      <strong>Success:</strong> {success}
    </div>
  {/if}
  
  {#if loading && !orders.length}
    <div class="loading">Loading orders...</div>
  {:else if orders.length === 0}
    <div class="no-data">No orders found.</div>
  {:else}
    <div class="orders-container">
      {#if showDetails && selectedOrder}
        <!-- Order details panel -->
        <div class="order-details-panel">
          <div class="panel-header">
            <h3>Order Details</h3>
            <button class="close-btn" on:click={() => { showDetails = false; selectedOrder = null; }}>×</button>
          </div>
          
          <div class="panel-content">
            <div class="detail-section">
              <h4>Order Information</h4>
              <div class="detail-item">
                <span class="label">Order ID:</span>
                <span class="value">{selectedOrder.id}</span>
              </div>
              <div class="detail-item">
                <span class="label">Created:</span>
                <span class="value">{formatDate(selectedOrder.created_at)}</span>
              </div>
              <div class="detail-item">
                <span class="label">Status:</span>
                <span class="value">
                  <select 
                    value={selectedOrder.status} 
                    on:change={(e) => updateOrderStatus(selectedOrder.id, e.target.value)}
                  >
                    {#each orderStatuses as status}
                      <option value={status}>{status}</option>
                    {/each}
                  </select>
                </span>
              </div>
              <div class="detail-item">
                <span class="label">Total Amount:</span>
                <span class="value">{formatCurrency(selectedOrder.total_amount || 0)}</span>
              </div>
            </div>
            
            <div class="detail-section">
              <h4>Customer Information</h4>
              <div class="detail-item">
                <span class="label">Name:</span>
                <span class="value">{selectedOrder.shipping_name || 'N/A'}</span>
              </div>
              <div class="detail-item">
                <span class="label">Email:</span>
                <span class="value">{selectedOrder.shipping_email || 'N/A'}</span>
              </div>
              {#if selectedOrder.shipping_address}
                <div class="detail-item">
                  <span class="label">Address:</span>
                  <span class="value">
                    {selectedOrder.shipping_address}, 
                    {selectedOrder.shipping_city || ''} 
                    {selectedOrder.shipping_state || ''}, 
                    {selectedOrder.shipping_zip || ''}, 
                    {selectedOrder.shipping_country || ''}
                  </span>
                </div>
              {/if}
            </div>
            
            <div class="detail-section">
              <h4>Payment Information</h4>
              <div class="detail-item">
                <span class="label">Payment ID:</span>
                <span class="value">{selectedOrder.payment_id || 'N/A'}</span>
              </div>
              {#if selectedOrder.monero_address}
                <div class="detail-item">
                  <span class="label">Monero Address:</span>
                  <span class="value monero-address">{selectedOrder.monero_address}</span>
                </div>
              {/if}
              <div class="detail-item">
                <span class="label">Payment Status:</span>
                <span class="value status-badge {getStatusClass(selectedOrder.payment_status)}">
                  {selectedOrder.payment_status || 'Unknown'}
                </span>
              </div>
            </div>
            
            {#if selectedOrder.items && selectedOrder.items.length > 0}
              <div class="detail-section">
                <h4>Order Items</h4>
                <table class="items-table">
                  <thead>
                    <tr>
                      <th>Product</th>
                      <th>Quantity</th>
                      <th>Price</th>
                      <th>Total</th>
                    </tr>
                  </thead>
                  <tbody>
                    {#each selectedOrder.items as item}
                      <tr>
                        <td>{item.name || item.product_id}</td>
                        <td>{item.quantity}</td>
                        <td>{formatCurrency(item.price)}</td>
                        <td>{formatCurrency(item.price * item.quantity)}</td>
                      </tr>
                    {/each}
                  </tbody>
                </table>
              </div>
            {/if}
            
            <div class="detail-actions">
              <button class="back-btn" on:click={() => { showDetails = false; selectedOrder = null; }}>
                Back to Orders
              </button>
            </div>
          </div>
        </div>
      {:else}
        <!-- Orders list table -->
        <table class="orders-table">
          <thead>
            <tr>
              <th>Order ID</th>
              <th>Customer</th>
              <th>Date</th>
              <th>Total</th>
              <th>Status</th>
              <th>Payment</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            {#each orders as order (order.id)}
              <tr class="order-row">
                <td class="order-id">{order.id}</td>
                <td>{order.shipping_name || 'N/A'}</td>
                <td>{formatDate(order.created_at)}</td>
                <td>{formatCurrency(order.total_amount || 0)}</td>
                <td>
                  <span class="status-badge {getStatusClass(order.status)}">
                    {order.status || 'Unknown'}
                  </span>
                </td>
                <td>
                  <span class="status-badge {getStatusClass(order.payment_status)}">
                    {order.payment_status || 'Unknown'}
                  </span>
                </td>
                <td class="actions">
                  <button class="view-btn" on:click={() => viewOrderDetails(order)}>View</button>
                  <select 
                    value={order.status} 
                    on:change={(e) => updateOrderStatus(order.id, e.target.value)}
                    class="status-select"
                  >
                    {#each orderStatuses as status}
                      <option value={status}>{status}</option>
                    {/each}
                  </select>
                </td>
              </tr>
            {/each}
          </tbody>
        </table>
      {/if}
    </div>
  {/if}
</div>

<style>
  .admin-orders {
    padding: 20px;
    max-width: 1200px;
    margin: 0 auto;
  }

  h2 {
    color: #2c3e50;
    margin-bottom: 20px;
    border-bottom: 2px solid #f1f1f1;
    padding-bottom: 10px;
  }

  .actions {
    display: flex;
    gap: 10px;
    margin-bottom: 20px;
  }

  .refresh-btn {
    padding: 10px 15px;
    background-color: #3498db;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-weight: bold;
  }

  .refresh-btn:hover {
    background-color: #2980b9;
  }

  .orders-table {
    width: 100%;
    border-collapse: collapse;
    margin-top: 20px;
    box-shadow: 0 2px 5px rgba(0,0,0,0.1);
  }

  th, td {
    padding: 12px 15px;
    text-align: left;
    border-bottom: 1px solid #ddd;
  }

  th {
    background-color: #f8f9fa;
    font-weight: bold;
    color: #333;
  }

  .order-row:hover {
    background-color: #f5f5f5;
  }

  .order-id {
    font-family: monospace;
    font-size: 0.9em;
    color: #666;
  }

  .status-badge {
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 0.9em;
    font-weight: bold;
    display: inline-block;
  }

  .status-pending {
    background-color: #fcf8e3;
    color: #8a6d3b;
  }

  .status-processing {
    background-color: #d9edf7;
    color: #31708f;
  }

  .status-shipped {
    background-color: #dff0d8;
    color: #3c763d;
  }

  .status-delivered {
    background-color: #d4edda;
    color: #155724;
  }

  .status-cancelled {
    background-color: #f8d7da;
    color: #721c24;
  }

  .status-unknown {
    background-color: #e9ecef;
    color: #495057;
  }

  .actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .view-btn {
    padding: 6px 10px;
    background-color: #17a2b8;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.9em;
  }

  .view-btn:hover {
    background-color: #138496;
  }

  .status-select {
    padding: 6px;
    border: 1px solid #ddd;
    border-radius: 4px;
    background-color: white;
  }

  .error-message {
    background-color: #f8d7da;
    color: #721c24;
    padding: 10px;
    border-radius: 4px;
    margin-bottom: 15px;
    border-left: 4px solid #dc3545;
  }

  .success-message {
    background-color: #d4edda;
    color: #155724;
    padding: 10px;
    border-radius: 4px;
    margin-bottom: 15px;
    border-left: 4px solid #28a745;
  }

  .loading, .no-data {
    text-align: center;
    padding: 30px;
    color: #666;
    background-color: #f8f9fa;
    border-radius: 4px;
    margin-top: 20px;
  }

  /* Order details panel styling */
  .order-details-panel {
    background-color: white;
    border-radius: 8px;
    box-shadow: 0 2px 10px rgba(0,0,0,0.1);
    margin-bottom: 20px;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 15px 20px;
    border-bottom: 1px solid #eee;
  }

  .panel-header h3 {
    margin: 0;
    color: #2c3e50;
  }

  .close-btn {
    background: none;
    border: none;
    font-size: 24px;
    cursor: pointer;
    color: #95a5a6;
  }

  .close-btn:hover {
    color: #e74c3c;
  }

  .panel-content {
    padding: 20px;
  }

  .detail-section {
    margin-bottom: 25px;
    padding-bottom: 15px;
    border-bottom: 1px solid #eee;
  }

  .detail-section h4 {
    margin-top: 0;
    margin-bottom: 15px;
    color: #3498db;
  }

  .detail-item {
    display: flex;
    margin-bottom: 10px;
  }

  .label {
    font-weight: bold;
    width: 120px;
    color: #555;
  }

  .value {
    flex: 1;
  }

  .monero-address {
    font-family: monospace;
    word-break: break-all;
  }

  .items-table {
    width: 100%;
    border-collapse: collapse;
    margin-top: 10px;
  }

  .items-table th, .items-table td {
    padding: 8px 12px;
    text-align: left;
    border-bottom: 1px solid #eee;
  }

  .items-table th {
    background-color: #f8f9fa;
    font-weight: bold;
    color: #333;
  }

  .detail-actions {
    margin-top: 20px;
    display: flex;
    justify-content: flex-end;
  }

  .back-btn {
    padding: 8px 15px;
    background-color: #6c757d;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }

  .back-btn:hover {
    background-color: #5a6268;
  }
</style> 