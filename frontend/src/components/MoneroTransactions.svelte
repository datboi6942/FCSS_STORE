<script>
  import { onMount } from 'svelte';
  import { auth } from '../stores/auth.js';
  
  let transactions = [];
  let loading = true;
  let error = null;
  
  onMount(async () => {
    await loadTransactions();
  });
  
  async function loadTransactions() {
    loading = true;
    error = null;
    
    try {
      const response = await fetch('/api/monero/user_transactions', {
        headers: {
          'Authorization': `Bearer ${$auth.token}`
        }
      });
      
      if (!response.ok) {
        throw new Error('Failed to load transactions');
      }
      
      const data = await response.json();
      transactions = data.transactions || [];
    } catch (err) {
      console.error('Error loading transactions:', err);
      error = err.message;
    } finally {
      loading = false;
    }
  }
  
  function formatDate(timestamp) {
    return new Date(timestamp * 1000).toLocaleString();
  }
  
  function getStatusBadgeClass(status) {
    switch(status) {
      case 'Completed': return 'status-completed';
      case 'Confirmed': return 'status-confirmed';
      case 'Pending': return 'status-pending';
      case 'Expired': return 'status-expired';
      default: return 'status-unknown';
    }
  }
</script>

<div class="transactions-page">
  <h2>Monero Payment History</h2>
  
  {#if loading}
    <div class="loading">Loading transactions...</div>
  {:else if error}
    <div class="error-message">
      <p>{error}</p>
      <button on:click={loadTransactions}>Try Again</button>
    </div>
  {:else if transactions.length === 0}
    <div class="empty-state">
      <p>You don't have any Monero transactions yet.</p>
    </div>
  {:else}
    <div class="transactions-list">
      <table>
        <thead>
          <tr>
            <th>Order ID</th>
            <th>Date</th>
            <th>Amount (XMR)</th>
            <th>Status</th>
            <th>Actions</th>
          </tr>
        </thead>
        <tbody>
          {#each transactions as tx}
            <tr>
              <td>{tx.order_id}</td>
              <td>{formatDate(tx.created_at)}</td>
              <td>{tx.amount.toFixed(8)}</td>
              <td>
                <span class={`status-badge ${getStatusBadgeClass(tx.status)}`}>
                  {tx.status}
                </span>
              </td>
              <td>
                {#if tx.status === 'Pending'}
                  <a href={`/payment/${tx.payment_id}`} class="action-btn">
                    Resume Payment
                  </a>
                {:else if tx.status === 'Completed' || tx.status === 'Confirmed'}
                  <a href={`/orders/${tx.order_id}`} class="action-btn">
                    View Order
                  </a>
                {/if}
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
</div>

<style>
  .transactions-page {
    max-width: 1000px;
    margin: 0 auto;
    padding: 2rem;
  }
  
  h2 {
    margin-bottom: 2rem;
    border-bottom: 1px solid #eee;
    padding-bottom: 1rem;
  }
  
  .loading, .empty-state {
    text-align: center;
    padding: 3rem;
    color: #666;
  }
  
  .error-message {
    background-color: #ffebee;
    padding: 1.5rem;
    border-radius: 4px;
    margin-bottom: 2rem;
    border-left: 4px solid #f44336;
  }
  
  table {
    width: 100%;
    border-collapse: collapse;
  }
  
  th, td {
    padding: 1rem;
    text-align: left;
    border-bottom: 1px solid #eee;
  }
  
  th {
    background-color: #f5f5f5;
    font-weight: bold;
  }
  
  .status-badge {
    display: inline-block;
    padding: 0.25rem 0.75rem;
    border-radius: 20px;
    font-size: 0.8rem;
    font-weight: bold;
  }
  
  .status-completed {
    background-color: #e8f5e9;
    color: #2e7d32;
  }
  
  .status-confirmed {
    background-color: #e3f2fd;
    color: #1565c0;
  }
  
  .status-pending {
    background-color: #fff8e1;
    color: #f57f17;
  }
  
  .status-expired {
    background-color: #f5f5f5;
    color: #757575;
  }
  
  .action-btn {
    display: inline-block;
    padding: 0.5rem 0.75rem;
    background-color: #f0f0f0;
    color: #333;
    text-decoration: none;
    border-radius: 4px;
    font-size: 0.85rem;
  }
  
  .action-btn:hover {
    background-color: #e0e0e0;
  }
</style> 