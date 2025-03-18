<script>
  import { onMount, onDestroy } from 'svelte';
  import { cart, cartTotal, clearCart } from '../stores/cart.js';
  import QRCode from 'qrcode';
  import { auth } from '../stores/auth.js';
  import { navigate } from 'svelte-routing';
  import { config } from '../config.js';
  import { api } from '../services/api.js';
  
  export const params = {}; // For external reference
  
  let paymentDetails = null;
  let error = null;
  let loading = true;
  let orderStatus = 'pending';
  let checkInterval;
  let order_id = '';
  let qrCodeUrl = '';
  let socket = null;
  let paymentData = null;
  let pollingEnabled = true; // Fallback to polling if WebSocket fails
  
  // Function to go back to the cart page
  function goToCart() {
    navigate('/cart');
  }

  // Function to poll for payment status when WebSocket isn't available
  function startPolling() {
    if (checkInterval) {
      clearInterval(checkInterval);
    }
    
    checkInterval = setInterval(async () => {
      if (!order_id || !pollingEnabled) return;
      
      try {
        const data = await api.monero.checkPayment(order_id);
        
        // Process the data
        if (data.success && data.status) {
          if (data.status.toLowerCase() === 'confirmed' || data.status.toLowerCase() === 'completed') {
            orderStatus = 'confirmed';
            clearInterval(checkInterval);
            
            // Make sure the order ID is properly stored before navigation
            if (order_id) {
              localStorage.setItem('current_order_id', order_id);
            }
            
            // Clear other checkout data but keep order_id for the success page
            localStorage.removeItem('monero_payment');
            localStorage.removeItem('checkout_data');
            cart.clear();
            
            // Redirect to success page
            setTimeout(() => {
              navigate('/checkout/success');
            }, 2000);
          }
        }
      } catch (err) {
        // Log the error but don't stop polling
        console.error("Error polling payment status:", err);
      }
    }, 5000); // Check every 5 seconds
  }
  
  function loadOrCreatePayment() {
    // Try to load from checkout data
    const checkoutDataString = localStorage.getItem('checkout_data');
    
    if (checkoutDataString) {
      try {
        const checkoutData = JSON.parse(checkoutDataString);
        
        // Use API service for checkout
        api.cart.checkout(checkoutData)
          .then(data => {
            // Set payment details and order ID
            paymentDetails = data;
            order_id = data.order_id;
            paymentData = data.payment;
            
            // Store payment details for future reference
            localStorage.setItem('monero_payment', JSON.stringify(data.payment));
            localStorage.setItem('current_order_id', data.order_id);
            
            // Generate QR code
            if (data.payment && data.payment.address) {
              generateQRCode(data.payment.address, data.payment.amount);
            }
            
            // Clear checkout data from storage
            localStorage.removeItem('checkout_data');
            
            // Try to set up WebSocket or fall back to polling
            setupWebSocket();
          })
          .catch(err => {
            error = 'Network error: ' + err.message;
            loading = false;
          });
        
        return; // Exit function after initiating checkout
      } catch (e) {
        console.error("Error parsing checkout data:", e);
        // Continue to fallback methods
      }
    }
    
    // Fallback: Try to load existing payment from localStorage
    const storedPayment = localStorage.getItem('monero_payment');
    const storedOrderId = localStorage.getItem('current_order_id');
    
    if (storedPayment && storedOrderId) {
      try {
        paymentData = JSON.parse(storedPayment);
        order_id = storedOrderId;
        
        paymentDetails = {
          success: true,
          order_id: order_id,
          payment: paymentData
        };
        
        // Generate QR code
        if (paymentData && paymentData.address) {
          generateQRCode(paymentData.address, paymentData.amount);
        }
        
        // Try to set up WebSocket or fall back to polling
        setupWebSocket();
        loading = false;
      } catch (e) {
        console.error("Error loading saved payment:", e);
        error = "Failed to load saved payment data";
        loading = false;
      }
      
      return; // Exit function after successfully loading
    }
    
    // If we get here, we couldn't load payment data
    error = "No payment information found. Please return to cart.";
    loading = false;
  }
  
  onMount(() => {
    console.log("MoneroCheckout component mounted");
    
    // Load or create payment
    loadOrCreatePayment();
    
    return () => {
      // Clean up
      if (socket) {
        socket.close();
      }
      if (checkInterval) {
        clearInterval(checkInterval);
      }
    };
  });
  
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
    
    if (!order_id) {
      console.error("Cannot setup WebSocket - no order ID");
      startPolling(); // Fall back to polling
      return;
    }
    
    try {
      socket = new WebSocket(config.ws.payment(order_id));
      
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
            const newStatus = data.status.toLowerCase();
            if (newStatus === "confirmed" || newStatus === "completed") {
              orderStatus = 'confirmed';
              
              // Make sure the order ID is properly stored before navigation
              if (order_id) {
                localStorage.setItem('current_order_id', order_id);
              }
              
              // Clear other checkout data
              localStorage.removeItem('monero_payment');
              localStorage.removeItem('checkout_data');
              cart.clear();
              
              // Redirect to success page
              setTimeout(() => {
                navigate('/checkout/success');
              }, 2000);
            }
          }
        } catch (e) {
          console.error("Error processing WebSocket message:", e);
        }
      };
      
      socket.onerror = (error) => {
        console.error("WebSocket error:", error);
        startPolling(); // Fall back to polling
      };
      
      socket.onclose = () => {
        console.log("WebSocket connection closed");
        startPolling(); // Fall back to polling
      };
    } catch (e) {
      console.error("Error setting up WebSocket:", e);
      startPolling(); // Fall back to polling
    }
  }
  
  onDestroy(() => {
    // Clean up
    if (socket) {
      socket.close();
    }
    if (checkInterval) {
      clearInterval(checkInterval);
    }
  });
  
  function copyToClipboard(text) {
    navigator.clipboard.writeText(text).then(
      () => {
        alert("Copied to clipboard");
      },
      (err) => {
        console.error("Could not copy: ", err);
      }
    );
  }
