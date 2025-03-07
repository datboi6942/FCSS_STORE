import { writable, derived } from 'svelte/store';

// Create the cart store
function createCartStore() {
  // Get initial value from localStorage if it exists
  const storedCart = localStorage.getItem('cart');
  const initialItems = storedCart ? JSON.parse(storedCart) : [];
  
  const { subscribe, set, update } = writable({
    items: initialItems,
    isOpen: false
  });

  return {
    subscribe,
    
    addItem: (product) => update(state => {
      const existingItem = state.items.find(item => item.id === product.id);
      
      if (existingItem) {
        existingItem.quantity += 1;
      } else {
        state.items.push({
          id: product.id,
          name: product.name,
          price: product.price,
          image: product.image,
          quantity: 1
        });
      }
      
      localStorage.setItem('cart', JSON.stringify(state.items));
      return state;
    }),
    
    removeItem: (productId) => update(state => {
      state.items = state.items.filter(item => item.id !== productId);
      localStorage.setItem('cart', JSON.stringify(state.items));
      return state;
    }),
    
    updateQuantity: (productId, quantity) => update(state => {
      const item = state.items.find(item => item.id === productId);
      
      if (item) {
        item.quantity = quantity;
        if (item.quantity <= 0) {
          state.items = state.items.filter(item => item.id !== productId);
        }
        localStorage.setItem('cart', JSON.stringify(state.items));
      }
      
      return state;
    }),
    
    toggleCart: () => update(state => {
      state.isOpen = !state.isOpen;
      return state;
    }),
    
    clearCart: () => update(state => {
      state.items = [];
      localStorage.setItem('cart', JSON.stringify(state.items));
      return state;
    })
  };
}

export const cart = createCartStore();

// Derived stores for calculated values
export const cartTotal = derived(cart, $cart => 
  $cart.items.reduce((total, item) => total + (item.price * item.quantity), 0)
);

export const cartCount = derived(cart, $cart => 
  $cart.items.reduce((count, item) => count + item.quantity, 0)
); 