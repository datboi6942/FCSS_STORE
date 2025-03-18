<script>
  import { config } from '../config.js';
  import { onMount } from 'svelte';
  
  let configString = '';
  let error = null;
  let testResults = {};
  
  onMount(() => {
    try {
      // Format config object for display
      configString = JSON.stringify(config, null, 2);
      
      // Run basic endpoint tests
      testEndpoints();
    } catch (err) {
      error = err.message;
    }
  });
  
  async function testEndpoints() {
    // Test API health endpoint
    try {
      const response = await fetch(config.api.health, {
        signal: AbortSignal.timeout(5000)
      });
      testResults.health = {
        success: response.ok,
        status: response.status,
        statusText: response.statusText
      };
    } catch (err) {
      testResults.health = {
        success: false,
        error: err.message
      };
    }
    
    // Force update
    testResults = {...testResults};
  }
</script>

<div class="config-tester">
  <h2>Config Tester</h2>
  
  {#if error}
    <div class="error">
      Error loading config: {error}
    </div>
  {:else}
    <div class="config-display">
      <h3>Current Configuration:</h3>
      <pre>{configString}</pre>
      
      <h3>Environment: {config.app.environment}</h3>
      <p>API Host: {config.api.base}</p>
      <p>WebSocket Host: {config.ws.base}</p>
      
      <h3>Endpoint Tests:</h3>
      {#if testResults.health}
        <div class="test-result {testResults.health.success ? 'success' : 'failure'}">
          <strong>Health Endpoint:</strong> 
          {#if testResults.health.success}
            ✅ Success ({testResults.health.status})
          {:else}
            ❌ Failed - {testResults.health.error || testResults.health.statusText}
          {/if}
        </div>
      {:else}
        <p>Running tests...</p>
      {/if}
      
      <button on:click={testEndpoints}>Run Tests Again</button>
    </div>
  {/if}
</div>

<style>
  .config-tester {
    padding: 20px;
    background-color: #f5f5f5;
    border-radius: 8px;
    margin: 20px 0;
  }
  
  .error {
    color: red;
    padding: 10px;
    background-color: #ffeeee;
    border-radius: 4px;
  }
  
  pre {
    background-color: #eee;
    padding: 10px;
    border-radius: 4px;
    overflow-x: auto;
    max-height: 300px;
  }
  
  .test-result {
    padding: 10px;
    margin: 5px 0;
    border-radius: 4px;
  }
  
  .success {
    background-color: #e7f5e7;
  }
  
  .failure {
    background-color: #f5e7e7;
  }
  
  button {
    margin-top: 15px;
    padding: 8px 15px;
    background-color: #4CAF50;
    color: white;
    border: none;
    border-radius: 4px;
    cursor: pointer;
  }
  
  button:hover {
    background-color: #45a049;
  }
</style> 