<script>
  import { onMount } from 'svelte';
  import { apiCall } from '../api.js';
  
  let orders = [];
  let users = [];
  let activeTab = 'orders';
  let isLoading = true;
  
  onMount(async () => {
    await fetchOrders();
    await fetchUsers();
    isLoading = false;
  });
  
  async function fetchOrders() {
    try {
      const response = await apiCall('/admin/orders');
      if (response.ok) {
        orders = await response.json();
      } else {
        console.error('Failed to fetch orders');
      }
    } catch (error) {
      console.error('Error fetching orders:', error);
    }
  }
  
  async function fetchUsers() {
    try {
      const response = await apiCall('/admin/users');
      if (response.ok) {
        users = await response.json();
      } else {
        console.error('Failed to fetch users');
      }
    } catch (error) {
      console.error('Error fetching users:', error);
    }
  }
  
  async function updateOrderStatus(orderId, newStatus) {
    try {
      const response = await apiCall('/admin/order/update-status', {
        method: 'POST',
        body: JSON.stringify({
          order_id: orderId,
          new_status: newStatus
        })
      });
      
      if (response.ok) {
        await fetchOrders();
        alert('Order status updated successfully');
      } else {
        alert('Failed to update order status');
      }
    } catch (error) {
      console.error('Error updating order status:', error);
      alert('Error updating order status');
    }
  }
</script>

<div class="admin-panel">
  <h1>Admin Panel</h1>
  
  <div class="tabs">
    <button 
      class:active={activeTab === 'orders'} 
      on:click={() => activeTab = 'orders'}>
      Orders
    </button>
    <button 
      class:active={activeTab === 'users'} 
      on:click={() => activeTab = 'users'}>
      Users
    </button>
    <button 
      class:active={activeTab === 'products'} 
      on:click={() => activeTab = 'products'}>
      Products
    </button>
  </div>
  
  {#if isLoading}
    <div class="loading">Loading data...</div>
  {:else}
    {#if activeTab === 'orders'}
      <div class="tab-content">
        <h2>Manage Orders</h2>
        {#if orders.length === 0}
          <p>No orders found.</p>
        {:else}
          <table>
            <thead>
              <tr>
                <th>Order ID</th>
                <th>User</th>
                <th>Product</th>
                <th>Status</th>
                <th>Date</th>
                <th>Actions</th>
              </tr>
            </thead>
            <tbody>
              {#each orders as order}
                <tr>
                  <td>{order.id}</td>
                  <td>{order.user_id}</td>
                  <td>{order.product_id}</td>
                  <td>
                    <span class="status-badge status-{order.status.toLowerCase()}">
                      {order.status}
                    </span>
                  </td>
                  <td>{new Date(order.created_at).toLocaleDateString()}</td>
                  <td>
                    <select on:change={(e) => updateOrderStatus(order.id, e.target.value)}>
                      <option value="">Change Status</option>
                      <option value="PENDING">Pending</option>
                      <option value="PROCESSING">Processing</option>
                      <option value="SHIPPED">Shipped</option>
                      <option value="DELIVERED">Delivered</option>
                      <option value="CANCELLED">Cancelled</option>
                    </select>
                  </td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}
      </div>
    {:else if activeTab === 'users'}
      <div class="tab-content">
        <h2>Manage Users</h2>
        {#if users.length === 0}
          <p>No users found.</p>
        {:else}
          <table>
            <thead>
              <tr>
                <th>ID</th>
                <th>Username</th>
                <th>Role</th>
                <th>Created</th>
              </tr>
            </thead>
            <tbody>
              {#each users as user}
                <tr>
                  <td>{user.id}</td>
                  <td>{user.username}</td>
                  <td>{user.role}</td>
                  <td>{new Date(user.created_at).toLocaleDateString()}</td>
                </tr>
              {/each}
            </tbody>
          </table>
        {/if}
      </div>
    {:else if activeTab === 'products'}
      <div class="tab-content">
        <h2>Manage Products</h2>
        <p>Product management interface would go here...</p>
      </div>
    {/if}
  {/if}
</div>

<style>
  .admin-panel {
    max-width: 1000px;
    margin: 0 auto;
  }
  
  .tabs {
    display: flex;
    margin-bottom: 2rem;
    border-bottom: 1px solid #ddd;
  }
  
  .tabs button {
    background: none;
    border: none;
    padding: 0.75rem 1.5rem;
    font-size: 1rem;
    cursor: pointer;
    border-bottom: 3px solid transparent;
  }
  
  .tabs button.active {
    border-bottom: 3px solid #2196F3;
    font-weight: bold;
  }
  
  table {
    width: 100%;
    border-collapse: collapse;
  }
  
  th, td {
    padding: 0.75rem;
    text-align: left;
    border-bottom: 1px solid #ddd;
  }
  
  th {
    background-color: #f5f5f5;
  }
  
  .status-badge {
    display: inline-block;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    font-size: 0.875rem;
  }
  
  .status-pending {
    background-color: #FFC107;
    color: #333;
  }
  
  .status-processing {
    background-color: #2196F3;
    color: white;
  }
  
  .status-shipped {
    background-color: #9C27B0;
    color: white;
  }
  
  .status-delivered {
    background-color: #4CAF50;
    color: white;
  }
  
  .status-cancelled {
    background-color: #F44336;
    color: white;
  }
  
  select {
    padding: 0.5rem;
    border-radius: 4px;
    border: 1px solid #ddd;
  }
  
  .loading {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 200px;
  }
</style> 