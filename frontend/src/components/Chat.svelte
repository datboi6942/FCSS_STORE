<script lang="ts">
    import { onMount, onDestroy } from 'svelte';
    let messages = [];
    let inputMessage = '';
    let socket;
    let username = '';
    let usernameInput = 'testuser'; // Default value
    let isConnected = false;
    let connecting = false;
    let error = null;
    let message = '';
    let chatHistory: any[] = [];
  
    function connect() {
      if (!usernameInput.trim()) {
        error = "Username cannot be empty";
        return;
      }
  
      error = null;
      connecting = true;
      username = usernameInput;
      const wsUrl = `ws://127.0.0.1:8443/chat?username=${username}`;
      
      socket = new WebSocket(wsUrl);
      
      socket.onopen = () => {
        console.log("Connected to chat");
        isConnected = true;
        connecting = false;
      };
      
      socket.onmessage = (event) => {
        let data;
        try {
          data = JSON.parse(event.data);
        } catch (e) {
          data = event.data;
        }
        
        if (Array.isArray(data)) {
          messages = data; // initial history
        } else {
          messages = [...messages, data];
        }
      };
      
      socket.onclose = () => {
        console.log("Disconnected from chat");
        isConnected = false;
      };
      
      socket.onerror = (e) => {
        console.error("WebSocket error:", e);
        error = "Failed to connect to chat server";
        connecting = false;
        isConnected = false;
      };
    }
  
    function disconnect() {
      if (socket) {
        socket.close();
        isConnected = false;
      }
    }
  
    function sendMessage() {
      if (!message.trim()) return;
      
      // Add user message to chat
      chatHistory = [...chatHistory, {
        sender: 'You',
        text: message,
        time: new Date()
      }];
      
      // Simulate response
      setTimeout(() => {
        chatHistory = [...chatHistory, {
          sender: 'Support',
          text: 'Thank you for your message. A security specialist will respond shortly.',
          time: new Date()
        }];
      }, 1000);
      
      // Clear input
      message = '';
    }
  
    function handleKeyPress(event) {
      if (event.key === 'Enter' && !event.shiftKey) {
        event.preventDefault();
        sendMessage();
      }
    }
  
    onMount(() => {
      connect();
    });
  
    onDestroy(() => {
      if (socket) {
        socket.close();
      }
    });
  </script>
  

   