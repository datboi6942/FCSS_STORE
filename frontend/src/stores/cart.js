import { writable, derived, get } from 'svelte/store';
import { auth } from './auth.js';

// Initialize cart stores based on user
function createCartStore() {
    // Create a store map of user ID -> cart items
    const userCarts = new Map();
    // Current active cart
    const { subscribe, set, update } = writable([]);
    // Current user ID (or guest ID)
    let currentUserId = 'guest-' + Math.random().toString(36).substring(2, 15);
    
    // Subscribe to auth changes to switch carts when user changes
    auth.subscribe(authState => {
        if (authState && authState.isAuthenticated && authState.user && authState.user.id) {
            const userId = authState.user.id;
            // User logged in - switch to their cart
            if (userId !== currentUserId) {
                // Save current cart if it's a guest cart
                if (currentUserId.startsWith('guest-') && get({ subscribe }).length > 0) {
                    userCarts.set(currentUserId, get({ subscribe }));
                }
                
                // Switch to user's cart
                currentUserId = userId;
                
                // Load user's cart or initialize empty
                const userCart = userCarts.get(userId) || loadCartFromStorage(userId) || [];
                set(userCart);
                
                // If we had a guest cart, merge it with the user cart if requested
                const guestId = localStorage.getItem('guestCartId');
                if (guestId && userCarts.has(guestId)) {
                    const guestCart = userCarts.get(guestId);
                    if (guestCart && guestCart.length > 0) {
                        // Merge guest cart into user cart
                        const mergedCart = mergeCartItems(userCart, guestCart);
                        set(mergedCart);
                        
                        // Clear guest cart
                        userCarts.delete(guestId);
                        localStorage.removeItem('cart-' + guestId);
                    }
                }
                
                console.log(`Switched to cart for user ${userId}`);
            }
        } else {
            // User logged out - switch to a new guest cart
            if (!currentUserId.startsWith('guest-')) {
                // Save current cart in user's storage
                if (get({ subscribe }).length > 0) {
                    userCarts.set(currentUserId, get({ subscribe }));
                    saveCartToStorage(currentUserId, get({ subscribe }));
                }
                
                // Generate new guest ID
                currentUserId = 'guest-' + Math.random().toString(36).substring(2, 15);
                localStorage.setItem('guestCartId', currentUserId);
                
                // Start with empty cart
                set([]);
                console.log(`Switched to new guest cart ${currentUserId}`);
            }
        }
    });
    
    // Helper function to merge cart items
    function mergeCartItems(userCart, guestCart) {
        const result = [...userCart];
        
        guestCart.forEach(guestItem => {
            const existingItemIndex = result.findIndex(item => item.id === guestItem.id);
            
            if (existingItemIndex >= 0) {
                // Item exists, combine quantities
                result[existingItemIndex].quantity += guestItem.quantity;
            } else {
                // New item, add to cart
                result.push(guestItem);
            }
        });
        
        return result;
    }
    
    // Helper to load cart from localStorage
    function loadCartFromStorage(userId) {
        try {
            const storedCart = localStorage.getItem('cart-' + userId);
            return storedCart ? JSON.parse(storedCart) : null;
        } catch (e) {
            console.error('Failed to load cart from storage:', e);
            return null;
        }
    }
    
    // Helper to save cart to localStorage
    function saveCartToStorage(userId, items) {
        try {
            localStorage.setItem('cart-' + userId, JSON.stringify(items));
        } catch (e) {
            console.error('Failed to save cart to storage:', e);
        }
    }
    
    return {
        subscribe,
        addItem: (item) => {
            update(items => {
                const existingItem = items.find(i => i.id === item.id);
                const newItems = existingItem
                    ? items.map(i => i.id === item.id ? { ...i, quantity: i.quantity + 1 } : i)
                    : [...items, { ...item, quantity: 1 }];
                
                // Save to storage for current user
                saveCartToStorage(currentUserId, newItems);
                return newItems;
            });
        },
        removeItem: (id) => {
            update(items => {
                const newItems = items.filter(i => i.id !== id);
                saveCartToStorage(currentUserId, newItems);
                return newItems;
            });
        },
        updateQuantity: (id, quantity) => {
            if (quantity < 1) return;
            update(items => {
                const newItems = items.map(item => item.id === id ? { ...item, quantity } : item);
                saveCartToStorage(currentUserId, newItems);
                return newItems;
            });
        },
        clear: () => {
            set([]);
            saveCartToStorage(currentUserId, []);
        },
        getCurrentUserId: () => currentUserId
    };
}

// Create the cart store
export const cart = createCartStore();

// Create derived store for cart total
export const cartTotal = derived(cart, ($cart) => {
    return $cart.reduce((total, item) => {
        return total + (item.price * item.quantity);
    }, 0);
});

// Export cart count for convenience
export const cartCount = derived(cart, $cart => 
    $cart.reduce((count, item) => count + item.quantity, 0)
);

// Export helper functions
export const addToCart = cart.addItem;
export const removeFromCart = cart.removeItem;
export const updateQuantity = cart.updateQuantity;
export const clearCart = cart.clear; 