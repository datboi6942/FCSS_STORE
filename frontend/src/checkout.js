async function processMoneroCheckout(cartItems, shippingInfo) {
    try {
        const response = await fetch('/api/checkout/monero', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                items: cartItems,
                shipping_info: shippingInfo,
                user_id: getCurrentUserId() // Get current user ID if available
            }),
        });
        
        const data = await response.json();
        
        if (data.success) {
            // Store the order data (including order_id) in the state
            setCheckoutData(data);
            
            // Navigate to the Monero checkout page
            navigateTo('/checkout/monero');
            
            // You might want to store the order ID in localStorage as well
            localStorage.setItem('current_order_id', data.order_id);
        } else {
            showError(data.error || 'Checkout failed');
        }
    } catch (error) {
        console.error('Error during checkout:', error);
        showError('Network error during checkout');
    }
} 