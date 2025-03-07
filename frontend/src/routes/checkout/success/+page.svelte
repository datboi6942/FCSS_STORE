<script>
  import { onMount } from 'svelte';
  import { goto } from '$app/navigation';
  
  function goHome() {
    goto('/');
  }
</script>

<div class="success-page">
  <div class="success-container">
    <div class="success-icon">✓</div>
    <h1>Payment Successful!</h1>
    <p>Thank you for your purchase. Your order has been processed successfully.</p>
    <p class="detail-text">You will receive a confirmation email with your order details shortly.</p>
    
    <button on:click={goHome} class="home-btn">Return to Home</button>
  </div>
</div>

<style>
  .success-page {
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: calc(100vh - 200px);
    padding: 2rem;
  }
  
  .success-container {
    max-width: 600px;
    padding: 3rem;
    background-color: white;
    border-radius: 8px;
    box-shadow: 0 2px 10px rgba(0,0,0,0.1);
    text-align: center;
  }
  
  .success-icon {
    width: 80px;
    height: 80px;
    background-color: #4caf50;
    border-radius: 50%;
    color: white;
    font-size: 2.5rem;
    line-height: 80px;
    margin: 0 auto 2rem;
  }
  
  h1 {
    color: #4caf50;
    margin-bottom: 1.5rem;
  }
  
  p {
    font-size: 1.1rem;
    margin-bottom: 1rem;
  }
  
  .detail-text {
    color: #666;
    font-size: 0.9rem;
  }
  
  .home-btn {
    background-color: #2196f3;
    color: white;
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: 4px;
    cursor: pointer;
    font-size: 1rem;
    margin-top: 2rem;
  }
  
  .home-btn:hover {
    background-color: #1976d2;
  }
</style> 