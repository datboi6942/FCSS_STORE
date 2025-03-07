<script>
  import { onMount, onDestroy } from 'svelte';
  import { cartItems, cartTotal } from '../stores/cart.js';
  
  export let params = {};
  
  let paymentDetails = null;
  let error = null;
  let loading = true;
  let orderStatus = 'pending';
  let checkInterval;
  let order_id = '';
  let qrCodeUrl = '';
  let socket = null;
  
  onMount(async () => {
    try {
      if (params && params.order_id) {
        order_id = params.order_id;
      } else {
        const pathParts = window.location.pathname.split('/');
        order_id = pathParts[pathParts.length - 1];
      }
      
      console.log("Checking payment for order:", order_id);
      
      if (order_id) {
        const response = await fetch(`http://localhost:5000/monero/order_payment/${order_id}`);
        
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        const data = await response.json();
        console.log("Payment data:", data);
        
        if (data.success) {
          paymentDetails = data;
          orderStatus = data.payment?.status?.toLowerCase() || 'pending';
          
          // Generate QR code URL with proper Monero URI format
          if (data.payment && data.payment.address) {
            const address = data.payment.address;
            const amount = data.payment.amount.toFixed(12); // Monero amounts use 12 decimal places
            qrCodeUrl = `https://api.qrserver.com/v1/create-qr-code/?size=200x200&data=monero:${address}?tx_amount=${amount}`;
            console.log("Generated QR code for Monero address:", address);
            
            // Set up WebSocket connection for real-time updates
            setupWebSocket();
          }
        } else {
          error = data.error || 'Failed to load payment details';
        }
      } else {
        error = 'No order ID provided';
      }
      
    } catch (e) {
      console.error("Error loading payment details:", e);
      error = e.message;
    } finally {
      loading = false;
    }
  });
  
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
    <div class="loading">Loading payment details...</div>
  {:else if error}
    <div class="error">
      <p>Error: {error}</p>
      <button on:click={() => window.history.back()}>Go Back</button>
    </div>
  {:else if paymentDetails}
    <div class="payment-details">
      <div class="order-summary">
        <h2>Order Summary</h2>
        {#each $cartItems as item}
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
            <div class="qr-placeholder">QR Code Loading...</div>
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
            <code>{paymentDetails.payment?.amount.toFixed(12)} XMR</code>
            <button class="copy-btn" on:click={() => copyToClipboard(paymentDetails.payment?.amount.toFixed(12))}>Copy</button>
          </div>
          
          <div class="detail-row">
            <span>Address:</span>
            <code class="monero-address">{paymentDetails.payment?.address}</code>
            <button class="copy-btn" on:click={() => copyToClipboard(paymentDetails.payment?.address)}>Copy</button>
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