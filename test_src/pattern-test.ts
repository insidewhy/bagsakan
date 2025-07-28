interface SubscribeRequest {
  email: string
  topic: string
}

function test() {
  const data = { email: 'test@example.com', topic: 'news' }

  // This should be found
  if (validateSubscribeRequest(data)) {
    console.log('Valid')
  }
}
