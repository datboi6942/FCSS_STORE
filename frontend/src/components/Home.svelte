<script>
    export let setView;
  </script>
  
  <section class="hero">
    <h1>Enhance Your Security</h1>
    <p>High-Security Phones, Payment Gateways, and More!</p>
    <button on:click={() => setView('products')}>Get Started</button>
  </section>
  
  <section class="features">
    <h2>Our Products and Services</h2>
    
    <div class="feature-grid">
      <div class="feature-card">
        <h3>Secure Devices</h3>
        <p>Encrypted smartphones and tablets designed with privacy in mind.</p>
      </div>
      
      <div class="feature-card">
        <h3>Payment Solutions</h3>
        <p>Secure payment gateways with multi-factor authentication.</p>
      </div>
      
      <div class="feature-card">
        <h3>24/7 Support</h3>
        <p>Our security experts are available around the clock to assist you.</p>
      </div>
      
      <div class="feature-card">
        <h3>Data Protection</h3>
        <p>Advanced encryption protocols to keep your information safe.</p>
      </div>
    </div>
  </section>
  
  <style>
    .hero {
      background-color: #1e90ff;
      color: white;
      padding: 4rem 2rem;
      text-align: center;
      border-radius: 8px;
      margin-bottom: 2rem;
    }
  
    .hero h1 {
      font-size: 2.5rem;
      margin-bottom: 1rem;
    }
  
    .hero p {
      font-size: 1.2rem;
      margin-bottom: 2rem;
    }
  
    .hero button {
      background-color: white;
      color: #1e90ff;
      border: none;
      padding: 0.75rem 1.5rem;
      border-radius: 5px;
      cursor: pointer;
      font-size: 1rem;
      font-weight: 600;
      transition: background-color 0.2s;
    }
  
    .hero button:hover {
      background-color: #f0f0f0;
    }
  
    .features {
      padding: 2rem 0;
    }
  
    .features h2 {
      text-align: center;
      margin-bottom: 2rem;
    }
  
    .feature-grid {
      display: grid;
      grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
      gap: 2rem;
    }
  
    .feature-card {
      background-color: white;
      padding: 1.5rem;
      border-radius: 8px;
      box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
    }
  
    .feature-card h3 {
      color: #1e90ff;
      margin-bottom: 1rem;
    }
  </style>