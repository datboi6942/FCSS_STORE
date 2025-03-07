<script>
  import { onMount } from 'svelte';
  
  let orders = [];
  let loading = true;
  let error = null;
  
  onMount(async () => {
    await loadOrders();
  });
  
  async function loadOrders() {
    try {
      const response = await fetch('http://localhost:5000/admin/orders');
      if (!response.ok) throw new Error('Failed to fetch orders');
      
      orders = await response.json();
    } catch (err) {
      error = err.message;
    } finally {
      loading = false;
    }
  }
  
  async function updateOrderStatus(orderId, newStatus) {
    try {
      const response = await fetch(`http://localhost:5000/orders/admin/update/${orderId}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ status: newStatus })
      });
      
      if (!response.ok) throw new Error('Failed to update order status');
      
      await loadOrders();
    } catch (err) {
      alert(`Error updating order: ${err.message}`);
    }
  }
</script>

<div class="admin-orders">
  <h2>Manage Orders</h2>
  
  {#if loading}
    <div class="loading">Loading orders...</div>
  {:else if error}
    <div class="error">{error}</div>
  {:else}
    <table>
      <thead>
        <tr>
          <th>Order ID</th>
          <th>Customer</th>
          <th>Amount</th>
          <th>Status</th>
          <th>Created</th>
          <th>Actions</th>
        </tr>
      </thead>
      <tbody>
        {#each orders as order}
          <tr>
            <td>{order.id}</td>
            <td>{order.shipping_info.name}</td>
            <td>${order.total_amount.toFixed(2)}</td>
            <td>
              <span class="status-badge {order.status.toLowerCase()}">
                {order.status}
              </span>
            </td>
            <td>{new Date(order.created_at * 1000).toLocaleString()}</td>
            <td>
              <select 
                value={order.status}
                on:change={(e) => updateOrderStatus(order.id, e.target.value)}
              >
                <option value="Pending">Pending</option>
                <option value="AwaitingPayment">Awaiting Payment</option>
                <option value="Paid">Paid</option>
                <option value="Shipped">Shipped</option>
                <option value="Delivered">Delivered</option>
                <option value="Completed">Completed</option>
                <option value="Cancelled">Cancelled</option>
              </select>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<style>
  .admin-orders {
    padding: 2rem;
  }
  
  table {
    width: 100%;
    border-collapse: collapse;
    margin-top: 2rem;
  }
  
  th, td {
    padding: 1rem;
    text-align: left;
    border-bottom: 1px solid #eee;
  }
  
  th {
    background: #f5f5f5;
  }
  
  select {
    padding: 0.5rem;
    border: 1px solid #ddd;
    border-radius: 4px;
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