<div class="order-confirmation">
    <h2>Order Confirmed</h2>
    <p>Thank you for your purchase!</p>
    
    <div class="order-details">
        <p>Order ID: <strong>{orderData.order_id}</strong></p>
        <p>Date: {new Date().toLocaleDateString()}</p>
        <p>Total: ${orderData.total.toFixed(2)}</p>
    </div>
    
    <!-- Rest of your order confirmation content -->
</div> 