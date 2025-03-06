<script>
    let orderId = '';
    let amount = 0;
    let method = 'card';
    let currency = 'USD';
    let paymentResponse = null;
    let isSubmitting = false;
    let error = null;
  
    async function initiatePayment() {
      if (!orderId.trim()) {
        error = "Order ID is required";
        return;
      }
  
      if (amount <= 0) {
        error = "Amount must be greater than 0";
        return;
      }
  
      try {
        error = null;
        isSubmitting = true;
        const res = await fetch("http://127.0.0.1:8443/payment/initiate", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ 
            order_id: orderId, 
            method, 
            amount: amount.toString(),
            currency 
          })
        });
        
        if (!res.ok) {
          throw new Error('Payment initiation failed');
        }
        
        paymentResponse = await res.json();
      } catch (err) {
        error = err.message;
        console.error(err);
      } finally {
        isSubmitting = false;
      }
    }
  
    function resetForm() {
      orderId = '';
      amount = 0;
      method = 'card';
      currency = 'USD';
      paymentResponse = null;
      error = null;
    }
  </script>
  
  <div class="payment-container">
    <h2>Secure Payment</h2>
    
    {#if paymentResponse}
      <div class="payment-result">
        <h3>Payment {paymentResponse.status}</h3>
        <p>{paymentResponse.message}</p>
        
        {#if paymentResponse.session_id}
          <div class="session-info">
            <p>Session ID: <code>{paymentResponse.session_id}</code></p>
          </div>
        {/if}
        
        <button class="secondary-button" on:click={resetForm}>
          New Payment
        </button>
      </div>
    {:else}
      <div class="payment-form">
        <div class="form-group">
          <label for="order-id">Order ID:</label>
          <input 
            id="order-id"
            bind:value={orderId} 
            placeholder="Enter order ID" 
            disabled={isSubmitting}
          />
        </div>
        
        <div class="form-group">
          <label for="amount">Amount:</label>
          <input 
            id="amount"
            type="number" 
            bind:value={amount} 
            min="0.01" 
            step="0.01" 
            placeholder="0.00" 
            disabled={isSubmitting}
          />
        </div>
        
        <div class="form-group">
          <label for="method">Payment Method:</label>
          <select id="method" bind:value={method} disabled={isSubmitting}>
            <option value="card">Credit/Debit Card</option>
            <option value="crypto">Cryptocurrency</option>
          </select>
        </div>
        
        <div class="form-group">
          <label for="currency">Currency:</label>
          <select id="currency" bind:value={currency} disabled={isSubmitting}>
            <option value="USD">USD - US Dollar</option>
            <option value="EUR">EUR - Euro</option>
            <option value="GBP">GBP - British Pound</option>
            <option value="BTC">BTC - Bitcoin</option>
          </select>
        </div>
        
        {#if error}
          <p class="error">{error}</p>
        {/if}
        
        <button on:click={initiatePayment} disabled={isSubmitting}>
          {isSubmitting ? 'Processing...' : 'Initiate Payment'}
        </button>
      </div>
    {/if}
    
    <div class="payment-info">
      <h3>About Our Secure Payments</h3>
      <p>All payment information is encrypted using industry-standard protocols. We support both traditional payment methods and cryptocurrencies for maximum flexibility and security.</p>
      <p>Your payment information is never stored on our servers and is processed through secure, PCI-compliant gateways.</p>
    </div>
  </div>
  
  <style>
    .payment-container {
      max-width: 700px;
      margin: 0 auto;
    }
  
    .payment-form {
      background: white;
      padding: 1.5rem;
      border-radius: 8px;
      box-shadow: 0 2px 4px rgba(0, 0, 0, 0.1);
      margin-bottom: 2rem;
    }
  
    .form-group {
      margin-bottom: 1.25rem;
    }
  
    label {
      display: block;
      margin-bottom: 0.5rem;
      font-weight: 500;
    }
  
    input, select {
      width: 100%;
      padding: 0.75rem;
      border: 1px solid #ddd;
      border-radius: 4px;
      font-size: 1rem;
    }
  
    button {
      background: #1e90ff;
      color: white;
      border: none;
      padding: 0.75rem 1.5rem;
      border-radius: 4px;
      cursor: pointer;
      font-size: 1rem;
      width: 100%;
    }
  
    button:hover {
      background: #167edb;
    }
  
    button:disabled {
      background: #b3d1ff;
      cursor: not-allowed;
    }
  
    .secondary-button {
      background: #f5f5f5;
      color: #333;
      border: 1px solid #ddd;
    }
  
    .secondary-button:hover {
      background: #e5e5e5;
    }
  
    .error {
      color: #e74c3c;
      background: #fadbd8;
      padding: 0.75rem;
      border-radius: 4px;
      margin-bottom: 1rem;
    }
  
    .payment-result {
      background: #d4edda;
      color: #155724;
      padding: 1.5rem;
      border-radius: 8px;
      margin-bottom: 2rem;
      text-align: center;
    }
  
    .session-info {
      background: #f8f9fa;
      padding: 0.75rem;
      border-radius: 4px;
      margin: 1rem 0;
    }
  
    .session-info code {
      background: #e9ecef;
      padding: 0.2rem 0.4rem;
      border-radius: 3px;
      font-family: monospace;
    }
  
    .payment-info {
      background: #f8f9fa;
      padding: 1.5rem;
      border-radius: 8px;
      border-left: 4px solid #1e90ff;
    }
  
    .payment-info h3 {
      margin-top: 0;
      color: #1e90ff;
    }
  </style>