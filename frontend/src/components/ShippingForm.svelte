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
    
    console.log('Submitting shipping info:', checkoutData); // Debug log
    dispatch('submit', checkoutData);
  }
  
  function handleCancel() {
    dispatch('close');
  }
</script>

<div class="overlay">
  <div class="shipping-form">
    <h2>Shipping Details</h2>
    
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
      
      <div class="form-row">
        <div class="form-group">
          <label for="city">City</label>
          <input type="text" id="city" bind:value={shippingInfo.city} required>
        </div>
        
        <div class="form-group">
          <label for="state">State/Province</label>
          <input type="text" id="state" bind:value={shippingInfo.state} required>
        </div>
      </div>
      
      <div class="form-row">
        <div class="form-group">
          <label for="zip">ZIP/Postal Code</label>
          <input type="text" id="zip" bind:value={shippingInfo.zip} required>
        </div>
        
        <div class="form-group">
          <label for="country">Country</label>
          <input type="text" id="country" bind:value={shippingInfo.country} required>
        </div>
      </div>
      
      <div class="order-summary">
        <h3>Order Summary</h3>
        <div class="items">
          {#each $cartItems as item}
            <div class="item">
              <span>{item.name} × {item.quantity}</span>
              <span>${(item.price * item.quantity).toFixed(2)}</span>
            </div>
          {/each}
        </div>
        <div class="total">
          <span>Total:</span>
          <span>${$cartTotal.toFixed(2)}</span>
        </div>
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
  .overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(0, 0, 0, 0.75);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
  }
  
  .shipping-form {
    background: white;
    padding: 30px;
    border-radius: 8px;
    width: 90%;
    max-width: 600px;
    max-height: 90vh;
    overflow-y: auto;
  }
  
  .form-group {
    margin-bottom: 15px;
  }
  
  .form-row {
    display: flex;
    gap: 15px;
  }
  
  label {
    display: block;
    margin-bottom: 5px;
    font-weight: bold;
  }
  
  input {
    width: 100%;
    padding: 8px;
    border: 1px solid #ddd;
    border-radius: 4px;
  }
  
  .order-summary {
    margin: 20px 0;
    padding: 15px;
    background: #f9f9f9;
    border-radius: 4px;
  }
  
  .items {
    margin: 10px 0;
  }
  
  .item {
    display: flex;
    justify-content: space-between;
    margin: 5px 0;
  }
  
  .total {
    display: flex;
    justify-content: space-between;
    font-weight: bold;
    margin-top: 10px;
    padding-top: 10px;
    border-top: 1px solid #ddd;
  }
  
  .form-actions {
    display: flex;
    justify-content: space-between;
    margin-top: 20px;
  }
  
  .btn-primary {
    background: #2ecc71;
    color: white;
    border: none;
    padding: 10px 20px;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .btn-secondary {
    background: #95a5a6;
    color: white;
    border: none;
    padding: 10px 20px;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .btn-primary:hover {
    background: #27ae60;
  }
  
  .btn-secondary:hover {
    background: #7f8c8d;
  }
</style> 