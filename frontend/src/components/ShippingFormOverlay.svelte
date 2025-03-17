<script>
  import { createEventDispatcher } from 'svelte';
  
  const dispatch = createEventDispatcher();
  
  let shippingInfo = {
    name: '',
    email: '',
    address: '',
    city: '',
    state: '',
    zip: '',
    country: 'United States'
  };
  
  function handleSubmit() {
    // Validate form
    if (!shippingInfo.name || !shippingInfo.address || !shippingInfo.city || 
        !shippingInfo.state || !shippingInfo.zip || !shippingInfo.country || 
        !shippingInfo.email) {
      alert('Please fill out all fields');
      return;
    }
    
    // Dispatch the submit event with the shipping info
    dispatch('submit', shippingInfo);
  }
  
  function close() {
    dispatch('close');
  }
</script>

<div class="shipping-overlay">
  <div class="shipping-form">
    <h2>Shipping Information</h2>
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
        <label for="address">Street Address</label>
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
      
      <div class="button-group">
        <button type="button" class="cancel-btn" on:click={close}>Cancel</button>
        <button type="submit" class="submit-btn">Continue to Payment</button>
      </div>
    </form>
  </div>
</div>

<style>
  .shipping-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background-color: rgba(0, 0, 0, 0.5);
    z-index: 1100;
    display: flex;
    justify-content: center;
    align-items: center;
  }
  
  .shipping-form {
    background-color: white;
    padding: 2rem;
    border-radius: 8px;
    width: 90%;
    max-width: 500px;
    max-height: 90vh;
    overflow-y: auto;
  }
  
  h2 {
    margin-top: 0;
    margin-bottom: 1.5rem;
    text-align: center;
  }
  
  .form-group {
    margin-bottom: 1rem;
    width: 100%;
  }
  
  .form-row {
    display: flex;
    gap: 1rem;
  }
  
  label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: bold;
  }
  
  input {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid #ccc;
    border-radius: 4px;
    font-size: 1rem;
  }
  
  .button-group {
    display: flex;
    justify-content: space-between;
    margin-top: 1.5rem;
  }
  
  .cancel-btn, .submit-btn {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 1rem;
  }
  
  .cancel-btn {
    background-color: #e0e0e0;
    color: #333;
  }
  
  .submit-btn {
    background-color: #4CAF50;
    color: white;
  }
</style> 