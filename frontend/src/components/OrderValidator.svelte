<script>
  import { onMount } from 'svelte';
  import { auth } from '../stores/auth.js';

  let orders = [];
  let loading = true;
  let error = null;
  let success = null;
  let selectedOrder = null;
  let showDetails = false;
  
  // Define available statuses
  const orderStatuses = ['Pending', 'Processing', 'Shipped', 'Delivered', 'Cancelled'];

  onMount(() => {
    console.log("OrderValidator component mounted");
    fetchOrders();
  });
  
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
      
      console.log("Response status:", response.status);
      
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
      
      console.log('Fetched orders:', orders);
    } catch (e) {
      error = e.message;
      console.error('Error fetching orders:', e);
    } finally {
      loading = false;
    }
  }
  
  // Update order status
  async function updateOrderStatus(orderId, newStatus) {
    try {
      loading = true;
      error = null;
      success = null;
      
      console.log(`Updating order ${orderId} status to ${newStatus}`);
      
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
        throw new Error(data.error || 'Failed to update order status');
      }
      
      // Update the order in the local array
      orders = orders.map(order => 
        order.id === orderId 
          ? { ...order, status: newStatus }
          : order
      );
      
      // If we're viewing this order's details, update the selected order too
      if (selectedOrder && selectedOrder.id === orderId) {
        selectedOrder = { ...selectedOrder, status: newStatus };
      }
      
      success = `Order ${orderId} status updated to ${newStatus}`;
      console.log(success);
      
    } catch (e) {
      error = e.message;
      console.error('Error updating status:', e);
    } finally {
      loading = false;
    }
  }
  
  function viewOrderDetails(order) {
    selectedOrder = order;
    showDetails = true;
  }
  
  function closeOrderDetails() {
    showDetails = false;
    selectedOrder = null;
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

<div class="order-validator">
  <h2>Order Validator</h2>
  
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
  
  {#if showDetails && selectedOrder}
    <!-- Order Details Panel -->
    <div class="order-details">
      <div class="details-header">
        <h3>Order Details: {selectedOrder.id}</h3>
        <button class="close-btn" on:click={closeOrderDetails}>×</button>
      </div>
      
      <div class="details-content">
        <div class="detail-section">
          <h4>Order Information</h4>
          <div class="detail-row">
            <span class="detail-label">Order ID:</span>
            <span class="detail-value">{selectedOrder.id}</span>
          </div>
          <div class="detail-row">
            <span class="detail-label">Created:</span>
            <span class="detail-value">{formatDate(selectedOrder.created_at)}</span>
          </div>
          <div class="detail-row">
            <span class="detail-label">Status:</span>
            <span class="detail-value">
              <select 
                value={selectedOrder.status} 
                on:change={(e) => updateOrderStatus(selectedOrder.id, e.target.value)}
                class={`status-dropdown ${getStatusClass(selectedOrder.status)}`}
              >
                {#each orderStatuses as status}
                  <option value={status}>{status}</option>
                {/each}
              </select>
            </span>
          </div>
          <div class="detail-row">
            <span class="detail-label">Total:</span>
            <span class="detail-value">${selectedOrder.total_amount?.toFixed(2) || '0.00'}</span>
          </div>
        </div>
        
        <div class="detail-section">
          <h4>Customer Information</h4>
          <div class="detail-row">
            <span class="detail-label">Name:</span>
            <span class="detail-value">{selectedOrder.shipping_name || 'Not provided'}</span>
          </div>
          <div class="detail-row">
            <span class="detail-label">Email:</span>
            <span class="detail-value">{selectedOrder.shipping_email || 'Not provided'}</span>
          </div>
          <div class="detail-row">
            <span class="detail-label">Address:</span>
            <span class="detail-value">
              {#if selectedOrder.shipping_address}
                {selectedOrder.shipping_address}, 
                {selectedOrder.shipping_city || ''} 
                {selectedOrder.shipping_state || ''} 
                {selectedOrder.shipping_zip || ''}, 
                {selectedOrder.shipping_country || ''}
              {:else}
                Not provided
              {/if}
            </span>
          </div>
        </div>
        
        <div class="detail-section">
          <h4>Payment Information</h4>
          <div class="detail-row">
            <span class="detail-label">Payment ID:</span>
            <span class="detail-value">{selectedOrder.payment_id || 'N/A'}</span>
          </div>
          {#if selectedOrder.monero_address}
            <div class="detail-row">
              <span class="detail-label">Monero Address:</span>
              <span class="detail-value monero-address">{selectedOrder.monero_address}</span>
            </div>
          {/if}
          <div class="detail-row">
            <span class="detail-label">Payment Status:</span>
            <span class="detail-value status-badge {getStatusClass(selectedOrder.payment_status)}">
              {selectedOrder.payment_status || 'Unknown'}
            </span>
          </div>
        </div>
        
        <div class="actions">
          <button class="back-btn" on:click={closeOrderDetails}>Back to List</button>
        </div>
      </div>
    </div>
  {:else}
    <!-- Orders Table -->
    {#if loading && orders.length === 0}
      <div class="loading">Loading orders...</div>
    {:else if orders.length === 0}
      <div class="no-orders">No orders found</div>
    {:else}
      <table class="orders-table">
        <thead>
          <tr>
            <th>Order ID</th>
            <th>Customer</th>
            <th>Status</th>
            <th>Total</th>
            <th>Created</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {#each orders as order}
            <tr>
              <td class="order-id">{order.id}</td>
              <td>{order.shipping_name || order.user_id || 'Guest'}</td>
              <td>
                <span class="status-badge {getStatusClass(order.status)}">
                  {order.status || 'Unknown'}
                </span>
              </td>
              <td>${order.total_amount?.toFixed(2) || '0.00'}</td>
              <td>{formatDate(order.created_at)}</td>
              <td class="actions-cell">
                <button class="view-btn" on:click={() => viewOrderDetails(order)}>
                  View Details
                </button>
                <select 
                  class="status-select"
                  value={order.status} 
                  on:change={(e) => updateOrderStatus(order.id, e.target.value)}
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
  {/if}
</div>

<style>
  .order-validator {
    padding: 20px;
    max-width: 1200px;
    margin: 0 auto;
  }
  
  h2 {
    margin-bottom: 20px;
    color: #333;
  }
  
  .actions {
    display: flex;
    justify-content: space-between;
    margin-bottom: 20px;
  }
  
  .refresh-btn {
    padding: 8px 16px;
    background-color: #4CAF50;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .refresh-btn:hover {
    background-color: #45a049;
  }
  
  .error-message {
    color: #721c24;
    background-color: #f8d7da;
    border-left: 4px solid #f5c6cb;
    padding: 12px;
    margin-bottom: 20px;
    border-radius: 4px;
  }
  
  .success-message {
    color: #155724;
    background-color: #d4edda;
    border-left: 4px solid #c3e6cb;
    padding: 12px;
    margin-bottom: 20px;
    border-radius: 4px;
  }
  
  .loading, .no-orders {
    padding: 30px;
    text-align: center;
    background-color: #f8f9fa;
    border-radius: 4px;
    color: #6c757d;
  }
  
  .orders-table {
    width: 100%;
    border-collapse: collapse;
    box-shadow: 0 1px 3px rgba(0,0,0,0.1);
  }
  
  th, td {
    padding: 12px 15px;
    text-align: left;
    border-bottom: 1px solid #ddd;
  }
  
  th {
    background-color: #f8f9fa;
    font-weight: bold;
    color: #444;
    position: sticky;
    top: 0;
  }
  
  tr:hover {
    background-color: #f5f5f5;
  }
  
  .order-id {
    font-family: monospace;
    font-size: 0.9em;
    color: #6c757d;
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
  
  .actions-cell {
    display: flex;
    gap: 8px;
  }
  
  .view-btn {
    padding: 6px 10px;
    background-color: #17a2b8;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.85em;
  }
  
  .view-btn:hover {
    background-color: #138496;
  }
  
  .status-select {
    padding: 6px;
    border: 1px solid #ced4da;
    border-radius: 4px;
    background-color: white;
  }
  
  /* Order details styling */
  .order-details {
    background-color: white;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    margin-bottom: 20px;
    overflow: hidden;
  }
  
  .details-header {
    background-color: #f8f9fa;
    padding: 15px 20px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid #dee2e6;
  }
  
  .details-header h3 {
    margin: 0;
    color: #495057;
  }
  
  .close-btn {
    background: none;
    border: none;
    font-size: 24px;
    cursor: pointer;
    color: #6c757d;
  }
  
  .close-btn:hover {
    color: #343a40;
  }
  
  .details-content {
    padding: 20px;
  }
  
  .detail-section {
    margin-bottom: 25px;
  }
  
  .detail-section h4 {
    border-bottom: 1px solid #e9ecef;
    padding-bottom: 8px;
    margin-top: 0;
    margin-bottom: 15px;
    color: #495057;
  }
  
  .detail-row {
    display: flex;
    margin-bottom: 10px;
  }
  
  .detail-label {
    width: 150px;
    font-weight: bold;
    color: #6c757d;
  }
  
  .detail-value {
    flex: 1;
  }
  
  .monero-address {
    font-family: monospace;
    word-break: break-all;
    background-color: #f8f9fa;
    padding: 5px;
    border-radius: 3px;
  }
  
  .back-btn {
    padding: 8px 16px;
    background-color: #6c757d;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    margin-top: 10px;
  }
  
  .back-btn:hover {
    background-color: #5a6268;
  }
  
  .status-dropdown {
    padding: 6px 10px;
    border: 1px solid #ced4da;
    border-radius: 4px;
  }
</style>