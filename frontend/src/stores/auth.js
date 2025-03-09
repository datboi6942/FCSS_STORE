import { writable } from 'svelte/store';

// Initial auth state
const initialState = {
  isAuthenticated: false,
  token: null,
  user: null,
  isAdmin: false
};

// Create the store
const createAuthStore = () => {
  const { subscribe, set, update } = writable(initialState);

  // Initialize from localStorage
  if (typeof window !== 'undefined') {
    const token = localStorage.getItem('jwt');
    if (token) {
      // We'll validate this token on app start
      update(state => ({
        ...state,
        isAuthenticated: true,
        token
      }));
    }
  }

  return {
    subscribe,
    
    // Set entire auth state
    set,
    
    // Update partial auth state
    update,
    
    // Login handler
    login: (userData) => {
      update(state => ({
        ...state,
        isAuthenticated: true,
        token: userData.token,
        user: {
          id: userData.user_id || userData.id,
          username: userData.username,
          role: userData.role
        },
        isAdmin: userData.role === 'admin'
      }));
      
      // Store token in localStorage
      if (typeof window !== 'undefined') {
        localStorage.setItem('jwt', userData.token);
      }
    },
    
    // Logout handler
    logout: () => {
      // Check if we're in the checkout flow - don't logout during checkout
      if (typeof window !== 'undefined' && 
          window.location.pathname.includes('/checkout/')) {
        console.log("Preventing logout during checkout flow");
        return;
      }
      
      localStorage.removeItem('jwt');
      // Only remove auth_token_backup if not in checkout flow
      if (!window.location.pathname.includes('/checkout/')) {
        localStorage.removeItem('auth_token_backup');
        localStorage.removeItem('auth_user_backup');
      }
      
      console.log('Auth store: Logging out user');
      
      // Reset to initial state
      set(initialState);
    },
    
    // Check if user is authenticated and fetch profile
    checkAuth: async () => {
      const token = localStorage.getItem('jwt') || localStorage.getItem('auth_token_backup');
      if (!token) {
        console.log("No token found in localStorage");
        return false;
      }
      
      try {
        console.log("Checking auth with token:", token.substring(0, 10) + "...");
        
        // First check if the token is valid using our debug endpoint
        const tokenCheckResponse = await fetch('http://localhost:5000/orders/debug-token', {
          headers: {
            'Authorization': `Bearer ${token}`
          }
        });
        
        const tokenData = await tokenCheckResponse.json();
        console.log("Token check result:", tokenData);
        
        if (!tokenCheckResponse.ok || !tokenData.success) {
          console.error("Token is invalid:", tokenData.error);
          throw new Error(tokenData.error || 'Invalid token');
        }
        
        // If token is valid, fetch the user profile
        const profileResponse = await fetch('http://localhost:5000/auth/profile', {
          headers: {
            'Authorization': `Bearer ${token}`
          }
        });
        
        if (!profileResponse.ok) {
          console.error("Profile fetch failed with status:", profileResponse.status);
          const errorText = await profileResponse.text();
          console.error("Error response:", errorText);
          throw new Error('Failed to fetch profile');
        }
        
        const profileData = await profileResponse.json();
        console.log("Profile data:", profileData);
        
        if (profileData.id) {
          update(state => ({
            ...state,
            isAuthenticated: true,
            token,
            user: {
              id: profileData.id,
              username: profileData.username,
              role: profileData.role
            },
            isAdmin: profileData.role === 'admin'
          }));
          return true;
        } else {
          throw new Error('Invalid user data');
        }
      } catch (error) {
        console.error('Auth check failed:', error);
        
        // Don't clear tokens during checkout
        if (!window.location.pathname.includes('/checkout/')) {
          localStorage.removeItem('jwt');
          localStorage.removeItem('auth_token_backup');
          set(initialState);
        }
        return false;
      }
    }
  };
};

// Create and export the store
const authStore = createAuthStore();

// Named export for destructured imports: import { auth } from '../stores/auth';  
export const auth = authStore;

// Default export for: import auth from '../stores/auth';
export default authStore;

// Convenience exports
export const login = authStore.login;
export const logout = authStore.logout;
export const checkAuth = authStore.checkAuth;

// Helper function for debouncing operations
function debounce(func, timeout = 300) {
  let timer;
  return (...args) => {
    clearTimeout(timer);
    timer = setTimeout(() => { 
      if (typeof func === 'function') {
        func.apply(this, args);
      }
    }, timeout);
  };
}

// Update API URL to port 5000
const API_URL = 'http://localhost:5000'; 