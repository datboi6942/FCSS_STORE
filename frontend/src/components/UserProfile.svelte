<script>
  import { onMount } from 'svelte';
  import { auth } from '../stores/auth.js';
  import { navigate } from 'svelte-routing';
  
  let user = null;
  let addresses = [];
  let orders = [];
  let loading = true;
  let error = null;
  let success = null;
  let activeTab = 'profile'; // profile, addresses, orders
  
  // Form states
  let editingAddress = null;
  let newAddress = {
    name: '',
    address: '',
    city: '',
    state: '',
    zip: '',
    country: '',
    isDefault: false
  };
  
  onMount(async () => {
    if (!$auth.isAuthenticated) {
      navigate('/login');
        return;
      }
      
    try {
      await Promise.all([
        fetchUserData(),
        fetchUserAddresses(),
        fetchUserOrders()
      ]);
    } catch (err) {
      console.error('Error loading profile data:', err);
      error = 'Failed to load profile data';
    } finally {
      loading = false;
    }
  });
  
  async function fetchUserData() {
    const response = await fetch('http://localhost:5000/auth/profile', {
      headers: {
        'Authorization': `Bearer ${$auth.token}`
      }
    });
    
    if (!response.ok) {
      throw new Error('Failed to fetch user profile');
    }
    
    const data = await response.json();
    user = {
      id: data.id,
      username: data.username,
      role: data.role,
      createdAt: data.created_at
    };
  }
  
  async function fetchUserAddresses() {
    const response = await fetch('http://localhost:5000/users/addresses', {
      headers: {
        'Authorization': `Bearer ${$auth.token}`
      }
    });
    
    if (!response.ok) {
      // If 404, it just means no addresses yet
      if (response.status === 404) {
        addresses = [];
        return;
      }
      throw new Error('Failed to fetch addresses');
    }
    
    const data = await response.json();
    addresses = data.addresses || [];
  }
  
  async function fetchUserOrders() {
    console.log("Fetching orders for user profile");
    try {
      console.log("Fetching user orders with token:", $auth.token.slice(0, 10) + "...");
      console.log("User ID:", $auth.user?.id);
      
      const response = await fetch("http://localhost:5000/orders/my-orders", {
        headers: {
          "Authorization": `Bearer ${$auth.token}`
        }
      });
      
      console.log("Profile orders response status:", response.status);
      
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      
      const data = await response.json();
      console.log("Profile orders data:", data);
      
      if (data.success) {
        console.log(`Fetched ${data.count} orders for profile:`, data.orders);
        orders = data.orders;
      } else {
        console.error("Error fetching orders:", data.error);
      }
    } catch (error) {
      console.error("Error fetching user orders:", error);
    }
  }
  
  async function saveAddress() {
    try {
      const isEdit = !!editingAddress;
      const addressData = isEdit ? editingAddress : newAddress;
      
      // Log the data being sent
      console.log(`Saving address (${isEdit ? 'edit' : 'new'}):`, addressData);
      
      const url = isEdit 
        ? `http://localhost:5000/users/addresses/${editingAddress.id}` 
        : 'http://localhost:5000/users/addresses';
      
      const response = await fetch(url, {
        method: isEdit ? 'PUT' : 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${$auth.token}`
        },
        body: JSON.stringify(addressData)
      });
      
      const responseData = await response.json();
      console.log('Address save response:', responseData);
      
      if (!response.ok) {
        throw new Error(responseData.error || 'Failed to save address');
      }
      
      // Refresh addresses
      await fetchUserAddresses();
      
      // Reset form
      editingAddress = null;
      newAddress = {
        name: '',
        address: '',
        city: '',
        state: '',
        zip: '',
        country: '',
        isDefault: false
      };
      
      // Show success message
      error = null;
      success = 'Address saved successfully';
      
      // Hide success message after 3 seconds
      setTimeout(() => {
        success = null;
      }, 3000);
    } catch (err) {
      console.error('Error saving address:', err);
      error = err.message || 'Failed to save address';
    }
  }
  
  async function deleteAddress(id) {
    if (!confirm('Are you sure you want to delete this address?')) {
      return;
    }
    
    try {
      const response = await fetch(`http://localhost:5000/users/addresses/${id}`, {
        method: 'DELETE',
        headers: {
          'Authorization': `Bearer ${$auth.token}`
        }
      });
      
      if (!response.ok) {
        throw new Error('Failed to delete address');
      }
      
      // Refresh addresses
      await fetchUserAddresses();
    } catch (err) {
      console.error('Error deleting address:', err);
      error = 'Failed to delete address';
    }
  }
  
  function startEditingAddress(address) {
    editingAddress = { ...address };
  }
  
  function cancelEditingAddress() {
    editingAddress = null;
  }
  
  function formatDate(timestamp) {
    const date = new Date(timestamp * 1000);
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
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
  
  // Add this function to redirect to order status page
  function viewOrderStatus(orderId) {
    navigate(`/orders?id=${orderId}`);
  }
  
  // Add the copy function if it doesn't exist already
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
  
  async function refreshOrders() {
    console.log("Manual refresh triggered in UserProfile");
    
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
        console.log("⚡ Orders refreshed in profile:", data.orders);
        orders = data.orders; // Update the orders
      } else {
        console.error("Failed to refresh orders in profile:", data.error);
      }
    } catch (error) {
      console.error("Error refreshing orders in profile:", error);
    }
  }
  
  async function checkPaymentStatus(paymentId) {
    if (!paymentId) {
      console.error("No payment ID available for this order");
      
      // Try to run the diagnostic fix instead
      await runDiagnosticFix();
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
  
  // Set up auto-refresh when component mounts
  onMount(() => {
    // Existing code...
    
    // Set up an interval to refresh orders every 10 seconds
    const orderRefreshInterval = setInterval(refreshOrders, 10000);
    
    return () => {
      // Clean up intervals when component is destroyed
      clearInterval(orderRefreshInterval);
      // Other cleanup...
    };
  });
  
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
  
  // Add this function to your script section
  async function dumpDebugInfo() {
    try {
      console.log("============ DEBUG INFO ============");
      console.log("Current orders in UserProfile:", orders);
      
      // Try to get more detailed info from backend
      if (orders.length > 0) {
        const firstOrderId = orders[0].id;
        console.log(`Fetching detailed info for order: ${firstOrderId}`);
        
        const response = await fetch(`http://localhost:5000/orders/debug/dump-order/${firstOrderId}`, {
          headers: {
            "Authorization": `Bearer ${$auth.token}`
          }
        });
        
        const data = await response.json();
        console.log("Raw order data from backend:", data);
        
        // Check payment links
        if (data.payment_id) {
          console.log(`Order has payment_id: ${data.payment_id}`);
        } else {
          console.log("⚠️ Order has NO payment_id!");
        }
        
        if (orders[0].payment_id) {
          console.log(`Frontend knows payment_id: ${orders[0].payment_id}`);
        } else {
          console.log("⚠️ Frontend does NOT have payment_id!");
          
          // Attempt to fix by querying monero_payments
          console.log("Attempting to find a matching payment for this order...");
          const fixResponse = await fetch(`http://localhost:5000/orders/fix-order-status-mismatch`, {
            method: 'POST',
            headers: {
              "Authorization": `Bearer ${$auth.token}`
            }
          });
          
          const fixData = await fixResponse.json();
          console.log("Fix attempt result:", fixData);
        }
      }
      console.log("===================================");
    } catch (error) {
      console.error("Error dumping debug info:", error);
    }
  }
  
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
  
  // Add this function to your script
  async function fixOrphanedPayments() {
    try {
      console.log("Fixing orphaned payments...");
      const response = await fetch('http://localhost:5000/monero/fix-orphaned-payments', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${$auth.token}`
        }
      });
      
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      
      const data = await response.json();
      console.log("Fix orphaned payments result:", data);
      
      if (data.fixed_count > 0) {
        alert(`Fixed ${data.fixed_count} payment link(s)! Your orders should now show the correct status.`);
      } else {
        alert("No orphaned payments found to fix.");
      }
      
      // Refresh orders to see the changes
      setTimeout(refreshOrders, 500);
      
    } catch (error) {
      console.error("Error fixing orphaned payments:", error);
      alert("Error fixing payment links: " + error.message);
    }
  }
  
  // Add this function to your debug section
  async function directlyFixMyOrders() {
    try {
      if (orders.length === 0) {
        alert("No orders to fix");
        return;
      }
      
      console.log("Attempting direct order fixes");
      
      // First, get all payment data for debugging
      const paymentResponse = await fetch('http://localhost:5000/monero/debug/dump-all-payments', {
        headers: {
          'Authorization': `Bearer ${$auth.token}`
        }
      });
      
      const paymentData = await paymentResponse.json();
      console.log("All payment records:", paymentData);
      
      // Fix the access to match the API response structure
      const dbPayments = paymentData.db_payments || [];
      const memoryPayments = paymentData.memory_payments || [];
      
      let fixedCount = 0;
      
      // For each order, try to find a matching payment
      for (const order of orders) {
        console.log(`Checking order ${order.id}`);
        
        // Try matching from both sources of payments
        const matchingDbPayment = dbPayments.find(p => 
          p.payment_id === order.id || 
          p.order_id === order.id
        );
        
        const matchingMemoryPayment = memoryPayments.find(p => 
          p.payment_id === order.id || 
          p.order_id === order.id
        );
        
        const matchingPayment = matchingDbPayment || matchingMemoryPayment;
        
        if (matchingPayment) {
          console.log(`Found matching payment for order ${order.id}:`, matchingPayment);
          
          // Update the order status directly
          const updateResponse = await fetch(`http://localhost:5000/orders/admin/force-update-order/${order.id}/Confirmed`, {
            method: 'POST',
            headers: {
              'Authorization': `Bearer ${$auth.token}`
            }
          });
          
          const updateResult = await updateResponse.json();
          console.log(`Order ${order.id} update result:`, updateResult);
          
          if (updateResult.success) {
            fixedCount++;
          }
        } else {
          console.log(`No matching payment found for order ${order.id}`);
        }
      }
      
      if (fixedCount > 0) {
        alert(`Fixed ${fixedCount} orders directly!`);
        setTimeout(refreshOrders, 500);
      } else {
        alert("Couldn't find any payments to match with your orders.");
      }
    } catch (error) {
      console.error("Error performing direct fix:", error);
      alert("Error: " + error.message);
    }
  }
  
  // Add this function to your debug section
  async function forceCreatePaymentLinks() {
    try {
      console.log("Force creating payment links...");
      const response = await fetch('http://localhost:5000/monero/admin/force-create-payment-links', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${$auth.token}`
        }
      });
      
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      
      const data = await response.json();
      console.log("Force create payment links result:", data);
      
      if (data.fixed_count > 0) {
        alert(`Created ${data.fixed_count} payment link(s)! Your orders should now have payment IDs.`);
      } else {
        alert("No orders without payment links found.");
      }
      
      // Refresh orders to see the changes
      setTimeout(refreshOrders, 500);
      
    } catch (error) {
      console.error("Error creating payment links:", error);
      alert("Error creating payment links: " + error.message);
    }
  }
  
  // Add this function to your script section
  async function forceConfirmOrder(orderId) {
    try {
      console.log(`Force confirming order ${orderId}`);
      const response = await fetch(`http://localhost:5000/monero/force-update-order-status/${orderId}`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${$auth.token}`
        }
      });
      
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }
      
      const data = await response.json();
      console.log("Force confirm result:", data);
      
      if (data.success) {
        alert("Order status has been manually confirmed!");
        setTimeout(refreshOrders, 500);
      } else {
        alert("Failed to update order status: " + (data.error || "Unknown error"));
      }
    } catch (error) {
      console.error("Error forcing order confirmation:", error);
      alert("Error: " + error.message);
    }
  }
