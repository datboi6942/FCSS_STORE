<script>
  import { onMount } from 'svelte';
  import MoneroCheckout from '../../../components/MoneroCheckout.svelte';
  
  let paymentData = null;
  let loading = true;
  let error = null;
  
  onMount(() => {
    try {
      const storedPayment = localStorage.getItem('monero_payment');
      if (!storedPayment) {
        error = "No payment information found. Please start checkout again.";
        loading = false;
        return;
      }
      
      paymentData = JSON.parse(storedPayment);
      loading = false;
    } catch (err) {
      console.error("Error loading payment data:", err);
      error = "Failed to load payment information. Please try again.";
      loading = false;
    }
  });
</script>

<div class="checkout-page">
  {#if loading}
    <div class="loading">
      <p>Loading payment information...</p>
    </div>
  {:else if error}
    <div class="error">
      <p>{error}</p>
      <button on:click={() => window.location.href = '/'}>Return to Store</button>
    </div>
  {:else if paymentData}
    <MoneroCheckout {paymentData} />
  {/if}
</div>

<style>
  .checkout-page {
    min-height: 100vh;
    background-color: #f5f5f5;
    padding: 2rem;
  }
  
  .loading, .error {
    text-align: center;
    padding: 3rem;
    background: white;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
    max-width: 600px;
    margin: 2rem auto;
  }
  
  .error {
    color: #e53935;
  }
  
  button {
    background-color: #ff6600;
    color: white;
    border: none;
    padding: 0.75rem 1.5rem;
    border-radius: 4px;
    cursor: pointer;
    margin-top: 1rem;
  }
  
  button:hover {
    background-color: #ff8533;
  }
</style> 