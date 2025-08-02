// Test the exact scenario the user reported
import { GetMessageThreadRequest } from 'user-messaging-types/entities'
import { validateGetMessageThreadRequest } from './validators'

const request: GetMessageThreadRequest = {
  threadId: '123',
  userId: '456',
  includeMessages: true,
}

console.log('Valid:', validateGetMessageThreadRequest(request))
