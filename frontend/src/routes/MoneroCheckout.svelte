<script>
  import { onMount, onDestroy } from 'svelte';
  import { cart, cartTotal } from '../stores/cart.js';
  import QRCode from 'qrcode';
  import { auth } from '../stores/auth.js';
  
  export let params = {};
  
  let paymentDetails = null;
  let error = null;
  let loading = true;
  let orderStatus = 'pending';
  let checkInterval;
  let order_id = '';
  let qrCodeUrl = '';
  let socket = null;
  let paymentData = null;
  
  onMount(() => {
    console.log("MoneroCheckout component mounted");
    
    // Enhanced auth restoration with more logging
    const tokenBackup = localStorage.getItem('auth_token_backup');
    const userBackup = localStorage.getItem('auth_user_backup');
    
    console.log("Token backup exists:", !!tokenBackup);
    console.log("User backup exists:", !!userBackup);
    console.log("Current auth state:", $auth);
    
    if (tokenBackup) {
      console.log("Restoring authentication from backup token");
      
      // More robust auth restoration
      let userData = null;
      try {
        if (userBackup) {
          userData = JSON.parse(userBackup);
        }
      } catch (e) {
        console.error("Error parsing user backup:", e);
      }
      
      auth.update(state => ({
        ...state,
        isAuthenticated: true,
        token: tokenBackup,
        user: userData || state.user,
        isAdmin: userData ? userData.role === 'admin' : state.isAdmin
      }));
      
      console.log("Auth state after restoration:", $auth);
      
      // Don't clear the token backup yet - keep it for the entire checkout process
      // We'll clear it when the checkout is complete
    }
    
    // Load payment data
    loadPaymentData();
    
    // Don't clear the cart yet - let the user see what they're buying
    // We'll clear it when the payment is confirmed
  });
  
  function loadPaymentData() {
    try {
      // Get payment data from localStorage
      const storedPayment = localStorage.getItem('monero_payment');
      const storedOrderId = localStorage.getItem('current_order_id');
      
      console.log("Retrieved from localStorage - payment:", storedPayment);
      console.log("Retrieved from localStorage - orderId:", storedOrderId);
      
      if (storedPayment) {
        paymentData = JSON.parse(storedPayment);
        
        if (paymentData && paymentData.address) {
          // Generate QR code for the Monero address
          generateQRCode(paymentData.address, paymentData.amount);
        }
      }
      
      if (storedOrderId) {
        order_id = storedOrderId;
      }
      
      if (!paymentData && !order_id) {
        error = "Payment information not found";
      }
    } catch (err) {
      console.error("Error loading payment data:", err);
      error = "Failed to load payment information";
    } finally {
      loading = false;
    }
  }
  
  async function generateQRCode(address, amount) {
    try {
      // Format: monero:<address>?tx_amount=<amount>
      const moneroUri = `monero:${address}?tx_amount=${amount}`;
      
      qrCodeUrl = await QRCode.toDataURL(moneroUri);
    } catch (err) {
      console.error("QR code generation error:", err);
      // Still allow checkout without QR code
    }
  }
  
  function setupWebSocket() {
    // Close existing socket if any
    if (socket) {
      socket.close();
    }
    
    // Create WebSocket connection
    socket = new WebSocket(`ws://localhost:5000/ws/payment/${order_id}`);
    
    socket.onopen = () => {
      console.log(`WebSocket connected for order ${order_id}`);
      // Request initial status
      socket.send(JSON.stringify({
        command: "check_status"
      }));
    };
    
    socket.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        console.log("WebSocket message:", data);
        
        if (data.type === "payment_status") {
          // Update payment status
          const newStatus = data.status.toLowerCase();
          if (newStatus === "confirmed" || newStatus === "completed") {
            orderStatus = 'confirmed';
            
            // Play a sound to notify the user
            const audio = new Audio('/payment-confirmed.mp3');
            audio.play().catch(e => console.log("Audio play error:", e));
            
            // Show a notification if supported
            if ("Notification" in window) {
              if (Notification.permission === "granted") {
                new Notification("Payment Confirmed!", {
                  body: "Your payment has been confirmed and your order is being processed.",
                  icon: "/monero-icon.png"
                });
              } else if (Notification.permission !== "denied") {
                Notification.requestPermission().then(permission => {
                  if (permission === "granted") {
                    new Notification("Payment Confirmed!", {
                      body: "Your payment has been confirmed and your order is being processed.",
                      icon: "/monero-icon.png"
                    });
                  }
                });
              }
            }
          } else {
            orderStatus = newStatus;
          }
        }
      } catch (e) {
        console.error("Error processing WebSocket message:", e);
      }
    };
    
    socket.onerror = (error) => {
      console.error("WebSocket error:", error);
    };
    
    socket.onclose = () => {
      console.log("WebSocket connection closed");
      // Reconnect after a delay
      setTimeout(() => {
        if (orderStatus !== 'confirmed' && orderStatus !== 'completed') {
          setupWebSocket();
        }
      }, 5000);
    };
  }
  
  onDestroy(() => {
    if (socket) {
      socket.close();
    }
    if (checkInterval) {
      clearInterval(checkInterval);
    }
  });

  // Helper to copy text to clipboard
  function copyToClipboard(text) {
    navigator.clipboard.writeText(text)
      .then(() => alert('Copied to clipboard!'))
      .catch(err => console.error('Failed to copy:', err));
  }
  
  // Manually check status
  function checkStatusNow() {
    if (socket && socket.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify({
        command: "check_status"
      }));
    }
  }