</script>

<div class="profile-container">
  <h1>My Account</h1>
  
  {#if error}
    <div class="error-message">
      {error}
    </div>
  {/if}
  
  {#if success}
    <div class="success-message">
      {success}
    </div>
  {/if}
  
  {#if loading}
    <div class="loading">
      Loading your profile...
    </div>
  {:else}
    <div class="profile-tabs">
      <button 
        class:active={activeTab === 'profile'} 
        on:click={() => activeTab = 'profile'}
      >
        Profile
      </button>
      <button 
        class:active={activeTab === 'addresses'} 
        on:click={() => activeTab = 'addresses'}
      >
        Addresses
      </button>
      <button 
        class:active={activeTab === 'orders'} 
        on:click={() => activeTab = 'orders'}
      >
        Orders
      </button>
    </div>
    
    <div class="tab-content">
      <!-- Profile Tab -->
      {#if activeTab === 'profile'}
        <div class="profile-info">
    <div class="profile-card">
            <h2>Account Information</h2>
            {#if user}
              <div class="info-row">
                <span class="label">Username:</span>
                <span class="value">{user.username}</span>
              </div>
              <div class="info-row">
                <span class="label">User ID:</span>
                <span class="value">{user.id}</span>
              </div>
              <div class="info-row">
                <span class="label">Role:</span>
                <span class="value">{user.role}</span>
              </div>
              <div class="info-row">
                <span class="label">Member Since:</span>
                <span class="value">{formatDate(user.createdAt)}</span>
              </div>
            {/if}
          </div>
          
          <div class="profile-actions">
            <button class="logout-btn" on:click={() => auth.logout()}>
              Logout
            </button>
          </div>
        </div>
      {/if}
      
      <!-- Addresses Tab -->
      {#if activeTab === 'addresses'}
        <div class="addresses-section">
          <h2>My Addresses</h2>
          
          {#if editingAddress}
            <div class="address-form">
              <h3>Edit Address</h3>
              
              <div class="form-group">
                <label for="edit-name">Full Name</label>
                <input type="text" id="edit-name" bind:value={editingAddress.name} required>
              </div>
              
              <div class="form-group">
                <label for="edit-address">Street Address</label>
                <input type="text" id="edit-address" bind:value={editingAddress.address} required>
              </div>
              
              <div class="form-row">
                <div class="form-group">
                  <label for="edit-city">City</label>
                  <input type="text" id="edit-city" bind:value={editingAddress.city} required>
                </div>
                
                <div class="form-group">
                  <label for="edit-state">State/Province</label>
                  <input type="text" id="edit-state" bind:value={editingAddress.state} required>
                </div>
              </div>
              
              <div class="form-row">
                <div class="form-group">
                  <label for="edit-zip">ZIP/Postal Code</label>
                  <input type="text" id="edit-zip" bind:value={editingAddress.zip} required>
                </div>
                
                <div class="form-group">
                  <label for="edit-country">Country</label>
                  <input type="text" id="edit-country" bind:value={editingAddress.country} required>
                </div>
              </div>
              
              <div class="form-group checkbox">
                <input type="checkbox" id="edit-default" bind:checked={editingAddress.isDefault}>
                <label for="edit-default">Make this my default address</label>
              </div>
              
              <div class="form-actions">
                <button class="save-btn" on:click={saveAddress}>Save Address</button>
                <button class="cancel-btn" on:click={cancelEditingAddress}>Cancel</button>
              </div>
            </div>
          {:else}
            <div class="address-form">
              <h3>Add New Address</h3>
              
              <div class="form-group">
                <label for="name">Full Name</label>
                <input type="text" id="name" bind:value={newAddress.name} required>
              </div>
              
              <div class="form-group">
                <label for="address">Street Address</label>
                <input type="text" id="address" bind:value={newAddress.address} required>
              </div>
              
              <div class="form-row">
                <div class="form-group">
                  <label for="city">City</label>
                  <input type="text" id="city" bind:value={newAddress.city} required>
                </div>
                
                <div class="form-group">
                  <label for="state">State/Province</label>
                  <input type="text" id="state" bind:value={newAddress.state} required>
                </div>
              </div>
              
              <div class="form-row">
                <div class="form-group">
                  <label for="zip">ZIP/Postal Code</label>
                  <input type="text" id="zip" bind:value={newAddress.zip} required>
                </div>
                
                <div class="form-group">
                  <label for="country">Country</label>
                  <input type="text" id="country" bind:value={newAddress.country} required>
                </div>
              </div>
              
              <div class="form-group checkbox">
                <input type="checkbox" id="default" bind:checked={newAddress.isDefault}>
                <label for="default">Make this my default address</label>
              </div>
              
              <div class="form-actions">
                <button class="save-btn" on:click={saveAddress}>Add Address</button>
              </div>
            </div>
          {/if}
          
          <div class="address-list">
            {#if addresses.length === 0}
              <p class="no-addresses">You haven't added any addresses yet.</p>
            {:else}
              {#each addresses as address}
                <div class="address-card">
                  {#if address.isDefault}
                    <span class="default-badge">Default</span>
                  {/if}
                  
                  <div class="address-details">
                    <p class="address-name">{address.name}</p>
                    <p>{address.address}</p>
                    <p>{address.city}, {address.state} {address.zip}</p>
                    <p>{address.country}</p>
      </div>
      
                  <div class="address-actions">
                    <button class="edit-btn" on:click={() => startEditingAddress(address)}>
                      Edit
                    </button>
                    <button class="delete-btn" on:click={() => deleteAddress(address.id)}>
                      Delete
                    </button>
                  </div>
                </div>
              {/each}
            {/if}
          </div>
        </div>
      {/if}
      
      <!-- Orders Tab -->
      {#if activeTab === 'orders'}
        <div class="orders-section">
          <h3>Your Orders</h3>
          <div class="order-controls">
            <button class="refresh-btn" on:click={refreshOrders}>🔄 Refresh Orders</button>
        </div>
        
          {#if orders.length === 0}
            <p class="no-orders">You don't have any orders yet.</p>
          {:else}
            <!-- Loop through orders -->
            {#each orders as order}
              <div class="order-item" class:confirmed={order.status === 'Confirmed' || order.status === 'Completed' || 
                order.payment_status === 'Confirmed' || order.payment_status === 'completed' ||
                order.payment_status === 'confirmed'}>
                <div class="order-header">
                  <h4>Order #{order.id}</h4>
                  <span class="order-date">{formatDate(order.created_at)}</span>
                </div>
                <div class="order-details">
                  <p>
                    <strong>Status:</strong> 
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
                  
                  <!-- Add debug info -->
                  <p class="debug-info">
                    <small>Debug Info: Order Status: {order.status}, Payment Status: {order.payment_status || 'N/A'}</small>
                  </p>
                  
                  <!-- Add check payment button for pending orders -->
                  {#if order.status === 'Pending'}
                    <div class="order-actions">
                      <button class="check-btn" on:click={() => checkPaymentStatus(order.payment_id)}>
                        Check Payment Status
        </button>
                    </div>
                  {/if}
                  
                  <!-- Add emergency fix button -->
                  {#if order.status === 'Pending' && (order.payment_status === 'Confirmed' || order.payment_status === 'confirmed')}
                    <div class="order-actions">
                      <button class="emergency-btn" on:click={() => forceFixOrderStatus(order.id)}>
                        🛠️ Fix Status Mismatch
        </button>
      </div>
                  {/if}
                  
                  <!-- Add new button to force confirm order -->
                  {#if order.status === 'Pending'}
                    <div class="order-actions">
                      <button class="emergency-btn" on:click={() => forceConfirmOrder(order.id)}>
                        🛠️ Force Confirm Order
                      </button>
                    </div>
                  {/if}
                </div>
              </div>
            {/each}
          {/if}
        </div>
      {/if}
    </div>
  {/if}
</div>

<!-- Add the debug section here, outside the script tag -->
<div class="debug-section">
  <button class="debug-btn" on:click={dumpDebugInfo}>Show Order Debug Info</button>
  <div class="emergency-controls">
    <button class="emergency-btn" on:click={fixOrphanedPayments}>🔄 Fix Missing Payment Links</button>
  </div>
  <button class="emergency-btn" on:click={directlyFixMyOrders}>🛠️ Direct Order Status Fix</button>
  <button class="emergency-btn" on:click={forceCreatePaymentLinks}>🔄 Force Create Payment Links</button>
</div>

<style>
  .profile-container {
    max-width: 1000px;
    margin: 0 auto;
    padding: 20px;
  }
  
  h1 {
    margin-bottom: 30px;
    color: #333;
  }
  
  .error-message {
    background-color: #f8d7da;
    color: #721c24;
    padding: 12px;
    border-radius: 4px;
    margin-bottom: 20px;
  }
  
  .success-message {
    background-color: #d4edda;
    color: #155724;
    padding: 12px;
    border-radius: 4px;
    margin-bottom: 20px;
  }
  
  .loading {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 200px;
    font-size: 1.2rem;
    color: #666;
  }
  
  .profile-tabs {
    display: flex;
    border-bottom: 1px solid #ddd;
    margin-bottom: 30px;
  }
  
  .profile-tabs button {
    background: none;
    border: none;
    padding: 12px 20px;
    cursor: pointer;
    font-size: 16px;
    color: #666;
    border-bottom: 3px solid transparent;
    transition: all 0.2s;
  }
  
  .profile-tabs button.active {
    color: #2196F3;
    border-bottom-color: #2196F3;
  }
  
  .profile-info {
    display: flex;
    flex-direction: column;
    gap: 30px;
  }
  
  .profile-card {
    background-color: white;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    padding: 20px;
  }
  
  .info-row {
    display: flex;
    margin-bottom: 12px;
    padding-bottom: 12px;
    border-bottom: 1px solid #eee;
  }
  
  .info-row:last-child {
    border-bottom: none;
    padding-bottom: 0;
  }
  
  .label {
    width: 150px;
    font-weight: 500;
    color: #666;
  }
  
  .value {
    flex: 1;
    color: #333;
  }
  
  .profile-actions {
    display: flex;
    justify-content: flex-end;
  }
  
  .logout-btn {
    background-color: #f44336;
    color: white;
    border: none;
    padding: 10px 20px;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .addresses-section, .orders-section {
    margin-bottom: 30px;
  }
  
  .address-form {
    background-color: white;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    padding: 20px;
    margin-bottom: 30px;
  }
  
  .form-group {
    margin-bottom: 15px;
  }
  
  .form-row {
    display: flex;
    gap: 15px;
  }
  
  .form-row .form-group {
    flex: 1;
  }
  
  label {
    display: block;
    margin-bottom: 8px;
    font-weight: 500;
  }
  
  input[type="text"] {
    width: 100%;
    padding: 10px;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 14px;
  }
  
  .checkbox {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  
  .checkbox input {
    width: auto;
  }
  
  .checkbox label {
    margin-bottom: 0;
  }
  
  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 20px;
  }
  
  .save-btn {
    background-color: #4CAF50;
    color: white;
    border: none;
    padding: 10px 20px;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .cancel-btn {
    background-color: #f44336;
    color: white;
    border: none;
    padding: 10px 20px;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .address-list {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 20px;
  }
  
  .address-card {
    background-color: white;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    padding: 20px;
    position: relative;
  }
  
  .default-badge {
    position: absolute;
    top: 10px;
    right: 10px;
    background-color: #2196F3;
    color: white;
    padding: 4px 8px;
    font-size: 12px;
    border-radius: 4px;
  }
  
  .address-name {
    font-weight: bold;
    margin-bottom: 8px;
  }
  
  .address-details p {
    margin: 5px 0;
    color: #666;
  }
  
  .address-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 15px;
  }
  
  .edit-btn {
    background-color: #2196F3;
    color: white;
    border: none;
    padding: 8px 12px;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .delete-btn {
    background-color: #f44336;
    color: white;
    border: none;
    padding: 8px 12px;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .no-addresses, .no-orders {
    text-align: center;
    padding: 30px;
    background-color: white;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    color: #666;
  }
  
  .orders-section {
    margin-bottom: 30px;
  }
  
  .order-item {
    background-color: white;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    padding: 20px;
    margin-bottom: 20px;
  }
  
  .order-header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 15px;
    padding-bottom: 15px;
    border-bottom: 1px solid #eee;
  }
  
  .order-id {
    font-weight: bold;
    color: #333;
  }
  
  .order-date {
    color: #666;
  }
  
  .order-details {
    margin-top: 1rem;
  }
  
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
  
  .debug-info {
    margin-top: 1rem;
    font-size: 0.8rem;
    color: #999;
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
  
  .check-btn {
    background-color: #FF9800;
    color: white;
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    cursor: pointer;
    margin-top: 1rem;
  }
  
  .check-btn:hover {
    background-color: #e68a00;
  }
  
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
  
  .debug-section {
    margin-top: 2rem;
    border-top: 1px dashed #ccc;
    padding-top: 1rem;
  }
  
  .debug-btn {
    background-color: #333;
    color: #00ff00;
    border: 1px solid #00ff00;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .debug-btn:hover {
    background-color: #444;
  }
</style> 