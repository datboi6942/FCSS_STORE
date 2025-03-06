import { writable } from 'svelte/store';

// Create an initial auth state
const initialState = {
  isAuthenticated: false,
  token: null,
  user: null
};

// Check if we have existing auth in localStorage
function getInitialState() {
  if (typeof localStorage !== 'undefined') {
    const storedToken = localStorage.getItem('authToken');
    if (storedToken) {
      return {
        isAuthenticated: true,
        token: storedToken,
        user: JSON.parse(localStorage.getItem('authUser') || '{}')
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
      localStorage.setItem('authToken', userData.token);
      localStorage.setItem('authUser', JSON.stringify({
        id: userData.id,
        username: userData.username,
        role: userData.role
      }));
      
      set({
        isAuthenticated: true,
        token: userData.token,
        user: {
          id: userData.id,
          username: userData.username,
          role: userData.role
        }
      });
    },
    logout: () => {
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

// For compatibility with older code, add these exports
export function login(userData) {
  auth.login(userData);
}

export function logout() {
  auth.logout();
} 