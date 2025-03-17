<div class="order-item" class:confirmed={order.status === 'Confirmed' || order.status === 'Completed'}>
  <div class="order-header">
    <h3>Order #{order.id}</h3>
    <p class="order-date">{formatDate(order.created_at)}</p>
  </div>
  
  <div class="order-details">
    <!-- Add this before the Status display to analyze what's happening -->
    {#if order}
      <!-- Debug output - Remove in production -->
      <div class="debug-analyzer">
        <p>Status Analyzer:</p>
        <ul>
          <li>Order ID: {order.id}</li>
          <li>Order Status: {order.status}</li>
          <li>Raw Order Object: <pre>{JSON.stringify(order, null, 2)}</pre></li>
          <li>Is Confirmed: {order.status === 'Confirmed' || order.status === 'Completed'}</li>
          <li>Has Payment Status: {order.payment_status ? 'Yes' : 'No'}</li>
          <li>Payment Status: {order.payment_status || 'N/A'}</li>
          <li>Would Display As: {
            order.status === 'Confirmed' || 
            order.status === 'Completed' || 
            order.payment_status === 'Confirmed' || 
            order.payment_status === 'confirmed' ? 
            'Payment Received' : 'Awaiting Payment'
          }</li>
        </ul>
      </div>
    {/if}

    <p><strong>Status:</strong> 
      {#if order.status === 'Confirmed' || order.status === 'Completed' || 
          order.payment_status === 'Confirmed' || order.payment_status === 'completed' ||
          order.payment_status === 'confirmed'}
        <span class="status confirmed">Payment Received</span>
      {:else if order.status === 'Pending'}
        <span class="status pending">Awaiting Payment</span>
      {:else}
        <span class="status">{order.status}</span>
      {/if}
    </p>
    <p><strong>Total:</strong> ${order.total_amount.toFixed(2)}</p>
    
    <!-- Add Monero address display -->
    {#if order.monero_address}
      <div class="payment-info">
        <p><strong>Monero Address:</strong></p>
        <div class="address-container">
          <code class="monero-address">{order.monero_address}</code>
          <button class="copy-btn" on:click={() => copyToClipboard(order.monero_address)}>
            Copy
          </button>
        </div>
      </div>
    {/if}

    <!-- Add this debug info below -->
    <p class="debug-info">
      <small>Debug Info: Order Status: {order.status}, Payment Status: {order.payment_status || 'N/A'}</small>
    </p>

    <!-- Add this inside the order-details div for each order -->
    {#if order.status === 'Pending'}
      <div class="order-actions">
        <button class="check-btn" on:click={() => checkPaymentStatus(order.payment_id)}>
          Check Payment Status
        </button>
      </div>
    {/if}

    <!-- Add this emergency fix button for each order -->
    {#if order.status === 'Pending' && (order.payment_status === 'Confirmed' || order.payment_status === 'confirmed')}
      <div class="order-actions">
        <button class="emergency-btn" on:click={() => forceFixOrderStatus(order.id)}>
          🛠️ Fix Status Mismatch
        </button>
      </div>
    {/if}
  </div>
</div>

<div class="order-controls">
  <button class="refresh-btn" on:click={refreshOrders}>🔄 Refresh Orders</button>
</div>

<!-- Add this after the refresh button -->
<div class="emergency-controls">
  <button class="refresh-btn" on:click={refreshOrders}>🔄 Refresh Orders</button>
  <button class="emergency-btn" on:click={runDiagnosticFix}>🔧 Fix All Payment Status Issues</button>
</div>

<style>
  /* Add these styles */
  .status {
    display: inline-block;
    padding: 3px 8px;
    border-radius: 4px;
    font-size: 0.9rem;
  }
  
  .status.confirmed {
    background-color: #4CAF50;
    color: white;
  }
  
  .status.pending {
    background-color: #FF9800;
    color: white;
  }
  
  .payment-info {
    margin-top: 1rem;
    background-color: #f8f9fa;
    padding: 1rem;
    border-radius: 4px;
    border-left: 3px solid #6610f2;
  }
  
  .address-container {
    display: flex;
    align-items: center;
    margin-top: 0.5rem;
  }
  
  .monero-address {
    background-color: #eee;
    padding: 0.5rem;
    border-radius: 4px;
    font-family: monospace;
    font-size: 0.9rem;
    word-break: break-all;
    flex: 1;
  }
  
  .copy-btn {
    margin-left: 0.5rem;
    background-color: #6610f2;
    color: white;
    border: none;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .order-item.confirmed {
    border-left: 4px solid #4CAF50;
  }
  
  .order-controls {
    margin-bottom: 1rem;
    display: flex;
    justify-content: flex-end;
  }
  
  .refresh-btn {
    background-color: #2196F3;
    color: white;
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    cursor: pointer;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  
  .refresh-btn:hover {
    background-color: #0b7dda;
  }

  /* Add CSS for emergency button */
  .emergency-btn {
    background-color: #ff5722;
    color: white;
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    cursor: pointer;
    margin-top: 1rem;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }
  
  .emergency-btn:hover {
    background-color: #e64a19;
  }

  /* Add to your existing CSS */
  .emergency-controls {
    display: flex;
    gap: 1rem;
    margin-bottom: 1rem;
    justify-content: flex-end;
  }

  /* Add this */
  .debug-analyzer {
    background-color: #333;
    color: #0f0;
    padding: 1rem;
    font-family: monospace;
    margin-bottom: 1rem;
    border-radius: 4px;
    max-height: 400px;
    overflow: auto;
  }
  
  .debug-analyzer pre {
    white-space: pre-wrap;
    max-height: 200px;
    overflow: auto;
  }
  
  .debug-analyzer ul {
    list-style: none;
    padding: 0;
  }
  
  .debug-analyzer li {
    margin-bottom: 0.5rem;
  }
</style>

<script>
  import { onMount } from 'svelte';
  import { auth } from '../stores/auth.js';
  
  let orders = [];
  let loading = true;
  
  onMount(async () => {
    await fetchOrders();
    // Set up an interval to refresh orders every 10 seconds
    const interval = setInterval(refreshOrders, 10000);
    
    return () => {
      clearInterval(interval);
    };
  });
  
  async function fetchOrders() {
    // Your existing fetchOrders code
  }
  
  async function refreshOrders() {
    console.log("Manual refresh triggered");
    loading = true;
    
    try {
      const response = await fetch("http://localhost:5000/orders/my-orders", {
        headers: {
          "Authorization": `Bearer ${$auth.token}`
        },
        cache: "no-cache" // Add this to prevent caching
      });
      
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      
      const data = await response.json();
      
      if (data.success) {
        console.log("⚡ Orders refreshed:", data.orders);
        orders = data.orders; // Update the orders
      } else {
        console.error("Failed to refresh orders:", data.error);
      }
    } catch (error) {
      console.error("Error refreshing orders:", error);
    } finally {
      loading = false;
    }
  }

  // Add a function to copy to clipboard
  function copyToClipboard(text) {
    navigator.clipboard.writeText(text).then(
      () => {
        alert('Address copied to clipboard');
      },
      (err) => {
        console.error('Could not copy text: ', err);
      }
    );
  }
  
  // Format date function
  function formatDate(timestamp) {
    const date = new Date(timestamp * 1000);
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
  }

  // Add to your script section
  async function checkPaymentStatus(paymentId) {
    if (!paymentId) {
      console.error("No payment ID available for this order");
      return;
    }
    
    try {
      const response = await fetch(`http://localhost:5000/monero/api/monero/check_now/${paymentId}`, {
        method: 'POST',
        headers: {
          "Authorization": `Bearer ${$auth.token}`
        }
      });
      
      const data = await response.json();
      console.log("Payment check result:", data);
      
      if (data.success) {
        // Immediately refresh orders to show updated status
        await refreshOrders();
      }
    } catch (error) {
      console.error("Error checking payment status:", error);
    }
  }

  // Add this to the script section
  async function forceFixOrderStatus(orderId) {
    if (!confirm('This will force the order status to match its payment status. Continue?')) {
      return;
    }
    
    try {
      // First dump the order data to see what's wrong
      const debugResponse = await fetch(`http://localhost:5000/orders/debug/dump-order/${orderId}`, {
        headers: {
          "Authorization": `Bearer ${$auth.token}`
        }
      });
      
      const debugData = await debugResponse.json();
      console.log("DEBUG - Order raw data:", debugData);
      
      // Now force update the status
      const response = await fetch(`http://localhost:5000/orders/admin/force-update-order/${orderId}/Confirmed`, {
        method: 'POST',
        headers: {
          "Authorization": `Bearer ${$auth.token}`
        }
      });
      
      const data = await response.json();
      console.log("Force update result:", data);
      
      // Immediately refresh
      setTimeout(refreshOrders, 500);
      
      alert('Order status has been fixed!');
    } catch (error) {
      console.error("Error fixing order status:", error);
      alert('Failed to fix the order status: ' + error.message);
    }
  }

  // Add this function to the script section
  async function runDiagnosticFix() {
    if (!confirm('This will analyze all orders and payments and fix any mismatched statuses. Continue?')) {
      return;
    }
    
    try {
      console.log("Running global diagnostic fix...");
      const response = await fetch('http://localhost:5000/orders/fix-order-status-mismatch', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${$auth.token}`
        }
      });
      
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      
      const data = await response.json();
      console.log("Diagnostic results:", data);
      
      // Show detailed results
      let message = `Diagnostic complete!\n\n` +
                   `- Found ${data.diagnostics.mismatched_orders} mismatched orders\n` +
                   `- Fixed ${data.diagnostics.fixed_orders} orders\n` +
                   `- Found ${data.diagnostics.missing_orders} payments without orders\n\n` +
                   `See console for full details.`;
                   
      alert(message);
      
      // Refresh the orders list
      setTimeout(refreshOrders, 500);
      
    } catch (error) {
      console.error("Error running diagnostic:", error);
      alert("Error running diagnostic: " + error.message);
    }
  }
</script> 