import { auth } from './stores/auth';
import { get } from 'svelte/store';

const API_URL = 'http://localhost:8443';

export async function apiCall(endpoint, options = {}) {
    const { token } = get(auth);
    
    const headers = {
        'Content-Type': 'application/json',
        ...options.headers
    };
    
    if (token) {
        headers['Authorization'] = `Bearer ${token}`;
    }
    
    const response = await fetch(`${API_URL}${endpoint}`, {
        ...options,
        headers
    });
    
    // Handle 401 Unauthorized - automatic logout
    if (response.status === 401) {
        auth.logout();
        throw new Error('Session expired. Please login again.');
    }
    
    return response;
}
