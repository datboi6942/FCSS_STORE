<script>
  import { onMount, onDestroy } from 'svelte';
  import { cart, cartTotal } from '../stores/cart.js';
  import QRCode from 'qrcode';
  
  export let orderId;
  
  let paymentData = null;
  let qrCodeUrl = '';
  let paymentStatus = 'initializing'; // initializing, pending, confirmed, completed, error
  let error = null;
  let pollingInterval;
  let timeLeft = 1800; // 30 minutes in seconds
  let countdown;
  
  // Add variables for manual transaction proof submission
  let showProofForm = false;
  let txHash = '';
  let txKey = '';
  let submittingProof = false;
  let proofError = null;
  
  let shippingInfo = {
    name: '',
    address: '',
    city: '',
    state: '',
    zipCode: '',
    country: '',
    email: ''
  };
  let step = 'shipping'; // 'shipping' or 'payment'
  
  async function createPayment() {
    try {
      const response = await fetch('/api/monero/create_payment', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          order_id: orderId,
          amount: $cartTotal
        })
      });
      
      if (!response.ok) {
        throw new Error('Failed to create payment request');
      }
      
      const data = await response.json();
      if (data.success && data.payment) {
        paymentData = data.payment;
        paymentStatus = 'pending';
        
        // Generate QR code
        generateQRCode();
        
        // Start polling for payment status
        startPolling();
        
        // Start countdown timer
        startCountdown();
      } else {
        throw new Error(data.message || 'Unknown error creating payment');
      }
    } catch (err) {
      console.error('Payment creation error:', err);
      error = err.message;
      paymentStatus = 'error';
    }
  }
  
  async function generateQRCode() {
    try {
      // Format: monero:<address>?tx_amount=<amount>&tx_payment_id=<payment_id>
      const moneroUri = `monero:${paymentData.address}?tx_amount=${paymentData.amount}&tx_payment_id=${paymentData.payment_id}`;
      
      qrCodeUrl = await QRCode.toDataURL(moneroUri, {
        width: 256,
        margin: 1,
        color: {
          dark: '#000000',
          light: '#ffffff'
        }
      });
    } catch (err) {
      console.error('QR code generation error:', err);
    }
  }
  
  function startPolling() {
    // Check payment status every 30 seconds
    pollingInterval = setInterval(checkPaymentStatus, 30000);
  }
  
  async function checkPaymentStatus() {
    if (!paymentData) return;
    
    try {
      const response = await fetch(`/api/monero/check_payment/${paymentData.payment_id}`);
      
      if (!response.ok) {
        throw new Error('Failed to check payment status');
      }
      
      const data = await response.json();
      
      if (data.success && data.payment) {
        paymentData = data.payment;
        
        if (data.payment.status === 'Confirmed' || data.payment.status === 'Completed') {
          paymentStatus = data.payment.status.toLowerCase();
          stopPolling();
          stopCountdown();
          
          // Finalize the order
          if (data.payment.status === 'Confirmed') {
            try {
              await fetch(`/api/monero/finalize_order/${paymentData.payment_id}`, {
                method: 'POST'
              });
            } catch (err) {
              console.error('Error finalizing order:', err);
            }
          }
          
          // Clear cart after successful payment
          if (data.payment.status === 'Completed') {
            cart.clearCart();
          }
        }
      }
    } catch (err) {
      console.error('Payment status check error:', err);
    }
  }
  
  function startCountdown() {
    countdown = setInterval(() => {
      timeLeft -= 1;
      
      if (timeLeft <= 0) {
        stopCountdown();
        paymentStatus = 'expired';
      }
    }, 1000);
  }
  
  function stopPolling() {
    if (pollingInterval) {
      clearInterval(pollingInterval);
      pollingInterval = null;
    }
  }
  
  function stopCountdown() {
    if (countdown) {
      clearInterval(countdown);
      countdown = null;
    }
  }
  
  function copyToClipboard(text) {
    navigator.clipboard.writeText(text);
    alert('Copied to clipboard!');
  }
  
  function formatTime(seconds) {
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    return `${minutes}:${remainingSeconds.toString().padStart(2, '0')}`;
  }
  
  // Testing function - in a real app, you would remove this
  async function mockConfirmPayment() {
    if (!paymentData) return;
    
    try {
      const response = await fetch(`/api/monero/mock_confirm/${paymentData.payment_id}`, {
        method: 'POST'
      });
      
      if (response.ok) {
        checkPaymentStatus();
      }
    } catch (err) {
      console.error('Mock confirm error:', err);
    }
  }
  
  // Function to submit transaction proof
  async function submitTransactionProof() {
    if (!txHash || !txKey) {
      proofError = "Transaction hash and key are required";
      return;
    }
    
    submittingProof = true;
    proofError = null;
    
    try {
      const response = await fetch(`/api/monero/submit_proof/${paymentData.payment_id}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({
          tx_hash: txHash,
          tx_key: txKey
        })
      });
      
      const data = await response.json();
      
      if (data.success) {
        // If proof is valid, update local payment data and status
        paymentData = data.payment;
        paymentStatus = data.payment.status.toLowerCase();
        
        if (paymentStatus === 'confirmed' || paymentStatus === 'completed') {
          stopPolling();
          stopCountdown();
        }
      } else {
        proofError = data.message || "Failed to verify transaction";
      }
    } catch (err) {
      console.error('Error submitting proof:', err);
      proofError = "Connection error. Please try again.";
    } finally {
      submittingProof = false;
    }
  }
  
  // Add this function to manually check the payment
  async function checkNow() {
    if (!paymentData) return;
    
    try {
      const response = await fetch(`/api/monero/check_now/${paymentData.payment_id}`, {
        method: 'POST'
      });
      
      if (!response.ok) {
        throw new Error('Failed to check payment');
      }
      
      const data = await response.json();
      
      if (data.success && data.payment) {
        paymentData = data.payment;
        
        if (data.payment.status === 'Confirmed' || data.payment.status === 'Completed') {
          paymentStatus = data.payment.status.toLowerCase();
          stopPolling();
          stopCountdown();
          
          try {
            await fetch(`/api/monero/finalize_order/${paymentData.payment_id}`, {
              method: 'POST'
            });
          } catch (err) {
            console.error('Error finalizing order:', err);
          }
          
          // Clear cart after successful payment
          if (data.payment.status === 'Completed') {
            cart.clearCart();
          }
        }
      }
    } catch (err) {
      console.error('Payment check error:', err);
    }
  }
  
  onMount(() => {
    try {
      const storedPayment = localStorage.getItem('monero_payment');
      if (storedPayment) {
        paymentData = JSON.parse(storedPayment);
      } else {
        error = "Payment information not found";
      }
    } catch (err) {
      console.error("Error loading payment data:", err);
      error = "Failed to load payment information";
    }
  });
  
  onDestroy(() => {
    stopPolling();
    stopCountdown();
  });
  
  async function submitShippingInfo() {
    if (!validateShippingInfo()) {
      return;
    }
    
    try {
      // Save shipping info to localStorage for now
      // In a real app, you'd send this to your backend
      localStorage.setItem('shipping_info', JSON.stringify(shippingInfo));
      step = 'payment';
    } catch (err) {
      console.error("Error saving shipping info:", err);
      error = "Failed to save shipping information";
    }
  }
  
  function validateShippingInfo() {
    // Add validation logic here
    return true;
  }
  
  function copyAddress() {
    if (paymentData?.address) {
      navigator.clipboard.writeText(paymentData.address);
    }
  }
</script>

<div class="checkout-container">
  <h1>Monero Checkout</h1>
  
  {#if error}
    <div class="error">
      <p>{error}</p>
      <button on:click={() => window.location.href = '/'}>Return to Store</button>
    </div>
  {:else if step === 'shipping'}
    <div class="shipping-form">
      <h2>Shipping Information</h2>
      <form on:submit|preventDefault={submitShippingInfo}>
        <div class="form-group">
          <label for="name">Full Name</label>
          <input 
            type="text" 
            id="name" 
            bind:value={shippingInfo.name} 
            required
          />
        </div>
        
        <div class="form-group">
          <label for="email">Email</label>
          <input 
            type="email" 
            id="email" 
            bind:value={shippingInfo.email} 
            required
          />
        </div>
        
        <div class="form-group">
          <label for="address">Address</label>
          <input 
            type="text" 
            id="address" 
            bind:value={shippingInfo.address} 
            required
          />
        </div>
        
        <div class="form-row">
          <div class="form-group">
            <label for="city">City</label>
            <input 
              type="text" 
              id="city" 
              bind:value={shippingInfo.city} 
              required
            />
          </div>
          
          <div class="form-group">
            <label for="state">State/Province</label>
            <input 
              type="text" 
              id="state" 
              bind:value={shippingInfo.state} 
              required
            />
          </div>
        </div>
        
        <div class="form-row">
          <div class="form-group">
            <label for="zipCode">ZIP/Postal Code</label>
            <input 
              type="text" 
              id="zipCode" 
              bind:value={shippingInfo.zipCode} 
              required
            />
          </div>
          
          <div class="form-group">
            <label for="country">Country</label>
            <input 
              type="text" 
              id="country" 
              bind:value={shippingInfo.country} 
              required
            />
          </div>
        </div>
        
        <button type="submit" class="submit-btn">Continue to Payment</button>
      </form>
    </div>
  {:else if step === 'payment' && paymentData}
    <div class="payment-info">
      <h2>Payment Information</h2>
      <p class="order-total">Order Total: {paymentData.amount} XMR</p>
      
      <div class="wallet-address">
        <h3>Send Monero to:</h3>
        <div class="address-container">
          <code>{paymentData.address}</code>
          <button class="copy-btn" on:click={copyAddress}>Copy</button>
        </div>
      </div>
      
      <div class="qr-code">
        {#if qrCodeUrl}
          <img src={qrCodeUrl} alt="Monero Payment QR Code" class="qr-code" />
        {:else}
          <p>Generating QR code...</p>
        {/if}
      </div>
      
      <div class="payment-instructions">
        <h3>Instructions:</h3>
        <ol>
          <li>Copy the Monero address above</li>
          <li>Send exactly {paymentData.amount} XMR to this address</li>
          <li>Wait for confirmation (usually takes 10-20 minutes)</li>
          <li>Your order will be processed automatically once payment is confirmed</li>
        </ol>
      </div>
      
      <div class="order-id">
        Order ID: {orderId}
      </div>
    </div>
  {/if}
</div>

<style>
  .checkout-container {
    max-width: 800px;
    margin: 2rem auto;
    padding: 2rem;
    background-color: white;
    border-radius: 8px;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  }
  
  h1, h2 {
    color: #333;
    margin-bottom: 1.5rem;
  }
  
  .form-group {
    margin-bottom: 1rem;
  }
  
  .form-row {
    display: flex;
    gap: 1rem;
  }
  
  label {
    display: block;
    margin-bottom: 0.5rem;
    color: #666;
  }
  
  input {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 1rem;
  }
  
  .submit-btn {
    background-color: #ff6600;
    color: white;
    border: none;
    padding: 1rem 2rem;
    border-radius: 4px;
    cursor: pointer;
    font-size: 1rem;
    width: 100%;
    margin-top: 1rem;
  }
  
  .submit-btn:hover {
    background-color: #ff8533;
  }
  
  .payment-info {
    text-align: center;
  }
  
  .order-total {
    font-size: 1.5rem;
    margin: 2rem 0;
  }
  
  .wallet-address {
    background-color: #f9f9f9;
    padding: 1rem;
    border-radius: 4px;
    margin: 2rem 0;
  }
  
  .address-container {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 1rem;
  }
  
  code {
    background-color: #eee;
    padding: 0.5rem;
    border-radius: 4px;
    font-family: monospace;
    word-break: break-all;
  }
  
  .copy-btn {
    background-color: #4CAF50;
    color: white;
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .payment-instructions {
    text-align: left;
    margin: 2rem 0;
  }
  
  .payment-instructions ol {
    padding-left: 1.5rem;
  }
  
  .payment-instructions li {
    margin: 0.5rem 0;
  }
  
  .order-id {
    color: #666;
    margin-top: 2rem;
  }
  
  .error {
    color: #e53935;
    text-align: center;
  }
</style> 