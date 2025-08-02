// Test importing from package subpath like 'package/entities'
// This simulates the issue with user-messaging-types/entities.js

export interface TestRequest {
  id: number
  message: string
}

// Simulate usage of validator
import { validateTestRequest } from './validators'

const test: TestRequest = {
  id: 1,
  message: 'test',
}

console.log('Valid:', validateTestRequest(test))
