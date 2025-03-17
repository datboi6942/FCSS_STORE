// Central configuration file for all API URLs
const API_HOST = import.meta.env.VITE_API_BASE_URL || 'http://192.168.6.53:5000';
const WS_HOST = API_HOST.replace('http://', 'ws://');

// Export configuration
export const config = {
  // API endpoints
  api: {
    base: API_HOST,
    products: `${API_HOST}/products`,
    auth: `${API_HOST}/auth`,
    orders: `${API_HOST}/orders`,
    admin: `${API_HOST}/admin`,
    monero: `${API_HOST}/monero`,
    payment: `${API_HOST}/payment`,
    cart: `${API_HOST}/cart`,
    health: `${API_HOST}/health`
  },
  
  // WebSocket endpoints
  ws: {
    base: WS_HOST,
    payment: (orderId) => `${WS_HOST}/ws/payment/${orderId}`
  }
};

// Helper function for API URLs
export function apiUrl(path) {
  return `${API_HOST}${path.startsWith('/') ? path : '/' + path}`;
}

// Helper for WebSocket URLs
export function wsUrl(path) {
  return `${WS_HOST}${path.startsWith('/') ? path : '/' + path}`;
}

// Export API_HOST directly for backward compatibility
export const API_BASE_URL = API_HOST; 