</script>

<div class="container">
  <h1>Monero Payment</h1>
  
  {#if loading}
    <div class="loading">
      <p>Loading payment details...</p>
      <div class="spinner"></div>
    </div>
  {:else if error}
    <div class="error-container">
      <p class="error">{error}</p>
      <button class="back-btn" on:click={goToCart}>Go Back</button>
    </div>
  {:else if paymentDetails && paymentData}
    {#if orderStatus === 'confirmed'}
      <div class="success-message">
        <h2>Payment Confirmed!</h2>
        <p>Your order has been processed successfully.</p>
        <p>You will be redirected to the success page...</p>
      </div>
    {:else}
      <div class="payment-details">
        <div class="order-info">
          <p><strong>Order ID:</strong> {order_id}</p>
          <p><strong>Amount:</strong> {paymentData.amount} XMR</p>
          <p><strong>Status:</strong> Awaiting Payment</p>
        </div>
        
        <div class="payment-instructions">
          <h2>Payment Instructions</h2>
          <p>Please send exactly <strong>{paymentData.amount} XMR</strong> to the following address:</p>
          
          <div class="address-container">
            <div class="monero-address">{paymentData.address}</div>
            <button class="copy-btn" on:click={() => copyToClipboard(paymentData.address)}>Copy</button>
          </div>
          
          {#if qrCodeUrl}
            <div class="qr-container">
              <h3>Scan with Monero Wallet</h3>
              <img src={qrCodeUrl} alt="Payment QR Code" class="qr-code" />
            </div>
          {/if}
          
          <div class="payment-notice">
            <p><strong>Important:</strong> This page will automatically update when your payment is confirmed.</p>
            <p>Please keep this page open until the payment is confirmed.</p>
          </div>
        </div>
      </div>
    {/if}
  {:else}
    <div class="error-container">
      <p class="error">Error: Could not load payment details</p>
      <button class="back-btn" on:click={goToCart}>Return to Cart</button>
    </div>
  {/if}
</div>

<style>
  .container {
    max-width: 800px;
    margin: 2rem auto;
    padding: 2rem;
    background-color: white;
    border-radius: 8px;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  }
  
  h1 {
    text-align: center;
    color: #333;
    margin-bottom: 2rem;
  }
  
  .loading {
    text-align: center;
    padding: 2rem;
  }
  
  .spinner {
    border: 4px solid rgba(0, 0, 0, 0.1);
    border-radius: 50%;
    border-top: 4px solid #3498db;
    width: 40px;
    height: 40px;
    animation: spin 1s linear infinite;
    margin: 1rem auto;
  }
  
  @keyframes spin {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
  }
  
  .error-container {
    text-align: center;
    padding: 2rem;
  }
  
  .error {
    color: #e74c3c;
    font-weight: bold;
    margin-bottom: 1.5rem;
  }
  
  .back-btn {
    padding: 0.75rem 1.5rem;
    background-color: #3498db;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 1rem;
  }
  
  .payment-details {
    display: flex;
    flex-direction: column;
    gap: 2rem;
  }
  
  .order-info {
    background-color: #f8f9fa;
    padding: 1.5rem;
    border-radius: 8px;
    border-left: 4px solid #3498db;
  }
  
  .payment-instructions {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }
  
  .address-container {
    display: flex;
    align-items: center;
    background-color: #f8f9fa;
    padding: 1rem;
    border-radius: 4px;
    margin-top: 0.5rem;
  }
  
  .monero-address {
    flex: 1;
    word-break: break-all;
    font-family: monospace;
    font-size: 0.9rem;
  }
  
  .copy-btn {
    background-color: #3498db;
    color: white;
    border: none;
    padding: 0.5rem 1rem;
    border-radius: 4px;
    cursor: pointer;
    margin-left: 1rem;
  }
  
  .qr-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    margin: 1.5rem 0;
  }
  
  .qr-code {
    max-width: 200px;
    margin-top: 1rem;
  }
  
  .payment-notice {
    background-color: #fff3e0;
    padding: 1rem;
    border-radius: 4px;
    border-left: 4px solid #ff9800;
  }
  
  .success-message {
    text-align: center;
    background-color: #e8f5e9;
    padding: 2rem;
    border-radius: 8px;
    border-left: 4px solid #4caf50;
  }
  
  @media (max-width: 768px) {
    .container {
      padding: 1rem;
      margin: 1rem;
    }
    
    .address-container {
      flex-direction: column;
      align-items: stretch;
    }
    
    .copy-btn {
      margin-left: 0;
      margin-top: 0.5rem;
      width: 100%;
    }
  }
</style> 