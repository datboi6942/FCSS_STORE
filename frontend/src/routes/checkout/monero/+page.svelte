<script>
  import { onMount } from 'svelte';
  
  // State variables
  let checkoutData = null;
  let orderID = 'Loading...';
  let paymentAddress = '';
  let amountXMR = '';
  let totalUSD = '';
  let debugInfo = [];
  
  // Helper to add debug messages
  function debug(message) {
    console.log("MoneroCheckout Debug:", message);
    debugInfo = [...debugInfo, { time: new Date().toISOString(), message }];
  }
  
  onMount(() => {
    debug("Component mounted");
    loadCheckoutData();
  });
  
  function loadCheckoutData() {
    debug("Attempting to load checkout data");
    
    // Method 1: Try to get from URL parameter
    const urlParams = new URLSearchParams(window.location.search);
    if (urlParams.has('order_id')) {
      const urlOrderId = urlParams.get('order_id');
      debug(`Found order_id in URL: ${urlOrderId}`);
      orderID = urlOrderId;
    } else {
      debug("No order_id found in URL");
    }
    
    // Method 2: Try to get direct order ID from localStorage
    const storedOrderID = localStorage.getItem('current_order_id');
    if (storedOrderID) {
      debug(`Found direct order_id in localStorage: ${storedOrderID}`);
      orderID = storedOrderID;
    } else {
      debug("No direct order_id found in localStorage");
    }
    
    // Method 3: Try to get from checkoutData in localStorage
    const storedData = localStorage.getItem('checkoutData');
    if (storedData) {
      debug(`Found checkoutData in localStorage: ${storedData.substring(0, 50)}...`);
      try {
        const parsedData = JSON.parse(storedData);
        debug(`Parsed checkoutData: ${JSON.stringify(parsedData).substring(0, 50)}...`);
        
        checkoutData = parsedData;
        
        if (parsedData.order_id) {
          debug(`Found order_id in checkoutData: ${parsedData.order_id}`);
          orderID = parsedData.order_id;
        } else {
          debug("No order_id found in checkoutData");
        }
        
        if (parsedData.payment && parsedData.payment.address) {
          debug(`Found payment address: ${parsedData.payment.address.substring(0, 10)}...`);
          paymentAddress = parsedData.payment.address;
        }
        
        if (parsedData.payment && parsedData.payment.amount_xmr) {
          debug(`Found XMR amount: ${parsedData.payment.amount_xmr}`);
          amountXMR = parsedData.payment.amount_xmr;
        }
        
        if (parsedData.total) {
          debug(`Found USD total: ${parsedData.total}`);
          totalUSD = parsedData.total;
        }
      } catch (e) {
        debug(`Error parsing checkoutData: ${e.message}`);
      }
    } else {
      debug("No checkoutData found in localStorage");
    }
    
    // Method 4: Create temporary data for testing if nothing else works
    if (orderID === 'Loading...' || !orderID) {
      debug("Using fallback dummy data");
      orderID = "test-order-" + Math.random().toString(36).substring(2, 10);
      paymentAddress = "44AFFq5kSiGBoZ4NMDwYtN18obc8AemS33DBLWs3H7otXft3XjrpDtQGv7SqSsaBYBb98uNbr2VBBEt7f2wfn3RVGQBEP3A";
      amountXMR = "0.25";
      totalUSD = "99.99";
    }
  }
  
  function copyToClipboard(text) {
    navigator.clipboard.writeText(text).then(() => {
      alert('Copied to clipboard!');
    }).catch(err => {
      console.error('Failed to copy: ', err);
    });
  }
  
  // Function to manually refresh data
  function refreshData() {
    debug("Manual refresh triggered");
    loadCheckoutData();
  }
  
  function createTestOrder() {
    debug("Creating test order data manually");
    
    const mockOrderId = "demo-order-" + Math.random().toString(36).substring(2, 10);
    const mockResponse = {
      success: true,
      order_id: mockOrderId,
      payment: {
        payment_id: "pay-" + Math.random().toString(36).substring(2, 10),
        address: "44AFFq5kSiGBoZ4NMDwYtN18obc8AemS33DBLWs3H7otXft3XjrpDtQGv7SqSsaBYBb98uNbr2VBBEt7f2wfn3RVGQBEP3A",
        amount_xmr: "0.25",
        status: "Pending"
      },
      total: 99.99,
      message: "Please send Monero to the provided address"
    };
    
    // Store the mock response
    localStorage.setItem('checkoutData', JSON.stringify(mockResponse));
    localStorage.setItem('current_order_id', mockOrderId);
    
    // Reload the page with the order ID in URL
    window.location.href = `/checkout/monero?order_id=${mockOrderId}`;
  }
</script>

