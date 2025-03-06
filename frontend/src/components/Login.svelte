<script lang="ts">
  import { onMount } from 'svelte';
  import authStore, { login, logout } from '../stores/auth.js';
  
  let username = '';
  let password = '';
  let confirmPassword = '';
  let errorMessage = '';
  let isLoading = false;
  let isRegistering = false;
  let successMessage = '';
  let adminPassword = '';
  
  // Check if the connection is from localhost
  const isLocalhost = 
    window.location.hostname === 'localhost' || 
    window.location.hostname === '127.0.0.1';
    
  onMount(() => {
    console.log("Login component mounted");
    // Get the admin password from the config file if available
    if (window.secureStoreConfig && window.secureStoreConfig.adminPassword) {
      adminPassword = window.secureStoreConfig.adminPassword;
      console.log("Admin config loaded successfully");
    } else {
      console.error("Admin config not found!");
    }
  });
    
  async function handleLogin() {
    if (!username || !password) {
      errorMessage = 'Please enter both username and password';
      return;
    }
    
    try {
      isLoading = true;
      errorMessage = '';
      
      // Special admin login check with password from config
      if (username === 'admin' && password === adminPassword && isLocalhost) {
        console.log("Admin login matched - logging in as admin");
        
        // Use a fixed admin user object that includes all necessary fields
        const adminUser = {
          username: 'admin',
          id: 'admin-user',
          role: 'admin',
          token: 'admin-token-' + Date.now(),
          isAdmin: true  // Explicitly set this flag
        };
        
        // Call the login function
        login(adminUser);
        successMessage = 'Logged in as administrator';
        
        // Redirect to admin panel
        setTimeout(() => {
          window.location.href = '/admin';
        }, 1000);
        
        return;
      }
      
      // Regular user login
      const response = await fetch('http://127.0.0.1:8443/auth/login', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ username, password })
      });
      
      if (!response.ok) {
        throw new Error('Login failed');
      }
      
      const userData = await response.json();
      
      // Prevent non-localhost connections from getting admin role
      if (userData.role === 'admin' && !isLocalhost) {
        throw new Error('Admin login restricted to localhost only');
      }
      
      login(userData);
      successMessage = 'Login successful';
      
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
      
      const response = await fetch('http://127.0.0.1:8443/auth/register', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json'
        },
        body: JSON.stringify({ username, password })
      });
      
      if (!response.ok) {
        throw new Error('Registration failed');
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
  <h2>{isRegistering ? 'Create Account' : 'Login'}</h2>
  
  {#if $authStore.isAuthenticated}
    <div class="welcome-message">
      <p>Welcome, {$authStore.user.username}!</p>
      <p>You are logged in as {$authStore.isAdmin ? 'Administrator' : 'User'}</p>
      <button on:click={logout}>Logout</button>
    </div>
  {:else}
    {#if successMessage}
      <p class="success">{successMessage}</p>
    {/if}
    
    <form on:submit|preventDefault={isRegistering ? handleRegister : handleLogin}>
      <div class="form-group">
        <label for="username">Username</label>
        <input 
          type="text" 
          id="username" 
          bind:value={username} 
          disabled={isLoading}
        />
      </div>
      
      <div class="form-group">
        <label for="password">Password</label>
        <input 
          type="password" 
          id="password" 
          bind:value={password} 
          disabled={isLoading}
        />
      </div>
      
      {#if isRegistering}
        <div class="form-group">
          <label for="confirm-password">Confirm Password</label>
          <input 
            type="password" 
            id="confirm-password" 
            bind:value={confirmPassword} 
            disabled={isLoading}
          />
        </div>
      {/if}
      
      {#if errorMessage}
        <p class="error">{errorMessage}</p>
      {/if}
      
      <button type="submit" disabled={isLoading}>
        {#if isLoading}
          {isRegistering ? 'Creating Account...' : 'Logging in...'}
        {:else}
          {isRegistering ? 'Create Account' : 'Login'}
        {/if}
      </button>
      
      <p class="toggle-mode">
        {isRegistering ? 'Already have an account?' : 'Need an account?'}
        <button type="button" class="link-button" on:click={toggleMode}>
          {isRegistering ? 'Login' : 'Register'}
        </button>
      </p>
    </form>
  {/if}
  
  {#if isLocalhost && !$authStore.isAuthenticated}
    <div class="admin-note">
      <p>Admin login available on localhost</p>
    </div>
  {/if}
</div>

<style>
  .login-container {
    max-width: 400px;
    margin: 0 auto;
    padding: 2rem;
    background-color: white;
    border-radius: 8px;
    box-shadow: 0 2px 4px rgba(0,0,0,0.1);
  }
  
  .form-group {
    margin-bottom: 1rem;
  }
  
  label {
    display: block;
    margin-bottom: 0.5rem;
    font-weight: 500;
  }
  
  input {
    width: 100%;
    padding: 0.75rem;
    border: 1px solid #ddd;
    border-radius: 4px;
  }
  
  button {
    background-color: #1e90ff;
    color: white;
    border: none;
    padding: 0.75rem 1rem;
    border-radius: 4px;
    cursor: pointer;
    width: 100%;
  }
  
  button:hover {
    background-color: #167edb;
  }
  
  button:disabled {
    background-color: #b3d1ff;
    cursor: not-allowed;
  }
  
  .error {
    color: #e74c3c;
    background-color: #fadbd8;
    padding: 0.5rem;
    border-radius: 4px;
    margin-bottom: 1rem;
  }
  
  .success {
    color: #2ecc71;
    background-color: #d4edda;
    padding: 0.5rem;
    border-radius: 4px;
    margin-bottom: 1rem;
  }
  
  .welcome-message {
    text-align: center;
  }
  
  .toggle-mode {
    text-align: center;
    margin-top: 1rem;
    font-size: 0.9rem;
  }
  
  .link-button {
    background: none;
    color: #1e90ff;
    border: none;
    padding: 0;
    font: inherit;
    text-decoration: underline;
    cursor: pointer;
    width: auto;
    display: inline;
  }
  
  .link-button:hover {
    background: none;
    color: #167edb;
  }
  
  .admin-note {
    margin-top: 1rem;
    text-align: center;
    font-size: 0.8rem;
    color: #777;
    border-top: 1px solid #eee;
    padding-top: 1rem;
  }
</style> 