<script>
  import { createEventDispatcher } from 'svelte';
  import { cartItems, cartTotal } from '../stores/cart.js';
  
  const dispatch = createEventDispatcher();
  
  let shippingInfo = {
    name: '',
    email: '',
    address: '',
    city: '',
    state: '',
    zip: '',
    country: ''
  };
  
  function handleSubmit() {
    // Validate form
    if (!shippingInfo.name || !shippingInfo.email || !shippingInfo.address || 
        !shippingInfo.city || !shippingInfo.state || !shippingInfo.zip || 
        !shippingInfo.country) {
      alert('Please fill in all fields');
      return;
    }
    
    // Prepare checkout data
    const checkoutData = {
      items: $cartItems,
      shipping_info: shippingInfo,
      total: $cartTotal
    };
    
    dispatch('submit', checkoutData);
  }
  
  function handleCancel() {
    dispatch('close');
  }
</script>

<div class="shipping-overlay">
  <div class="shipping-content">
    <h2>Enter Shipping Information</h2>
    
    <form on:submit|preventDefault={handleSubmit}>
      <div class="form-group">
        <label for="name">Full Name</label>
        <input type="text" id="name" bind:value={shippingInfo.name} required>
      </div>
      
      <div class="form-group">
        <label for="email">Email</label>
        <input type="email" id="email" bind:value={shippingInfo.email} required>
      </div>
      
      <div class="form-group">
        <label for="address">Address</label>
        <input type="text" id="address" bind:value={shippingInfo.address} required>
      </div>
      
      <div class="form-group">
        <label for="city">City</label>
        <input type="text" id="city" bind:value={shippingInfo.city} required>
      </div>
      
      <div class="form-group">
        <label for="state">State/Province</label>
        <input type="text" id="state" bind:value={shippingInfo.state} required>
      </div>
      
      <div class="form-group">
        <label for="zip">Zip/Postal Code</label>
        <input type="text" id="zip" bind:value={shippingInfo.zip} required>
      </div>
      
      <div class="form-group">
        <label for="country">Country</label>
        <input type="text" id="country" bind:value={shippingInfo.country} required>
      </div>
      
      <div class="form-actions">
        <button type="button" class="btn-secondary" on:click={handleCancel}>
          Cancel
        </button>
        <button type="submit" class="btn-primary">
          Continue to Payment
        </button>
      </div>
    </form>
  </div>
</div>

<style>
  .shipping-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.75);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 9999;
  }
  
  .shipping-content {
    background-color: white;
    padding: 25px;
    border-radius: 8px;
    max-width: 550px;
    width: 90%;
    box-shadow: 0 5px 20px rgba(0,0,0,0.3);
  }
  
  h2 {
    margin-bottom: 20px;
    color: #333;
    text-align: center;
  }
  
  .form-group {
    margin-bottom: 15px;
  }
  
  label {
    display: block;
    margin-bottom: 5px;
    font-weight: bold;
    color: #555;
  }
  
  input {
    width: 100%;
    padding: 10px;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 16px;
  }
  
  .form-actions {
    display: flex;
    justify-content: space-between;
    margin-top: 20px;
  }
  
  .btn-primary {
    background-color: #2ecc71;
    color: white;
    border: none;
    padding: 12px 20px;
    border-radius: 4px;
    cursor: pointer;
    font-weight: bold;
  }
  
  .btn-secondary {
    background-color: #95a5a6;
    color: white;
    border: none;
    padding: 12px 20px;
    border-radius: 4px;
    cursor: pointer;
    font-weight: bold;
  }
  
  .btn-primary:hover {
    background-color: #27ae60;
  }
  
  .btn-secondary:hover {
    background-color: #7f8c8d;
  }
</style> 