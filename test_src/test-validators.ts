import { validateUser, validateProduct, validateOrder } from './validators'

// Test valid objects
const validUser = {
  id: 1,
  name: 'John Doe',
  email: 'john@example.com',
  isActive: true,
  tags: ['developer', 'admin'],
}

const validProduct = {
  id: 100,
  name: 'Laptop',
  price: 999.99,
  description: 'High-performance laptop',
  categories: ['electronics', 'computers'],
  inStock: true,
}

const validOrder = {
  id: 1000,
  userId: 1,
  products: [validProduct],
  total: 999.99,
  status: 'pending' as const,
  createdAt: '2024-01-01T00:00:00Z',
}

// Test invalid objects
const invalidUser = {
  id: 'not a number', // Should be number
  name: 'John Doe',
  email: 'john@example.com',
  isActive: true,
}

const invalidProduct = {
  id: 100,
  name: 'Laptop',
  // Missing required price
  categories: ['electronics'],
  inStock: true,
}

const invalidOrder = {
  id: 1000,
  userId: 1,
  products: [validProduct],
  total: 999.99,
  status: 'invalid_status', // Invalid status
  createdAt: '2024-01-01T00:00:00Z',
}

// Run tests
console.log('Valid user:', validateUser(validUser)) // Should be true
console.log('Invalid user:', validateUser(invalidUser)) // Should be false
console.log('Valid product:', validateProduct(validProduct)) // Should be true
console.log('Invalid product:', validateProduct(invalidProduct)) // Should be false
console.log('Valid order:', validateOrder(validOrder)) // Should be true
console.log('Invalid order:', validateOrder(invalidOrder)) // Should be false

// Test edge cases
console.log('null:', validateUser(null)) // Should be false
console.log('undefined:', validateUser(undefined)) // Should be false
console.log('string:', validateUser('not an object')) // Should be false
console.log('array:', validateUser([])) // Should be false
