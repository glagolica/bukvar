/**
 * Shopping cart service for e-commerce applications.
 *
 * Handles cart operations including item management, pricing calculations,
 * discount application, and checkout validation.
 *
 * @module cart-service
 * @version 3.1.0
 * @author E-Commerce Team
 * @since 2.0.0
 */

/**
 * @typedef {Object} Product
 * @property {string} id - Unique product identifier
 * @property {string} name - Product name
 * @property {number} price - Unit price in cents
 * @property {string} [sku] - Stock keeping unit
 * @property {number} [weight] - Weight in grams for shipping
 * @property {boolean} [taxable=true] - Whether product is taxable
 */

/**
 * @typedef {Object} CartItem
 * @property {Product} product - The product
 * @property {number} quantity - Quantity in cart
 * @property {number} subtotal - Line item total (price × quantity)
 * @property {Discount[]} [appliedDiscounts] - Discounts applied to this item
 */

/**
 * @typedef {Object} Discount
 * @property {string} code - Discount code
 * @property {string} type - 'percentage' | 'fixed' | 'bogo'
 * @property {number} value - Discount value
 * @property {string} [description] - Human-readable description
 * @property {Date} [expiresAt] - Expiration date
 */

/**
 * @typedef {Object} CartSummary
 * @property {CartItem[]} items - All items in cart
 * @property {number} subtotal - Total before discounts and tax
 * @property {number} discountTotal - Total discount amount
 * @property {number} taxTotal - Total tax amount
 * @property {number} shippingTotal - Shipping cost
 * @property {number} grandTotal - Final total
 * @property {string} currency - Currency code (e.g., 'USD')
 */

/**
 * Shopping cart management class.
 *
 * @class
 * @example
 * const cart = new ShoppingCart('USD', 0.08); // 8% tax rate
 *
 * cart.addItem({ id: '1', name: 'Widget', price: 999 }, 2);
 * cart.applyDiscount({ code: 'SAVE10', type: 'percentage', value: 10 });
 *
 * const summary = cart.getSummary();
 * console.log(`Total: $${(summary.grandTotal / 100).toFixed(2)}`);
 */
class ShoppingCart {
  /**
   * Creates a new shopping cart.
   *
   * @param {string} [currency='USD'] - Currency code
   * @param {number} [taxRate=0] - Tax rate as decimal (0.08 = 8%)
   */
  constructor(currency = "USD", taxRate = 0) {
    /** @type {Map<string, CartItem>} */
    this.items = new Map();
    /** @type {Discount[]} */
    this.discounts = [];
    this.currency = currency;
    this.taxRate = taxRate;
  }

  /**
   * Adds a product to the cart.
   *
   * @param {Product} product - Product to add
   * @param {number} [quantity=1] - Quantity to add
   * @returns {CartItem} The added or updated cart item
   * @throws {Error} If quantity is less than 1
   *
   * @example
   * cart.addItem({ id: 'prod-1', name: 'T-Shirt', price: 1999 }, 3);
   */
  addItem(product, quantity = 1) {
    if (quantity < 1) {
      throw new Error("Quantity must be at least 1");
    }

    const existing = this.items.get(product.id);
    if (existing) {
      existing.quantity += quantity;
      existing.subtotal = existing.product.price * existing.quantity;
      return existing;
    }

    const item = {
      product,
      quantity,
      subtotal: product.price * quantity,
      appliedDiscounts: [],
    };
    this.items.set(product.id, item);
    return item;
  }

  /**
   * Updates the quantity of an item in the cart.
   *
   * @param {string} productId - Product ID to update
   * @param {number} quantity - New quantity
   * @returns {CartItem|null} Updated item or null if not found
   * @throws {Error} If quantity is negative
   */
  updateQuantity(productId, quantity) {
    if (quantity < 0) {
      throw new Error("Quantity cannot be negative");
    }

    if (quantity === 0) {
      return this.removeItem(productId);
    }

    const item = this.items.get(productId);
    if (!item) return null;

    item.quantity = quantity;
    item.subtotal = item.product.price * quantity;
    return item;
  }

