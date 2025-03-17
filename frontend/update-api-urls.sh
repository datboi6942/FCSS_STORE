#!/bin/bash
# Script to update all hardcoded localhost:5000 URLs

# Create backup directory
mkdir -p backups

# Files to process
FILES=$(grep -r --include="*.svelte" --include="*.js" "localhost:5000" src | cut -d':' -f1 | sort | uniq)

echo "Files to update:"
echo "$FILES"
echo ""
echo "Press Enter to continue or Ctrl+C to cancel..."
read

for file in $FILES; do
  echo "Processing $file..."
  
  # Create backup
  cp "$file" "backups/$(basename "$file").bak"
  
  # Replace direct URLs with config imports
  sed -i '1s/^/import { config } from "../config.js";\n/' "$file"
  
  # Replace hardcoded URLs with config values
  sed -i 's|http://localhost:5000/products|config.api.products|g' "$file"
  sed -i 's|http://localhost:5000/auth|config.api.auth|g' "$file"
  sed -i 's|http://localhost:5000/orders|config.api.orders|g' "$file"
  sed -i 's|http://localhost:5000/admin|config.api.admin|g' "$file"
  sed -i 's|http://localhost:5000/monero|config.api.monero|g' "$file"
  sed -i 's|http://localhost:5000/payment|config.api.payment|g' "$file"
  sed -i 's|http://localhost:5000/cart|config.api.cart|g' "$file"
  sed -i 's|http://localhost:5000/health|config.api.health|g' "$file"
  
  # WebSocket URLs
  sed -i 's|ws://localhost:5000/ws/payment/\${order_id}|config.ws.payment(order_id)|g' "$file"
  
  # Generic base URLs
  sed -i 's|http://localhost:5000|config.api.base|g' "$file"
  
  echo "Updated $file"
done

echo "All files updated! Backups saved in 'backups' directory." 