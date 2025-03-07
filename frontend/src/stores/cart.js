import { writable, derived } from 'svelte/store';

// Initialize the cart store
export const cartItems = writable([]);

// Create derived store for cart total
export const cartTotal = derived(cartItems, ($cartItems) => {
    return $cartItems.reduce((total, item) => {
        return total + (item.price * item.quantity);
    }, 0);
});

// Create the cart store with all methods
function createCart() {
    const { subscribe, set, update } = cartItems;

    return {
        subscribe,
        addItem: (item) => {
            update(items => {
                const existingItem = items.find(i => i.id === item.id);
                if (existingItem) {
                    return items.map(i => 
                        i.id === item.id 
                            ? { ...i, quantity: i.quantity + 1 }
                            : i
                    );
                }
                return [...items, { ...item, quantity: 1 }];
            });
        },
        removeItem: (id) => {
            update(items => items.filter(i => i.id !== id));
        },
        updateQuantity: (id, quantity) => {
            if (quantity < 1) return;
            update(items =>
                items.map(item =>
                    item.id === id
                        ? { ...item, quantity }
                        : item
                )
            );
        },
        clear: () => set([])
    };
}

// Export the cart store
export const cart = createCart();

// Export helper functions
export const addToCart = cart.addItem;
export const removeFromCart = cart.removeItem;
export const updateQuantity = cart.updateQuantity;
export const clearCart = cart.clear;

// Load cart from localStorage on initialization
const savedCart = localStorage.getItem('cart');
if (savedCart) {
    cartItems.set(JSON.parse(savedCart));
}

// Subscribe to changes and save to localStorage
cartItems.subscribe(items => {
    localStorage.setItem('cart', JSON.stringify(items));
});

// Export cart count for convenience
export const cartCount = derived(cartItems, $cartItems => 
    $cartItems.reduce((count, item) => count + item.quantity, 0)
); 