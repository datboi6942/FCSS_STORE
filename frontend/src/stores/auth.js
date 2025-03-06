import { writable } from 'svelte/store';

// Initial state
const initialState = {
  isAuthenticated: false,
  user: null,
  isAdmin: false
};

// Create the writable store
const authStore = writable(initialState);

// Store actions
export const login = (userData) => {
  authStore.update(() => ({
    isAuthenticated: true,
    user: userData,
    isAdmin: userData.role === 'admin'
  }));
};

export const logout = () => {
  authStore.set(initialState);
};

export default authStore; 