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
      console.log("Logging in with user data:", userData);
      
      // Store token in both localStorage and sessionStorage for redundancy
      if (typeof window !== 'undefined') {
        localStorage.setItem('jwt', userData.token);
        sessionStorage.setItem('jwt', userData.token);
        localStorage.setItem('user', JSON.stringify({
          id: userData.user_id || userData.id,
          username: userData.username,
          role: userData.role
        }));
      }
      
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
      sessionStorage.removeItem('jwt');
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
      const token = localStorage.getItem('jwt') || sessionStorage.getItem('jwt');
      const savedUser = localStorage.getItem('user');
      
      if (!token) {
        console.log("No token found in storage");
        return false;
      }
      
      try {
        console.log("Checking auth with token:", token.substring(0, 10) + "...");
        
        // First try to restore from saved user data
        if (savedUser) {
          const userData = JSON.parse(savedUser);
          update(state => ({
            ...state,
            isAuthenticated: true,
            token,
            user: userData,
            isAdmin: userData.role === 'admin'
          }));
        }
        
        // Verify with backend
        const response = await fetch('http://localhost:5000/auth/profile', {
          headers: {
            'Authorization': `Bearer ${token}`,
            'Accept': 'application/json',
            'Content-Type': 'application/json'
          }
        });
        
        if (!response.ok) {
          const errorText = await response.text();
          console.error("Auth check failed:", errorText);
          throw new Error('Invalid token');
        }
        
        const data = await response.json();
        console.log("Profile data:", data);
        
        // Update auth state with fresh data from server
        update(state => ({
          ...state,
          isAuthenticated: true,
          token,
          user: {
            id: data.id,
            username: data.username,
            role: data.role
          },
          isAdmin: data.role === 'admin'
        }));
        
        return true;
      } catch (error) {
        console.error('Auth check failed:', error);
        // Clear auth state on failure
        localStorage.removeItem('jwt');
        sessionStorage.removeItem('jwt');
        localStorage.removeItem('user');
        set(initialState);
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

// Update API URL
const API_URL = 'http://192.168.6.53:5000'; 