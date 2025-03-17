import { config, API_BASE_URL } from '../config.js';

// Product API
export const ProductAPI = {
  async getProducts() {
    const response = await fetch(config.api.products);
    if (!response.ok) {
      throw new Error(`Error fetching products: ${response.status}`);
    }
    return await response.json();
  },
  
  async checkHealth() {
    try {
      const response = await fetch(config.api.health, {
        signal: AbortSignal.timeout(5000)
      });
      return response.ok;
    } catch (err) {
      console.error("Health check failed:", err);
      return false;
    }
  }
};

// Auth API
export const AuthAPI = {
  async login(credentials) {
    const response = await fetch(`${config.api.auth}/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(credentials)
    });
    
    if (!response.ok) {
      throw new Error(`Login failed: ${response.status}`);
    }
    
    return await response.json();
  },
  
  // Add other auth methods...
};

// Order API
export const OrderAPI = {
  async getOrders(token) {
    const response = await fetch(`${config.api.orders}/my-orders`, {
      headers: { 'Authorization': `Bearer ${token}` }
    });
    
    if (!response.ok) {
      throw new Error(`Failed to fetch orders: ${response.status}`);
    }
    
    return await response.json();
  },
  
  // Add other order methods...
};

// For backward compatibility
export { API_BASE_URL }; 