  /**
   * Removes an item from the cart.
   *
   * @param {string} productId - Product ID to remove
   * @returns {CartItem|null} Removed item or null if not found
   */
  removeItem(productId) {
    const item = this.items.get(productId);
    this.items.delete(productId);
    return item || null;
  }

  /**
   * Applies a discount code to the cart.
   *
   * @param {Discount} discount - Discount to apply
   * @returns {boolean} True if discount was applied
   * @throws {Error} If discount code is already applied
   *
   * @example
   * // Percentage discount
   * cart.applyDiscount({ code: 'SUMMER20', type: 'percentage', value: 20 });
   *
   * // Fixed amount discount
   * cart.applyDiscount({ code: 'FLAT500', type: 'fixed', value: 500 });
   */
  applyDiscount(discount) {
    if (this.discounts.some((d) => d.code === discount.code)) {
      throw new Error(`Discount ${discount.code} is already applied`);
    }

    if (discount.expiresAt && new Date() > discount.expiresAt) {
      throw new Error(`Discount ${discount.code} has expired`);
    }

    this.discounts.push(discount);
    return true;
  }

  /**
   * Removes a discount from the cart.
   *
   * @param {string} code - Discount code to remove
   * @returns {boolean} True if discount was removed
   */
  removeDiscount(code) {
    const index = this.discounts.findIndex((d) => d.code === code);
    if (index === -1) return false;
    this.discounts.splice(index, 1);
    return true;
  }

  /**
   * Calculates the cart summary with all totals.
   *
   * @returns {CartSummary} Complete cart summary
   *
   * @example
   * const summary = cart.getSummary();
   * console.log(`Items: ${summary.items.length}`);
   * console.log(`Subtotal: ${summary.subtotal}`);
   * console.log(`Grand Total: ${summary.grandTotal}`);
   */
  getSummary() {
    const items = Array.from(this.items.values());
    const subtotal = items.reduce((sum, item) => sum + item.subtotal, 0);
    const discountTotal = this.calculateDiscounts(subtotal);
    const taxableAmount = subtotal - discountTotal;
    const taxTotal = Math.round(taxableAmount * this.taxRate);
    const shippingTotal = this.calculateShipping(items);

    return {
      items,
      subtotal,
      discountTotal,
      taxTotal,
      shippingTotal,
      grandTotal: subtotal - discountTotal + taxTotal + shippingTotal,
      currency: this.currency,
    };
  }

  /**
   * Calculates total discount amount.
   *
   * @private
   * @param {number} subtotal - Cart subtotal
   * @returns {number} Total discount in cents
   */
  calculateDiscounts(subtotal) {
    return this.discounts.reduce((total, discount) => {
      if (discount.type === "percentage") {
        return total + Math.round(subtotal * (discount.value / 100));
      }
      if (discount.type === "fixed") {
        return total + discount.value;
      }
      return total;
    }, 0);
  }

  /**
   * Calculates shipping cost based on items.
   *
   * @private
   * @param {CartItem[]} items - Cart items
   * @returns {number} Shipping cost in cents
   */
  calculateShipping(items) {
    const totalWeight = items.reduce(
      (sum, item) => sum + (item.product.weight || 0) * item.quantity,
      0
    );
    // Simple shipping: $5 base + $1 per 500g
    if (totalWeight === 0) return 0;
    return 500 + Math.ceil(totalWeight / 500) * 100;
  }

  /**
   * Clears all items and discounts from the cart.
   */
  clear() {
    this.items.clear();
    this.discounts = [];
  }

  /**
   * Returns the number of unique items in cart.
   *
   * @returns {number} Number of unique items
   */
  get itemCount() {
    return this.items.size;
  }

  /**
   * Returns the total quantity of all items.
   *
   * @returns {number} Total quantity
   */
  get totalQuantity() {
    return Array.from(this.items.values()).reduce(
      (sum, item) => sum + item.quantity,
      0
    );
  }
}

module.exports = { ShoppingCart };
