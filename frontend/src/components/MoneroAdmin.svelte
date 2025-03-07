<script>
  import { onMount, onDestroy } from 'svelte';
  import { auth } from '../stores/auth.js';
  
  let transactions = [];
  let loading = true;
  let error = null;
  let stats = {
    pending: 0,
    confirmed: 0,
    completed: 0,
    expired: 0,
    total: 0,
    volume: 0
  };
  let refreshInterval;
  
  onMount(() => {
    loadTransactions();
    // Auto-refresh every 30 seconds
    refreshInterval = setInterval(loadTransactions, 30000);
    return () => clearInterval(refreshInterval);
  });
  
  onDestroy(() => {
    if (refreshInterval) clearInterval(refreshInterval);
  });
  
  async function loadTransactions() {
    if (!$auth.isAdmin) return;
    
    loading = true;
    error = null;
    
    try {
      // PLACEHOLDER: In production, call your actual admin API
      const response = await fetch('/api/monero/admin/transactions', {
        headers: {
          'Authorization': `Bearer ${$auth.token}`
        }
      });
      
      if (!response.ok) {
        throw new Error('Failed to load transactions');
      }
      
      const data = await response.json();
      transactions = data.transactions || [];
      
      // Calculate stats
      stats = {
        pending: transactions.filter(tx => tx.status === 'Pending').length,
        confirmed: transactions.filter(tx => tx.status === 'Confirmed').length,
        completed: transactions.filter(tx => tx.status === 'Completed').length,
        expired: transactions.filter(tx => tx.status === 'Expired').length,
        total: transactions.length,
        volume: transactions
          .filter(tx => tx.status === 'Completed' || tx.status === 'Confirmed')
          .reduce((sum, tx) => sum + tx.amount, 0)
      };
    } catch (err) {
      console.error('Error loading transactions:', err);
      error = err.message;
    } finally {
      loading = false;
    }
  }
  
  async function manuallyConfirmPayment(paymentId) {
    if (!confirm('Are you sure you want to manually confirm this payment?')) return;
    
    try {
      const response = await fetch(`/api/monero/admin/confirm/${paymentId}`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${$auth.token}`,
          'Content-Type': 'application/json'
        }
      });
      
      if (!response.ok) {
        throw new Error('Failed to confirm payment');
      }
      
      // Refresh the list
      await loadTransactions();
    } catch (err) {
      console.error('Error confirming payment:', err);
      alert(`Error: ${err.message}`);
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

<div class="admin-page">
  <h1>Monero Payment Administration</h1>
  
  <div class="stats-cards">
    <div class="stat-card">
      <div class="stat-value">{stats.pending}</div>
      <div class="stat-label">Pending</div>
    </div>
    <div class="stat-card">
      <div class="stat-value">{stats.confirmed}</div>
      <div class="stat-label">Confirmed</div>
    </div>
    <div class="stat-card">
      <div class="stat-value">{stats.completed}</div>
      <div class="stat-label">Completed</div>
    </div>
    <div class="stat-card">
      <div class="stat-value">{stats.expired}</div>
      <div class="stat-label">Expired</div>
    </div>
    <div class="stat-card">
      <div class="stat-value">{stats.volume.toFixed(8)}</div>
      <div class="stat-label">XMR Volume</div>
    </div>
  </div>
  
  <div class="action-bar">
    <button on:click={loadTransactions} class="refresh-btn">
      Refresh Data
    </button>
    
    <!-- PLACEHOLDER: Add more admin actions here -->
  </div>
  
  {#if loading && transactions.length === 0}
    <div class="loading">Loading transactions...</div>
  {:else if error}
    <div class="error-message">
      <p>{error}</p>
      <button on:click={loadTransactions}>Try Again</button>
    </div>
  {:else if transactions.length === 0}
    <div class="empty-state">
      <p>No Monero transactions found.</p>
    </div>
  {:else}
    <div class="transactions-list">
      <table>
        <thead>
          <tr>
            <th>Payment ID</th>
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
              <td>{tx.payment_id}</td>
              <td>{tx.order_id}</td>
              <td>{formatDate(tx.created_at)}</td>
              <td>{tx.amount.toFixed(8)}</td>
              <td>
                <span class={`status-badge ${getStatusBadgeClass(tx.status)}`}>
                  {tx.status}
                </span>
              </td>
              <td class="actions-cell">
                {#if tx.status === 'Pending'}
                  <button 
                    on:click={() => manuallyConfirmPayment(tx.payment_id)}
                    class="action-btn confirm-btn"
                  >
                    Confirm
                  </button>
                {/if}
                <button class="action-btn details-btn">
                  Details
                </button>
              </td>
            </tr>
          {/each}
        </tbody>
      </table>
    </div>
  {/if}
  
  <!-- PLACEHOLDER: Transaction details modal would go here -->
</div>

<style>
  .admin-page {
    max-width: 1200px;
    margin: 0 auto;
    padding: 2rem;
  }
  
  h1 {
    margin-bottom: 2rem;
    border-bottom: 1px solid #eee;
    padding-bottom: 1rem;
  }
  
  .stats-cards {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 1rem;
    margin-bottom: 2rem;
  }
  
  .stat-card {
    background-color: white;
    border-radius: 8px;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
    padding: 1.5rem;
    text-align: center;
  }
  
  .stat-value {
    font-size: 2rem;
    font-weight: bold;
    margin-bottom: 0.5rem;
  }
  
  .stat-label {
    color: #666;
    font-size: 1rem;
  }
  
  .action-bar {
    display: flex;
    justify-content: space-between;
    margin-bottom: 2rem;
  }
  
  .refresh-btn {
    background-color: #4caf50;
    color: white;
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
  
  .actions-cell {
    display: flex;
    gap: 0.5rem;
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
    border: none;
    border-radius: 4px;
    padding: 0.5rem 0.75rem;
    cursor: pointer;
    font-size: 0.85rem;
  }
  
  .confirm-btn {
    background-color: #4caf50;
    color: white;
  }
  
  .details-btn {
    background-color: #f0f0f0;
    color: #333;
  }
</style> 