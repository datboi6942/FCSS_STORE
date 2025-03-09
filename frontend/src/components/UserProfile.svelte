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
    try {
      console.log("Fetching orders for user profile");
      if (!$auth || !$auth.token) {
        console.error("No authentication token available");
        orders = [];
        return;
      }
      
      console.log("Fetching user orders with token:", $auth.token.substring(0, 10) + "...");
      console.log("User ID:", $auth.user ? $auth.user.id : "Unknown");
      
      const response = await fetch('http://localhost:5000/orders/my-orders', {
        headers: {
          'Authorization': `Bearer ${$auth.token}`
        }
      });
      
      console.log("Profile orders response status:", response.status);
      
      if (!response.ok) {
        // If 404, it just means no orders yet
        if (response.status === 404) {
          console.log("No orders found for profile (404)");
          orders = [];
          return;
        }
        const text = await response.text();
        console.error("Error response:", text);
        throw new Error(`Failed to fetch orders: ${response.status} ${text}`);
      }
      
      const contentType = response.headers.get("content-type");
      if (!contentType || !contentType.includes("application/json")) {
        console.error("Response is not JSON:", await response.text());
        throw new Error("Invalid response format");
      }
      
      const data = await response.json();
      console.log("Profile orders data:", data);
      
      orders = data.success && data.orders ? data.orders : [];
      
      console.log(`Fetched ${orders.length} orders for profile:`, orders);
    } catch (err) {
      console.error('Error fetching orders for profile:', err);
      orders = [];
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
    if (!timestamp) return 'N/A';
    const date = new Date(timestamp * 1000);
    return date.toLocaleString();
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
          <h2>My Orders</h2>
          
          {#if orders.length === 0}
            <div class="no-orders">
              <p>You don't have any orders yet.</p>
            </div>
          {:else}
            <div class="orders-list">
              {#each orders as order}
                <div class="order-card">
                  <div class="order-header">
                    <span class="order-id">Order #{order.id}</span>
                    <span class="order-date">{formatDate(order.created_at)}</span>
                  </div>
                  
                  <div class="order-details">
                    <div class="detail-row">
                      <span class="detail-label">Status:</span>
                      <span class="detail-value status-badge {getStatusClass(order.status)}">
                        {order.status}
                      </span>
        </div>
        
        <div class="detail-row">
                      <span class="detail-label">Total:</span>
                      <span class="detail-value">${order.total_amount ? order.total_amount.toFixed(2) : '0.00'}</span>
        </div>
        
        <div class="detail-row">
                      <span class="detail-label">Payment Method:</span>
                      <span class="detail-value">{order.payment_id ? 'Monero (XMR)' : 'Unknown'}</span>
        </div>
      </div>
      
                  <div class="order-actions">
                    <button class="view-order-btn" on:click={() => viewOrderStatus(order.id)}>
                      View Status
        </button>
      </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      {/if}
    </div>
  {/if}
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
  
  .orders-list {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }
  
  .order-card {
    background-color: white;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
    padding: 20px;
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
  
  .detail-row {
    display: flex;
    margin-bottom: 8px;
  }
  
  .detail-label {
    width: 150px;
    font-weight: 500;
    color: #666;
  }
  
  .detail-value {
    flex: 1;
  }
  
  .status-badge {
    display: inline-block;
    padding: 5px 10px;
    border-radius: 20px;
    font-size: 0.9em;
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
  
  .order-actions {
    margin-top: 20px;
    display: flex;
    justify-content: flex-end;
  }
  
  .view-order-btn {
    display: inline-block;
    background-color: #2196F3;
    color: white;
    padding: 8px 15px;
    text-decoration: none;
    border-radius: 4px;
  }
</style> 