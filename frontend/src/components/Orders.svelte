<script>
  import { onMount } from 'svelte';
  let userId = '';
  let orders = [];
  let loading = false;
  let error = null;

  async function fetchOrders() {
    if (!userId.trim()) {
      error = "Please enter a User ID";
      return;
    }
    
    try {
      loading = true;
      error = null;
      const res = await fetch(`http://127.0.0.1:8443/order/history?user_id=${userId}`);
      
      if (!res.ok) {
        throw new Error('Failed to fetch orders');
      }
      
      orders = await res.json();
    } catch (err) {
      error = err.message;
      console.error(err);
    } finally {
      loading = false;
    }
  }
</script>

<div class="orders-container">
  <h2>Order History</h2>
  
  <div class="search-container">
    <p>Enter your User ID to view your order history:</p>
    
    <div class="search-form">
      <input 
        bind:value={userId} 
        placeholder="Enter your User ID" 
        disabled={loading}
      />
      <button on:click={fetchOrders} disabled={loading}>
        {loading ? 'Loading...' : 'Fetch Orders'}
      </button>
    </div>
    
    {#if error}
      <p class="error">{error}</p>
    {/if}
  </div>

  {#if orders.length > 0}
    <div class="orders-list">
      <h3>Your Orders</h3>
      <table>
        <thead>
          <tr>
            <th>Order ID</th>
            <th>Product</th>
            <th>Status</th>
            <th>Date</th>
          </tr>
        </thead>
        <tbody>
          {#each orders as order}
            <tr>
              <td>{order.id}</td>
              <td>{order.product}</td>
              <td>
                <span class="status-badge status-{order.status.toLowerCase()}">{order.status}</span>
              </td>
              <td>{new Date(order.created_at).toLocaleDateString()}</td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {:else if !error && !loading && userId}
    <p class="no-orders">No orders found for this User ID.</p>
  {/if}
</div>

<style>
  .orders-container {
    max-width: 800px;
    margin: 0 auto;
  }

  h2 {
    margin-bottom: 1.5rem;
  }

  .search-container {
    background: #f5f5f5;
    padding: 1.5rem;
    border-radius: 8px;
    margin-bottom: 2rem;
  }

  .search-form {
    display: flex;
    gap: 0.5rem;
  }

  input {
    flex: 1;
    padding: 0.5rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 1rem;
  }

  button {
    background: #1e90ff;
    color: white;
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    cursor: pointer;
    font-size: 1rem;
  }

  button:hover {
    background: #167edb;
  }

  button:disabled {
    background: #b3d1ff;
    cursor: not-allowed;
  }

  .error {
    color: #e74c3c;
    background: #fadbd8;
    padding: 0.5rem;
    border-radius: 4px;
    margin-top: 1rem;
  }

  .orders-list {
    background: white;
    padding: 1.5rem;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
  }

  table {
    width: 100%;
    border-collapse: collapse;
  }

  th, td {
    padding: 0.75rem;
    text-align: left;
    border-bottom: 1px solid #eee;
  }

  th {
    font-weight: 600;
    color: #555;
  }

  .status-badge {
    display: inline-block;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    font-size: 0.85rem;
  }

  .status-pending {
    background: #fff3cd;
    color: #856404;
  }

  .status-completed {
    background: #d4edda;
    color: #155724;
  }

  .status-cancelled {
    background: #f8d7da;
    color: #721c24;
  }

  .no-orders {
    text-align: center;
    color: #777;
    margin-top: 2rem;
  }
</style>