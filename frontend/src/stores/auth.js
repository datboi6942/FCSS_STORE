import { writable } from 'svelte/store';

// Create an initial auth state
const initialState = {
  isAuthenticated: false,
  token: null,
  user: null,
  isAdmin: false  // Added explicit isAdmin flag
};

// Check if we have existing auth in localStorage
function getInitialState() {
  if (typeof localStorage !== 'undefined') {
    const storedToken = localStorage.getItem('authToken');
    if (storedToken) {
      const user = JSON.parse(localStorage.getItem('authUser') || '{}');
      return {
        isAuthenticated: true,
        token: storedToken,
        user: user,
        isAdmin: user.role === 'admin'  // Set isAdmin based on role
      };
    }
  }
  return initialState;
}

// Create the store
const createAuthStore = () => {
  const { subscribe, set, update } = writable(getInitialState());
  
  return {
    subscribe,
    login: (userData) => {
      console.log("Auth store: Logging in user", userData);
      
      // Save to localStorage
      localStorage.setItem('authToken', userData.token);
      
      const userToStore = {
        id: userData.id,
        username: userData.username,
        role: userData.role
      };
      
      localStorage.setItem('authUser', JSON.stringify(userToStore));
      
      // Check if user is admin
      const isAdmin = userData.role === 'admin';
      console.log("Auth store: User is admin?", isAdmin);
      
      // Update store
      set({
        isAuthenticated: true,
        token: userData.token,
        user: userToStore,
        isAdmin: isAdmin
      });
      
      console.log("Auth store: Login complete");
    },
    logout: () => {
      console.log("Auth store: Logging out user");
      localStorage.removeItem('authToken');
      localStorage.removeItem('authUser');
      set(initialState);
    },
    updateToken: (newToken) => {
      update(state => {
        localStorage.setItem('authToken', newToken);
        return { 
          ...state, 
          token: newToken 
        };
      });
    },
    getToken: () => {
      return localStorage.getItem('authToken');
    }
  };
};

// Export the store
export const auth = createAuthStore();
export default auth;

// For compatibility with older code
export function login(userData) {
  console.log("Login function called with", userData);
  auth.login(userData);
}

export function logout() {
  console.log("Logout function called");
  auth.logout();
}

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