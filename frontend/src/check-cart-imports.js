// This is just a utility script - run with Node.js to check all Svelte files
// for incorrect imports

const fs = require('fs');
const path = require('path');

function scanDirectory(dir) {
  const files = fs.readdirSync(dir);
  
  files.forEach(file => {
    const filePath = path.join(dir, file);
    const stats = fs.statSync(filePath);
    
    if (stats.isDirectory()) {
      scanDirectory(filePath);
    } else if (file.endsWith('.svelte')) {
      const content = fs.readFileSync(filePath, 'utf8');
      if (content.includes('cartItems')) {
        console.log(`Found cartItems in: ${filePath}`);
      }
    }
  });
}

// Start scanning from the src directory
scanDirectory('./src'); 