<div class="monero-checkout">
  <h2>Monero Checkout</h2>
  
  <div class="payment-details">
    <p>Amount: {totalUSD || '0.00'} USD</p>
    <p>Monero Address: 
      <span class="monero-address">{paymentAddress || 'Loading...'}</span>
      <button on:click={() => copyToClipboard(paymentAddress)} class="copy-btn">Copy</button>
    </p>
    <p>Amount in XMR: {amountXMR || '0.00'}</p>
    
    <!-- Add the order ID section with styling -->
    <div class="order-id-section">
      <p>Your Order ID: <strong>{orderID}</strong></p>
      <button on:click={() => copyToClipboard(orderID)} class="copy-btn">Copy Order ID</button>
      <p class="order-id-note">Please save this Order ID for tracking your purchase.</p>
    </div>
  </div>
  
  <div class="instructions">
    <h3>How to Pay</h3>
    <ol>
      <li>Open your Monero wallet</li>
      <li>Send exactly {amountXMR || '0.00'} XMR to the address above</li>
      <li>Wait for confirmation (usually takes about 20 minutes)</li>
      <li>Your order will be processed automatically once payment is confirmed</li>
    </ol>
  </div>
  
  <button class="check-payment-btn" on:click={refreshData}>Refresh Order Data</button>
  <a href="/" class="back-btn">Return to Store</a>
  
  <!-- Debug section (only shown in development) -->
  {#if window.location.hostname === 'localhost'}
    <div class="debug-section">
      <h4>Debug Information</h4>
      <button on:click={refreshData}>Refresh Data</button>
      <p>Order ID: {orderID}</p>
      <p>Direct localStorage Order ID: {localStorage.getItem('current_order_id')}</p>
      <p>URL Params: {window.location.search}</p>
      <div class="debug-log">
        {#each debugInfo as entry}
          <div class="debug-entry">
            <span class="debug-time">{entry.time.split('T')[1].split('.')[0]}</span>
            <span class="debug-message">{entry.message}</span>
          </div>
        {/each}
      </div>
      <h4>Raw Checkout Data</h4>
      <pre>{JSON.stringify(checkoutData, null, 2)}</pre>
    </div>
    
    <!-- Add this button right after the debug information section -->
    <div class="test-controls">
      <h4>Test Controls</h4>
      <button on:click={createTestOrder}>Create Test Order Data</button>
      <button on:click={() => window.location.href = "/cart"}>Go to Cart</button>
    </div>
  {/if}
</div>

<style>
  .monero-checkout {
    max-width: 800px;
    margin: 0 auto;
    padding: 20px;
    background-color: #fff;
    border-radius: 8px;
    box-shadow: 0 2px 10px rgba(0, 0, 0, 0.1);
  }
  
  h2 {
    color: #333;
    border-bottom: 2px solid #f60;
    padding-bottom: 10px;
    margin-bottom: 20px;
  }
  
  .payment-details {
    background-color: #f9f9f9;
    padding: 15px;
    border-radius: 5px;
    margin-bottom: 20px;
  }
  
  .monero-address {
    font-family: monospace;
    background-color: #eee;
    padding: 3px 6px;
    border-radius: 3px;
    word-break: break-all;
  }
  
  .instructions {
    margin-bottom: 30px;
  }
  
  .instructions ol {
    padding-left: 20px;
  }
  
  .instructions li {
    margin-bottom: 10px;
  }
  
  .copy-btn {
    background-color: #4CAF50;
    color: white;
    border: none;
    padding: 5px 10px;
    border-radius: 3px;
    cursor: pointer;
    margin-left: 10px;
    font-size: 0.8rem;
  }
  
  .check-payment-btn {
    background-color: #f60;
    color: white;
    border: none;
    padding: 10px 15px;
    border-radius: 5px;
    cursor: pointer;
    font-size: 1rem;
    margin-right: 10px;
  }
  
  .back-btn {
    display: inline-block;
    background-color: #ccc;
    color: #333;
    padding: 10px 15px;
    border-radius: 5px;
    text-decoration: none;
  }
  
  .order-id-section {
    margin-top: 20px;
    padding: 15px;
    background-color: #f8f9fa;
    border-radius: 5px;
    border-left: 4px solid #28a745;
  }
  
  .order-id-note {
    font-size: 0.85em;
    color: #6c757d;
    margin-top: 5px;
  }
  
  /* Debug section styles */
  .debug-section {
    margin-top: 40px;
    border-top: 2px dashed #ccc;
    padding-top: 20px;
  }
  
  .debug-log {
    height: 200px;
    overflow-y: auto;
    background-color: #1e1e1e;
    color: #ddd;
    padding: 10px;
    border-radius: 5px;
    font-family: monospace;
    margin: 10px 0;
  }
  
  .debug-entry {
    margin-bottom: 5px;
    display: flex;
  }
  
  .debug-time {
    color: #f7a;
    margin-right: 10px;
    flex-shrink: 0;
  }
  
  .debug-message {
    color: #afa;
    word-break: break-all;
  }
  
  pre {
    background-color: #f5f5f5;
    padding: 10px;
    border-radius: 5px;
    overflow-x: auto;
    white-space: pre-wrap;
    word-break: break-all;
    max-height: 300px;
    overflow-y: auto;
  }
</style> 