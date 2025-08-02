// Test importing from external packages
import { validateUser } from './validators'

// Example: importing from a hypothetical types package
export interface ExtendedUser {
  id: number
  name: string
  email: string
  // Additional fields beyond basic User
  lastLogin: Date
  settings: Record<string, any>
}

// Use the validator to trigger generation
const testExtended: ExtendedUser = {
  id: 1,
  name: 'Test',
  email: 'test@example.com',
  lastLogin: new Date(),
  settings: {},
}

console.log('Extended user is valid:', validateExtendedUser(testExtended))

// This import will trigger validator generation
import { validateExtendedUser } from './validators'
