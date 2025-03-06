<script>
  import { onMount } from 'svelte';
  import { auth } from '../stores/auth.js';
  
  let userData = {
    id: '',
    username: '',
    role: ''
  };
  let isLoading = true;
  let error = null;
  
  onMount(async () => {
    auth.subscribe(value => {
      if (!value.isAuthenticated) {
        return;
      }
      
      fetchProfile(value.token);
    });
  });
  
  async function fetchProfile(token) {
    try {
      isLoading = true;
      error = null;
      
      const response = await fetch('http://localhost:8443/auth/profile', {
        headers: {
          'Authorization': `Bearer ${token}`
        }
      });
      
      if (!response.ok) {
        throw new Error(`Error fetching profile: ${response.status}`);
      }
      
      userData = await response.json();
      isLoading = false;
    } catch (err) {
      console.error('Profile fetch error:', err);
      error = err.message;
      isLoading = false;
    }
  }
  
  async function refreshToken() {
    try {
      const token = auth.getToken();
      if (!token) throw new Error('Not authenticated');
      
      const response = await fetch('http://localhost:8443/auth/refresh', {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`
        }
      });
      
      if (!response.ok) {
        throw new Error(`Failed to refresh token: ${response.status}`);
      }
      
      const data = await response.json();
      auth.updateToken(data.token);
      
      // Show success message
      alert('Token refreshed successfully');
    } catch (err) {
      console.error('Token refresh error:', err);
      error = err.message;
    }
  }
</script>

<div class="user-profile">
  <h1>User Profile</h1>
  
  {#if isLoading}
    <div class="loading">Loading profile data...</div>
  {:else if error}
    <div class="error">
      <p>{error}</p>
      <button on:click={() => auth.logout()}>Return to Login</button>
    </div>
  {:else}
    <div class="profile-card">
      <div class="profile-header">
        <div class="avatar">
          {userData.username?.charAt(0).toUpperCase() || 'U'}
        </div>
        <h2>{userData.username || 'User'}</h2>
        <span class="role-badge">{userData.role || 'user'}</span>
      </div>
      
      <div class="profile-details">
        <div class="detail-row">
          <span class="label">User ID:</span>
          <span class="value">{userData.id || 'Unknown'}</span>
        </div>
        
        <div class="detail-row">
          <span class="label">Username:</span>
          <span class="value">{userData.username || 'Unknown'}</span>
        </div>
        
        <div class="detail-row">
          <span class="label">Role:</span>
          <span class="value">{userData.role || 'user'}</span>
        </div>
      </div>
      
      <div class="actions">
        <button class="refresh-token" on:click={refreshToken}>
          Refresh Token
        </button>
        <button class="logout" on:click={() => auth.logout()}>
          Logout
        </button>
      </div>
    </div>
  {/if}
</div>

<style>
  .user-profile {
    max-width: 800px;
    margin: 0 auto;
    padding: 2rem;
  }
  
  h1 {
    color: #333;
    margin-bottom: 2rem;
  }
  
  .loading {
    display: flex;
    justify-content: center;
    align-items: center;
    height: 200px;
    font-size: 1.2rem;
    color: #666;
  }
  
  .error {
    background-color: #fff9fa;
    border: 1px solid #ffcdd2;
    border-radius: 8px;
    padding: 1.5rem;
    color: #d32f2f;
    text-align: center;
  }
  
  .profile-card {
    background-color: white;
    border-radius: 8px;
    box-shadow: 0 2px 8px rgba(0,0,0,0.1);
    overflow: hidden;
  }
  
  .profile-header {
    background: linear-gradient(135deg, #2196F3, #1976D2);
    color: white;
    padding: 2rem;
    text-align: center;
    position: relative;
  }
  
  .avatar {
    width: 80px;
    height: 80px;
    background-color: white;
    color: #1976D2;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 2rem;
    font-weight: bold;
    margin: 0 auto 1rem;
  }
  
  .role-badge {
    background-color: rgba(255,255,255,0.2);
    padding: 0.25rem 0.75rem;
    border-radius: 1rem;
    font-size: 0.875rem;
  }
  
  .profile-details {
    padding: 2rem;
  }
  
  .detail-row {
    display: flex;
    margin-bottom: 1rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid #eee;
  }
  
  .detail-row:last-child {
    border-bottom: none;
  }
  
  .label {
    font-weight: bold;
    width: 30%;
    color: #666;
  }
  
  .value {
    width: 70%;
  }
  
  .actions {
    display: flex;
    justify-content: space-between;
    padding: 1.5rem 2rem;
    background-color: #f9f9f9;
  }
  
  button {
    padding: 0.5rem 1rem;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-weight: bold;
  }
  
  .refresh-token {
    background-color: #2196F3;
    color: white;
  }
  
  .logout {
    background-color: #f44336;
    color: white;
  }
</style> 