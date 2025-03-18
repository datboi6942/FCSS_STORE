// API configuration for the application
const API_HOST = 'http://localhost:5000';  // Use your Raspberry Pi's actual IP
const WS_HOST = API_HOST.replace('http://', 'ws://');

// Export config object with the structure the app expects
export const config = {
  app: {
    name: 'Secure Store',
    environment: 'development',
    version: '1.0.0',
    debug: true
  },
  
  api: {
    base: API_HOST,
    products: `${API_HOST}/products`,
    auth: `${API_HOST}/auth`,
    orders: `${API_HOST}/orders`,
    admin: `${API_HOST}/admin`,
    monero: `${API_HOST}/monero`,
    payment: `${API_HOST}/payment`,
    cart: `${API_HOST}/cart`,
    health: `${API_HOST}/health`,
    profile: `${API_HOST}/auth/profile`,
    checkout: `${API_HOST}/monero/checkout`
  },
  
  ws: {
    base: WS_HOST,
    payment: (orderId) => `${WS_HOST}/ws/payment/${orderId}`
  },
  
  auth: {
    tokenStorageKey: 'jwt',
    userStorageKey: 'user',
    tokenBackupKey: 'auth_token_backup',
    sessionTimeout: 3600 * 24
  },
  
  features: {
    offlineMode: true,
    adminPanel: true,
    analytics: false
  }
};

// Helper functions
export function apiUrl(path) {
  return `${API_HOST}${path.startsWith('/') ? path : '/' + path}`;
}

export function wsUrl(path) {
  return `${WS_HOST}${path.startsWith('/') ? path : '/' + path}`;
}

// For backward compatibility
export const API_BASE_URL = API_HOST;

// Default export
export default config;