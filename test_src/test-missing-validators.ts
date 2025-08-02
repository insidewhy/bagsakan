// Test file to use the missing validators
import {
  validateSendMessageRequest,
  validateUserSawMessageThreadRequest,
  validateGetMessageThreadsRequest,
} from './validators'

const data = {}

if (validateSendMessageRequest(data)) {
  console.log('SendMessageRequest valid')
}

if (validateUserSawMessageThreadRequest(data)) {
  console.log('UserSawMessageThreadRequest valid')
}

if (validateGetMessageThreadsRequest(data)) {
  console.log('GetMessageThreadsRequest valid')
}
