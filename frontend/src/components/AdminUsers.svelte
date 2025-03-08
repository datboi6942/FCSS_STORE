<script>
  import { onMount } from 'svelte';
  import { auth } from '../stores/auth.js';

  let users = [];
  let loading = true;
  let error = null;
  let success = null;
  let newUser = { username: '', password: '', role: 'user' };
  let editingUser = null;
  let showForm = false;

  // Load users on mount
  onMount(fetchUsers);

  async function fetchUsers() {
    try {
      loading = true;
      error = null;
      success = null;
      console.log("Fetching users with token:", $auth.token);
      
      // Change the endpoint URL to match what's defined in admin.rs
      const response = await fetch('http://localhost:5000/admin/users', {
        headers: {
          'Authorization': `Bearer ${$auth.token}`
        }
      });

      if (!response.ok) {
        const errorText = await response.text();
        console.error("Error response:", errorText);
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const data = await response.json();
      console.log("Users data received:", data);
      
      // Handle both formats: array or {success: true, users: [...]}
      if (Array.isArray(data)) {
        users = data;
      } else if (data.success && Array.isArray(data.users)) {
        users = data.users;
      } else {
        throw new Error('Invalid response format');
      }
      
      console.log('Fetched users:', users);
    } catch (e) {
      error = e.message;
      console.error('Error fetching users:', e);
    } finally {
      loading = false;
    }
  }

  async function handleSubmit() {
    try {
      loading = true;
      error = null;
      success = null;
      
      const method = editingUser ? 'PUT' : 'POST';
      const url = editingUser 
        ? `http://localhost:5000/admin/users/${editingUser.id}` 
        : 'http://localhost:5000/admin/users';
      
      const userData = editingUser 
        ? { ...newUser } 
        : newUser;

      // If password is empty and we're editing, remove it
      if (editingUser && (!userData.password || userData.password.trim() === '')) {
        delete userData.password;
      }

      console.log('Submitting user data:', userData);

      const response = await fetch(url, {
        method,
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${$auth.token}`
        },
        body: JSON.stringify(userData)
      });

      const data = await response.json();
      
      if (!response.ok) {
        throw new Error(data.error || `Failed with status: ${response.status}`);
      }

      if (!data.success) {
        throw new Error(data.error || 'Operation failed');
      }

      // Show success message
      success = data.message || (editingUser ? 'User updated successfully' : 'User created successfully');
      
      // Refresh users
      await fetchUsers();
      
      // Reset form
      newUser = { username: '', password: '', role: 'user' };
      editingUser = null;
      showForm = false;
    } catch (e) {
      error = e.message;
      console.error('Error saving user:', e);
    } finally {
      loading = false;
    }
  }

  function editUser(user) {
    editingUser = user;
    newUser = { 
      username: user.username, 
      password: '', // Don't populate password
      role: user.role
    };
    showForm = true;
    // Clear messages
    error = null;
    success = null;
  }

  async function deleteUser(id) {
    if (!confirm('Are you sure you want to delete this user?')) {
      return;
    }

    try {
      loading = true;
      error = null;
      success = null;
      
      const response = await fetch(`http://localhost:5000/admin/users/${id}`, {
        method: 'DELETE',
        headers: {
          'Authorization': `Bearer ${$auth.token}`
        }
      });

      const data = await response.json();
      
      if (!response.ok) {
        throw new Error(data.error || `Failed with status: ${response.status}`);
      }

      if (!data.success) {
        throw new Error(data.error || 'Failed to delete user');
      }

      // Show success message
      success = data.message || 'User deleted successfully';
      
      // Refresh users
      await fetchUsers();
    } catch (e) {
      error = e.message;
      console.error('Error deleting user:', e);
    } finally {
      loading = false;
    }
  }
</script>

<div class="admin-users">
  <h2>Manage Users</h2>
  
  <div class="actions">
    <button class="action-btn" on:click={() => {
      editingUser = null;
      newUser = { username: '', password: '', role: 'user' };
      showForm = !showForm;
      error = null;
      success = null;
    }}>
      {showForm ? 'Cancel' : 'Add New User'}
    </button>
    <button class="refresh-btn" on:click={fetchUsers}>Refresh</button>
  </div>
  
  {#if error}
    <div class="error-message">
      <strong>Error:</strong> {error}
    </div>
  {/if}
  
  {#if success}
    <div class="success-message">
      <strong>Success:</strong> {success}
    </div>
  {/if}
  
  {#if showForm}
    <div class="user-form">
      <h3>{editingUser ? 'Edit User' : 'Add New User'}</h3>
      <form on:submit|preventDefault={handleSubmit}>
        <div class="form-group">
          <label for="username">Username</label>
          <input id="username" bind:value={newUser.username} required />
        </div>
        
        <div class="form-group">
          <label for="password">
            {editingUser ? 'Password (leave blank to keep current)' : 'Password'}
          </label>
          <input 
            type="password" 
            id="password" 
            bind:value={newUser.password} 
            required={!editingUser} 
          />
        </div>
        
        <div class="form-group">
          <label for="role">Role</label>
          <select id="role" bind:value={newUser.role}>
            <option value="user">User</option>
            <option value="admin">Admin</option>
          </select>
        </div>
        
        <div class="form-actions">
          <button type="button" class="cancel-btn" on:click={() => {
            showForm = false;
            editingUser = null;
          }}>
            Cancel
          </button>
          <button type="submit" class="submit-btn">
            {editingUser ? 'Update User' : 'Add User'}
          </button>
        </div>
      </form>
    </div>
  {/if}
  
  {#if loading}
    <div class="loading">Loading users...</div>
  {:else if users.length === 0}
    <div class="no-data">No users found. Create one using the "Add New User" button.</div>
  {:else}
    <table class="users-table">
      <thead>
        <tr>
          <th>ID</th>
          <th>Username</th>
          <th>Role</th>
          <th>Created</th>
          <th>Actions</th>
        </tr>
      </thead>
      <tbody>
        {#each users as user (user.id)}
          <tr>
            <td class="user-id">{user.id}</td>
            <td>{user.username}</td>
            <td>
              <span class="role-badge role-{user.role}">
                {user.role === 'admin' ? 'Administrator' : 'Regular User'}
              </span>
            </td>
            <td>{new Date(user.created_at * 1000).toLocaleString()}</td>
            <td class="actions">
              <button class="edit-btn" on:click={() => editUser(user)}>
                Edit
              </button>
              <button 
                class="delete-btn" 
                on:click={() => deleteUser(user.id)}
                disabled={user.id === $auth.user?.id}
                title="You cannot delete your own account"
              >
                Delete
              </button>
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</div>

<style>
  .admin-users {
    padding: 20px;
    max-width: 1200px;
    margin: 0 auto;
  }

  h2 {
    color: #2c3e50;
    margin-bottom: 20px;
    border-bottom: 2px solid #f1f1f1;
    padding-bottom: 10px;
  }

  .actions {
    display: flex;
    gap: 10px;
    margin-bottom: 20px;
  }

  .action-btn, .refresh-btn {
    padding: 10px 15px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-weight: bold;
  }

  .action-btn {
    background-color: #27ae60;
    color: white;
  }

  .action-btn:hover {
    background-color: #2ecc71;
  }

  .refresh-btn {
    background-color: #3498db;
    color: white;
  }

  .refresh-btn:hover {
    background-color: #2980b9;
  }

  .users-table {
    width: 100%;
    border-collapse: collapse;
    margin-top: 20px;
    box-shadow: 0 2px 5px rgba(0,0,0,0.1);
  }

  th, td {
    padding: 12px 15px;
    text-align: left;
    border-bottom: 1px solid #ddd;
  }

  th {
    background-color: #f8f9fa;
    font-weight: bold;
    color: #333;
  }

  tr:hover {
    background-color: #f5f5f5;
  }

  .user-id {
    font-family: monospace;
    font-size: 0.9em;
    color: #666;
  }

  .role-badge {
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 0.9em;
    font-weight: bold;
    display: inline-block;
  }

  .role-admin {
    background-color: #e74c3c;
    color: white;
  }

  .role-user {
    background-color: #3498db;
    color: white;
  }

  .actions {
    display: flex;
    gap: 8px;
  }

  .actions button {
    padding: 6px 10px;
    border: none;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.9em;
  }

  .edit-btn {
    background-color: #f39c12;
    color: white;
  }

  .edit-btn:hover {
    background-color: #e67e22;
  }

  .delete-btn {
    background-color: #e74c3c;
    color: white;
  }

  .delete-btn:hover:not(:disabled) {
    background-color: #c0392b;
  }

  .delete-btn:disabled {
    background-color: #bdc3c7;
    cursor: not-allowed;
  }

  .user-form {
    background-color: #f8f9fa;
    padding: 20px;
    border-radius: 8px;
    margin-bottom: 20px;
    box-shadow: 0 2px 5px rgba(0,0,0,0.1);
  }

  .user-form h3 {
    margin-top: 0;
    color: #2c3e50;
    border-bottom: 1px solid #eee;
    padding-bottom: 10px;
    margin-bottom: 20px;
  }

  .form-group {
    margin-bottom: 15px;
  }

  .form-group label {
    display: block;
    margin-bottom: 5px;
    font-weight: bold;
    color: #333;
  }

  .form-group input, .form-group select {
    width: 100%;
    padding: 10px;
    border: 1px solid #ddd;
    border-radius: 4px;
    font-size: 16px;
  }

  .form-group select {
    background-color: white;
  }

  .form-actions {
    display: flex;
    justify-content: flex-end;
    gap: 10px;
    margin-top: 20px;
  }

  .cancel-btn {
    background-color: #95a5a6;
    color: white;
    border: none;
    padding: 10px 15px;
    border-radius: 4px;
    cursor: pointer;
  }

  .cancel-btn:hover {
    background-color: #7f8c8d;
  }

  .submit-btn {
    background-color: #2ecc71;
    color: white;
    border: none;
    padding: 10px 15px;
    border-radius: 4px;
    cursor: pointer;
    font-weight: bold;
  }

  .submit-btn:hover {
    background-color: #27ae60;
  }

  .error-message {
    background-color: #f8d7da;
    color: #721c24;
    padding: 10px;
    border-radius: 4px;
    margin-bottom: 15px;
    border-left: 4px solid #dc3545;
  }

  .success-message {
    background-color: #d4edda;
    color: #155724;
    padding: 10px;
    border-radius: 4px;
    margin-bottom: 15px;
    border-left: 4px solid #28a745;
  }

  .loading {
    text-align: center;
    padding: 20px;
    color: #666;
  }

  .no-data {
    text-align: center;
    padding: 30px;
    color: #666;
    background-color: #f8f9fa;
    border-radius: 4px;
    margin-top: 20px;
  }
</style> 