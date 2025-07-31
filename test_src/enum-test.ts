// Test interface with enum type
export enum Status {
  Pending = 'pending',
  Processing = 'processing',
  Completed = 'completed',
  Cancelled = 'cancelled',
}

export interface OrderWithEnum {
  id: number
  status: Status
}

// TypeScript enum (numeric) - small range
export enum Priority {
  Low,
  Medium,
  High,
}

export interface TaskWithNumericEnum {
  id: number
  priority: Priority
}

// Larger consecutive range enum
export enum Level {
  Beginner = 1,
  Novice = 2,
  Intermediate = 3,
  Advanced = 4,
  Expert = 5,
  Master = 6,
}

export interface PlayerWithLevel {
  id: number
  level: Level
}

// Non-consecutive enum
export enum ErrorCode {
  NotFound = 404,
  Unauthorized = 401,
  BadRequest = 400,
  ServerError = 500,
  ServiceUnavailable = 503,
}

export interface ErrorResponse {
  code: ErrorCode
  message: string
}

// Add usage to trigger validator generation
import {
  validateOrderWithEnum,
  validateTaskWithNumericEnum,
  validatePlayerWithLevel,
  validateErrorResponse,
} from './validators'

const testEnum = {
  id: 1,
  status: Status.Pending,
}

const testNumEnum = {
  id: 2,
  priority: Priority.High,
}

const testPlayer = {
  id: 3,
  level: Level.Expert,
}

const testError = {
  code: ErrorCode.NotFound,
  message: 'Resource not found',
}

console.log('Enum order valid:', validateOrderWithEnum(testEnum))
console.log('Numeric enum task valid:', validateTaskWithNumericEnum(testNumEnum))
console.log('Player level valid:', validatePlayerWithLevel(testPlayer))
console.log('Error response valid:', validateErrorResponse(testError))
