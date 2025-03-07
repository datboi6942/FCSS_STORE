import { writable, derived } from 'svelte/store';

// Create the initial cart store
const createCart = () => {
  // Initialize cart from localStorage if available
  let initialCart = [];
  try {
    const savedCart = localStorage.getItem('cart');
    if (savedCart) {
      initialCart = JSON.parse(savedCart);
    }
  } catch (e) {
    console.error('Error loading cart from localStorage:', e);
    // Continue with empty cart if parsing fails
  }
  
  const { subscribe, set, update } = writable(initialCart);
  
  // Helper to save cart to localStorage
  const saveCart = (items) => {
    try {
      localStorage.setItem('cart', JSON.stringify(items));
    } catch (e) {
      console.error('Error saving cart to localStorage:', e);
    }
    return items;
  };
  
  return {
    subscribe,
    addItem: (item) => update(items => {
      // Check if item already exists
      const existingItem = items.find(i => i.id === item.id);
      
      let updatedItems;
      if (existingItem) {
        // Update quantity if item exists
        updatedItems = items.map(i => 
          i.id === item.id 
            ? { ...i, quantity: i.quantity + 1 }
            : i
        );
      } else {
        // Add new item with quantity 1
        updatedItems = [...items, { ...item, quantity: 1 }];
      }
      
      return saveCart(updatedItems);
    }),
    updateQuantity: (id, quantity) => update(items => {
      let updatedItems;
      if (quantity <= 0) {
        // Remove item if quantity is zero or negative
        updatedItems = items.filter(i => i.id !== id);
      } else {
        // Update quantity
        updatedItems = items.map(i => 
          i.id === id ? { ...i, quantity } : i
        );
      }
      
      return saveCart(updatedItems);
    }),
    removeItem: (id) => update(items => {
      const updatedItems = items.filter(i => i.id !== id);
      return saveCart(updatedItems);
    }),
    clear: () => {
      const emptyCart = [];
      saveCart(emptyCart);
      set(emptyCart);
    }
  };
};

// Create the cart store
export const cart = createCart();

// Derived stores for calculated values
export const cartTotal = derived(cart, $cart => 
  $cart ? $cart.reduce((total, item) => total + (item.price * item.quantity), 0) : 0
);

export const cartCount = derived(cart, $cart => 
  $cart ? $cart.reduce((count, item) => count + item.quantity, 0) : 0
); 