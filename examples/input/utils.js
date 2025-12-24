/**
 * Sample JavaScript file with JSDoc comments.
 * @module sample
 * @version 1.0.0
 */

/**
 * Represents a user in the system.
 * @class
 * @param {string} name - The user's name.
 * @param {number} age - The user's age.
 * @example
 * const user = new User("John", 30);
 */
class User {
  constructor(name, age) {
    this.name = name;
    this.age = age;
  }

  /**
   * Get a greeting message.
   * @returns {string} A personalized greeting.
   */
  greet() {
    return `Hello, I'm ${this.name}!`;
  }
}

/**
 * Calculate the sum of two numbers.
 * @param {number} a - First number.
 * @param {number} b - Second number.
 * @returns {number} The sum of a and b.
 * @throws {TypeError} If arguments are not numbers.
 * @since 1.0.0
 * @deprecated Use `add()` from math module instead.
 */
function sum(a, b) {
  if (typeof a !== "number" || typeof b !== "number") {
    throw new TypeError("Arguments must be numbers");
  }
  return a + b;
}

module.exports = { User, sum };
