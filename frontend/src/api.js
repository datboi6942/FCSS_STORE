import { auth } from './stores/auth.js';

// Base URL for all API calls
const API_BASE_URL = 'http://localhost:8443';

/**
 * Helper function to make API calls with proper authentication
 * @param {string} endpoint - API endpoint (starting with /)
 * @param {Object} options - Fetch options
 * @returns {Promise<Response>} Fetch response
 */
export async function apiCall(endpoint, options = {}) {
  // Get the current token
  let token = null;
  
  // This works because we're not waiting for the subscription - just checking current value
  auth.subscribe(value => {
    token = value.token;
  })();
  
  // Set up default headers
  const headers = {
    'Content-Type': 'application/json',
    ...options.headers || {}
  };
  
  // Add authorization header if we have a token
  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }
  
  // Make the request
  return fetch(`${API_BASE_URL}${endpoint}`, {
    ...options,
    headers
  });
}

/**
 * Shorthand for GET requests
 */
export async function get(endpoint) {
  return apiCall(endpoint);
}

/**
 * Shorthand for POST requests
 */
export async function post(endpoint, data) {
  return apiCall(endpoint, {
    method: 'POST',
    body: JSON.stringify(data)
  });
}

/**
 * Shorthand for PUT requests
 */
export async function put(endpoint, data) {
  return apiCall(endpoint, {
    method: 'PUT',
    body: JSON.stringify(data)
  });
}

/**
 * Shorthand for DELETE requests
 */
export async function del(endpoint) {
  return apiCall(endpoint, {
    method: 'DELETE'
  });
} 