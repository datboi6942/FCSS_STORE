<script>
  import { onMount } from 'svelte';
  import { navigate } from 'svelte-routing';
  import { cart } from '../stores/cart.js';

  let orderNumber = '';
  
  onMount(() => {
    // Clear any remaining cart items
    cart.clear();
    
    // Get order ID from localStorage if available
    orderNumber = localStorage.getItem('current_order_id') || 'Unknown';
    
    // Clear localStorage data related to checkout
    localStorage.removeItem('checkout_data');
    localStorage.removeItem('monero_payment');
    localStorage.removeItem('current_order_id');
  });
  
  function goToOrders() {
    navigate('/account/orders');
  }
  
  function goToHome() {
    navigate('/');
  }
</script>

<div class="success-container">
  <div class="success-card">
    <div class="success-header">
      <svg class="checkmark" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 52 52">
        <circle class="checkmark-circle" cx="26" cy="26" r="25" fill="none"/>
        <path class="checkmark-check" fill="none" d="M14.1 27.2l7.1 7.2 16.7-16.8"/>
      </svg>
      <h1>Payment Successful!</h1>
    </div>
    
    <div class="order-details">
      <p>Thank you for your purchase. Your order has been successfully processed.</p>
      <p>Order Number: <strong>{orderNumber}</strong></p>
      <p>A confirmation email will be sent to your registered email address.</p>
    </div>
    
    <div class="next-steps">
      <p>What would you like to do next?</p>
      <div class="button-group">
        <button class="orders-btn" on:click={goToOrders}>View My Orders</button>
        <button class="home-btn" on:click={goToHome}>Continue Shopping</button>
      </div>
    </div>
  </div>
</div>

<style>
  .success-container {
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 80vh;
    padding: 2rem;
  }
  
  .success-card {
    background-color: white;
    border-radius: 8px;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
    padding: 2rem;
    max-width: 600px;
    width: 100%;
  }
  
  .success-header {
    display: flex;
    flex-direction: column;
    align-items: center;
    margin-bottom: 2rem;
  }
  
  .checkmark {
    width: 80px;
    height: 80px;
    margin-bottom: 1rem;
  }
  
  .checkmark-circle {
    stroke: #4CAF50;
    stroke-width: 2;
    stroke-dasharray: 166;
    stroke-dashoffset: 166;
    fill: none;
    animation: stroke 0.6s cubic-bezier(0.65, 0, 0.45, 1) forwards;
  }
  
  .checkmark-check {
    stroke: #4CAF50;
    stroke-width: 2;
    stroke-dasharray: 48;
    stroke-dashoffset: 48;
    animation: stroke 0.3s cubic-bezier(0.65, 0, 0.45, 1) 0.8s forwards;
  }
  
  @keyframes stroke {
    100% {
      stroke-dashoffset: 0;
    }
  }
  
  h1 {
    color: #333;
    font-size: 1.8rem;
    text-align: center;
    margin: 0;
  }
  
  .order-details {
    margin-bottom: 2rem;
    line-height: 1.6;
  }
  
  .next-steps {
    text-align: center;
    margin-top: 2rem;
  }
  
  .button-group {
    display: flex;
    justify-content: center;
    gap: 1rem;
    margin-top: 1rem;
  }
  
  button {
    padding: 0.75rem 1.5rem;
    border: none;
    border-radius: 4px;
    font-size: 1rem;
    cursor: pointer;
    transition: background-color 0.2s;
  }
  
  .orders-btn {
    background-color: #4CAF50;
    color: white;
  }
  
  .home-btn {
    background-color: #f1f1f1;
    color: #333;
  }
  
  .orders-btn:hover {
    background-color: #3d8b40;
  }
  
  .home-btn:hover {
    background-color: #e0e0e0;
  }
  
  @media (max-width: 600px) {
    .button-group {
      flex-direction: column;
    }
    
    button {
      width: 100%;
      margin-bottom: 0.5rem;
    }
  }
</style> 