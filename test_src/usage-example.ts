import { User, Product, Order } from './models'

// Example usage - these calls should trigger validator generation
function processUser(data: unknown) {
  if (validateUser(data)) {
    console.log('Valid user:', data.name)
  }
}

class OrderService {
  processOrder(data: unknown) {
    if (validateOrder(data)) {
      // Check nested Product validation
      data.products.forEach((p) => {
        if (validateProduct(p)) {
          console.log('Valid product in order')
        }
      })
    }
  }
}

// Arrow function
const checkUser = (data: unknown) => {
  return validateUser(data) && data.isActive
}

// In object
const validators = {
  checkAll(user: unknown, product: unknown) {
    return validateUser(user) && validateProduct(product)
  },
}
