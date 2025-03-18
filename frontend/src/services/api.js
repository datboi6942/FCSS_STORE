import { config } from '../config.js';

// Generic API request function
async function apiRequest(endpoint, options = {}) {
  const url = endpoint.startsWith('http') ? endpoint : `${config.api.base}${endpoint}`;
  
  const defaultOptions = {
    headers: {
      'Content-Type': 'application/json',
      'Accept': 'application/json'
    }
  };
  
  // Add auth token if available
  const token = localStorage.getItem(config.auth.tokenStorageKey);
  if (token) {
    defaultOptions.headers.Authorization = `Bearer ${token}`;
  }
  
  // Merge options
  const fetchOptions = {
    ...defaultOptions,
    ...options,
    headers: {
      ...defaultOptions.headers,
      ...options.headers
    }
  };
  
  try {
    const response = await fetch(url, fetchOptions);
    
    // Handle non-2xx responses
    if (!response.ok) {
      const errorData = await response.json().catch(() => null);
      throw new Error(
        errorData?.error || 
        `API request failed: ${response.status} ${response.statusText}`
      );
    }
    
    // Check content type to determine how to parse the response
    const contentType = response.headers.get('content-type');
    
    // If it's a 204 No Content response, return null
    if (response.status === 204) {
      return null;
    }
    
    // If it's a JSON response, parse it as JSON
    if (contentType && contentType.includes('application/json')) {
      return await response.json();
    }
    
    // For text/plain responses, return the text
    if (contentType && contentType.includes('text/plain')) {
      return await response.text();
    }
    
    // For all other responses, try JSON first, then fall back to text
    try {
      return await response.json();
    } catch (e) {
      return await response.text();
    }
  } catch (error) {
    // You can add logging or custom error handling here
    console.error('API request failed:', error);
    throw error;
  }
}

// API service object
export const api = {
  // Auth methods
  auth: {
    login: (credentials) => apiRequest(`${config.api.auth}/login`, {
      method: 'POST',
      body: JSON.stringify(credentials)
    }),
    
    register: (userData) => apiRequest(`${config.api.auth}/register`, {
      method: 'POST',
      body: JSON.stringify(userData)
    }),
    
    profile: () => apiRequest(config.api.profile)
  },
  
  // Products methods
  products: {
    list: () => apiRequest(config.api.products),
    
    add: (product) => apiRequest(config.api.products, {
      method: 'POST',
      body: JSON.stringify(product)
    }),
    
    purchase: (data) => apiRequest(`${config.api.products}/purchase`, {
      method: 'POST',
      body: JSON.stringify(data)
    })
  },
  
  // Orders methods
  orders: {
    list: () => apiRequest(`${config.api.orders}/my-orders`),
    
    getStatus: (orderId) => apiRequest(`${config.api.orders}/status/${orderId}`)
  },
  
  // Cart methods
  cart: {
    checkout: (checkoutData) => apiRequest(`${config.api.cart}/checkout`, {
      method: 'POST',
      body: JSON.stringify(checkoutData)
    })
  },
  
  // Monero methods
  monero: {
    checkout: (data) => apiRequest(config.api.checkout, {
      method: 'POST',
      body: JSON.stringify(data)
    }),
    
    checkPayment: (orderId) => apiRequest(`${config.api.monero}/order_payment/${orderId}`)
  },
  
  // Health check
  health: {
    check: async () => {
      try {
        const result = await apiRequest(config.api.health, { 
          method: 'GET',
          headers: { 'Accept': 'text/plain' }
        });
        // For health endpoint, we just care if we got a response
        return result === "OK" || result === true;
      } catch (err) {
        console.error("Health check failed:", err);
        return false;
      }
    }
  }
};

// For backward compatibility
export const API_BASE_URL = config.api.base; 