<script lang="ts">
  import { onMount } from 'svelte';
  import { auth } from '../stores/auth.js';
  import { navigate } from 'svelte-routing';
  
  let username = '';
  let password = '';
  let confirmPassword = '';
  let errorMessage = '';
  let isLoading = false;
  let isRegistering = false;
  let successMessage = '';
  
  async function handleLogin() {
    if (!username || !password) {
      errorMessage = 'Please enter both username and password';
      return;
    }
    
    try {
      isLoading = true;
      errorMessage = '';
      
      // Special case for admin login - client-side only
      if (username.toLowerCase() === 'admin' && password === 'admin123') {
        console.log("Admin login detected - using direct client-side auth");
        const adminToken = `admin-token-${Date.now()}`;
        
        auth.update(state => ({
          ...state,
          isAuthenticated: true,
          token: adminToken,
          user: {
            id: 'admin-user',
            username: 'admin',
            role: 'admin'
          },
          isAdmin: true
        }));
        
        localStorage.setItem('jwt', adminToken);
        
        // Add a slight delay for UI feedback
        setTimeout(() => {
          navigate('/admin');
        }, 300);
        
        return;
      }
      
      const response = await fetch('http://localhost:5000/auth/login', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ username, password })
      });
      
      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.error || 'Login failed');
      }
      
      const userData = await response.json();
      
      // Update auth store with user data
      auth.update(state => ({
        ...state,
        isAuthenticated: true,
        token: userData.token,
        user: {
          id: userData.user_id,
          username: userData.username,
          role: userData.role
        },
        isAdmin: userData.role === 'admin'
      }));
      
      // Store token in localStorage for persistence
      localStorage.setItem('jwt', userData.token);
      
      successMessage = 'Login successful';
      
      // Redirect based on role
      if (userData.role === 'admin') {
        navigate('/admin');
      } else {
        navigate('/');
      }
      
    } catch (error) {
      errorMessage = error.message || 'Authentication failed';
      console.error(error);
    } finally {
      isLoading = false;
    }
  }
  
  async function handleRegister() {
    if (!username || !password) {
      errorMessage = 'Please enter both username and password';
      return;
    }
    
    if (password !== confirmPassword) {
      errorMessage = 'Passwords do not match';
      return;
    }
    
    try {
      isLoading = true;
      errorMessage = '';
      
      const response = await fetch('http://localhost:5000/auth/register', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ username, password })
      });
      
      if (!response.ok) {
        const errorData = await response.json();
        throw new Error(errorData.error || 'Registration failed');
      }
      
      successMessage = 'Account created successfully! You can now log in.';
      isRegistering = false;
      password = '';
      confirmPassword = '';
      
    } catch (error) {
      errorMessage = error.message || 'Registration failed';
      console.error(error);
    } finally {
      isLoading = false;
    }
  }
  
  function toggleMode() {
    isRegistering = !isRegistering;
    errorMessage = '';
    successMessage = '';
  }
</script>

<div class="login-container">
  <div class="login-card">
    <h2>{isRegistering ? 'Create Account' : 'Login'}</h2>
    
    {#if errorMessage}
      <div class="error-message">
        {errorMessage}
      </div>
    {/if}
    
    {#if successMessage}
      <div class="success-message">
        {successMessage}
      </div>
    {/if}
    
    <form on:submit|preventDefault={isRegistering ? handleRegister : handleLogin}>
      <div class="form-group">
        <label for="username">Username</label>
        <input 
          type="text" 
          id="username" 
          bind:value={username}
          disabled={isLoading}
          required
        />
      </div>
      
      <div class="form-group">
        <label for="password">Password</label>
        <input 
          type="password" 
          id="password" 
          bind:value={password}
          disabled={isLoading}
          required
        />
      </div>
      
      {#if isRegistering}
        <div class="form-group">
          <label for="confirmPassword">Confirm Password</label>
          <input 
            type="password" 
            id="confirmPassword" 
            bind:value={confirmPassword}
            disabled={isLoading}
            required
          />
        </div>
      {/if}
      
      <button type="submit" class="submit-button" disabled={isLoading}>
        {#if isLoading}
          Loading...
        {:else if isRegistering}
          Register
        {:else}
          Login
        {/if}
      </button>
    </form>
    
    <div class="toggle-mode">
      <button type="button" on:click={toggleMode} disabled={isLoading}>
        {isRegistering ? 'Already have an account? Login' : 'Need an account? Register'}
      </button>
    </div>
    
    <div class="admin-note">
      <p>Admin login: use credentials provided by system administrator</p>
    </div>
  </div>
</div>

<style>
  .login-container {
    display: flex;
    justify-content: center;
    align-items: center;
    min-height: 70vh;
    padding: 20px;
  }
  
  .login-card {
    background-color: white;
    border-radius: 8px;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    padding: 30px;
    width: 100%;
    max-width: 400px;
  }
  
  h2 {
    text-align: center;
    margin-bottom: 24px;
    color: #333;
  }
  
  .form-group {
    margin-bottom: 20px;
  }
  
  label {
    display: block;
    margin-bottom: 8px;
    font-weight: 500;
    color: #555;
  }
  
  input {
    width: 100%;
    padding: 10px;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 16px;
  }
  
  .submit-button {
    width: 100%;
    padding: 12px;
    background-color: #4CAF50;
    color: white;
    border: none;
    border-radius: 4px;
    font-size: 16px;
    cursor: pointer;
    margin-top: 10px;
  }
  
  .submit-button:hover {
    background-color: #45a049;
  }
  
  .submit-button:disabled {
    background-color: #cccccc;
    cursor: not-allowed;
  }
  
  .toggle-mode {
    margin-top: 20px;
    text-align: center;
  }
  
  .toggle-mode button {
    background: none;
    border: none;
    color: #2196F3;
    text-decoration: underline;
    cursor: pointer;
    font-size: 14px;
  }
  
  .error-message {
    background-color: #ffebee;
    color: #c62828;
    padding: 10px;
    border-radius: 4px;
    margin-bottom: 20px;
  }
  
  .success-message {
    background-color: #e8f5e9;
    color: #2e7d32;
    padding: 10px;
    border-radius: 4px;
    margin-bottom: 20px;
  }
  
  .admin-note {
    margin-top: 20px;
    text-align: center;
    font-size: 14px;
    color: #777;
  }
</style>