</script>

<div class="monero-checkout">
  <h1>Monero Payment</h1>
  
  {#if loading}
    <div class="loading">Loading payment information...</div>
  {:else if error}
    <div class="error">
      <p>{error}</p>
      <button on:click={() => window.history.back()}>Go Back</button>
    </div>
  {:else if paymentData}
    <div class="payment-details">
      <div class="order-summary">
        <h2>Order Summary</h2>
        {#each $cart as item}
          <div class="order-item">
            <span>{item.name} × {item.quantity}</span>
            <span>${(item.price * item.quantity).toFixed(2)}</span>
          </div>
        {/each}
        <div class="total">
          <strong>Total:</strong>
          <span>${$cartTotal.toFixed(2)}</span>
        </div>
      </div>
      
      <div class="payment-info">
        <h2>Payment Information</h2>
        <div class="qr-code">
          {#if qrCodeUrl}
            <img src={qrCodeUrl} alt="Monero QR Code" />
          {:else}
            <div class="qr-placeholder">QR Code Unavailable</div>
          {/if}
        </div>
        
        <div class="payment-details">
          <div class="detail-row">
            <span>Order ID:</span>
            <code>{order_id}</code>
            <button class="copy-btn" on:click={() => copyToClipboard(order_id)}>Copy</button>
          </div>
          
          <div class="detail-row">
            <span>Amount:</span>
            <code>{paymentData.amount} XMR</code>
            <button class="copy-btn" on:click={() => copyToClipboard(paymentData.amount.toFixed(12))}>Copy</button>
          </div>
          
          <div class="detail-row">
            <span>Address:</span>
            <code class="monero-address">{paymentData.address}</code>
            <button class="copy-btn" on:click={() => copyToClipboard(paymentData.address)}>Copy</button>
          </div>
        </div>
        
        <div class="status-indicator status-{orderStatus}">
          <h3>Payment Status: <span class="status-label">{orderStatus}</span></h3>
          
          {#if orderStatus === 'pending'}
            <p>Please send the exact amount to the address above. The payment will be automatically detected.</p>
            <button class="check-btn" on:click={checkStatusNow}>Check Payment Status</button>
          {:else if orderStatus === 'confirmed' || orderStatus === 'completed'}
            <div class="confirmed-message">
              <span class="checkmark">✓</span>
              <p>Your payment has been confirmed! Your order will be processed shortly.</p>
              <button class="order-btn" on:click={() => window.location.href = '/orders'}>View Order</button>
            </div>
          {/if}
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .monero-checkout {
    max-width: 800px;
    margin: 0 auto;
    padding: 20px;
  }
  
  h1 {
    text-align: center;
    margin-bottom: 30px;
    color: #4a4a4a;
  }
  
  h2 {
    color: #2a2a2a;
    margin-bottom: 15px;
    padding-bottom: 10px;
    border-bottom: 1px solid #eee;
  }
  
  .loading {
    text-align: center;
    padding: 50px;
    font-size: 18px;
    color: #666;
  }
  
  .error {
    background-color: #fff8f8;
    border-left: 4px solid #ff6b6b;
    padding: 20px;
    margin: 20px 0;
    border-radius: 4px;
  }
  
  .error button {
    background-color: #4a4a4a;
    color: white;
    border: none;
    padding: 10px 15px;
    border-radius: 4px;
    cursor: pointer;
    margin-top: 15px;
  }
  
  .payment-details {
    display: flex;
    flex-direction: column;
    gap: 30px;
  }
  
  .order-summary {
    background-color: #f9f9f9;
    padding: 20px;
    border-radius: 8px;
  }
  
  .order-item {
    display: flex;
    justify-content: space-between;
    padding: 10px 0;
    border-bottom: 1px dashed #eee;
  }
  
  .total {
    display: flex;
    justify-content: space-between;
    padding: 15px 0 5px;
    font-size: 1.1em;
  }
  
  .payment-info {
    background-color: #f0f0f0;
    padding: 25px;
    border-radius: 8px;
    display: flex;
    flex-direction: column;
    gap: 20px;
  }
  
  .qr-code {
    display: flex;
    justify-content: center;
    margin-bottom: 20px;
  }
  
  .qr-code img {
    max-width: 200px;
    border: 10px solid white;
    border-radius: 8px;
    box-shadow: 0 2px 10px rgba(0,0,0,0.1);
  }
  
  .detail-row {
    display: flex;
    align-items: center;
    margin-bottom: 15px;
    flex-wrap: wrap;
    gap: 10px;
  }
  
  .detail-row span {
    font-weight: bold;
    min-width: 80px;
  }
  
  .detail-row code {
    background-color: #f8f8f8;
    padding: 8px;
    border-radius: 4px;
    border: 1px solid #e0e0e0;
    font-family: monospace;
    flex: 1;
    word-break: break-all;
  }
  
  .monero-address {
    font-size: 0.85em;
  }
  
  .copy-btn {
    background-color: #4a4a4a;
    color: white;
    border: none;
    padding: 6px 10px;
    border-radius: 4px;
    cursor: pointer;
  }
  
  .copy-btn:hover {
    background-color: #333;
  }
  
  .status-indicator {
    background-color: #fff;
    padding: 20px;
    border-radius: 8px;
    margin-top: 20px;
    position: relative;
    transition: all 0.3s ease;
  }
  
  .status-pending {
    border-left: 4px solid #f39c12;
  }
  
  .status-confirmed, .status-completed {
    border-left: 4px solid #2ecc71;
    background-color: #f0fff4;
  }
  
  .status-label {
    text-transform: capitalize;
    font-weight: bold;
  }
  
  .confirmed-message {
    display: flex;
    flex-direction: column;
    align-items: center;
    text-align: center;
  }
  
  .checkmark {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 50px;
    height: 50px;
    background-color: #2ecc71;
    color: white;
    border-radius: 50%;
    font-size: 24px;
    margin-bottom: 15px;
    animation: pulse 2s infinite;
  }
  
  @keyframes pulse {
    0% {
      box-shadow: 0 0 0 0 rgba(46, 204, 113, 0.4);
    }
    70% {
      box-shadow: 0 0 0 15px rgba(46, 204, 113, 0);
    }
    100% {
      box-shadow: 0 0 0 0 rgba(46, 204, 113, 0);
    }
  }
  
  .check-btn, .order-btn {
    background-color: #3498db;
    color: white;
    border: none;
    padding: 12px 20px;
    border-radius: 4px;
    cursor: pointer;
    font-weight: bold;
    margin-top: 15px;
    transition: background-color 0.2s;
  }
  
  .check-btn:hover {
    background-color: #2980b9;
  }
  
  .order-btn {
    background-color: #2ecc71;
  }
  
  .order-btn:hover {
    background-color: #27ae60;
  }
  
  .qr-placeholder {
    width: 200px;
    height: 200px;
    background-color: #eee;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #666;
    border-radius: 8px;
  }
  
  @media (max-width: 600px) {
    .detail-row {
      flex-direction: column;
      align-items: flex-start;
    }
    
    .detail-row code {
      width: 100%;
      margin: 5px 0;
    }
    
    .copy-btn {
      align-self: flex-end;
    }
  }
</style> 