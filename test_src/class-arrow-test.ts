interface SubscribeRequest {
  authToken: string
  message: string
}

export class WebSocketEventsHandler {
  private onSubscribe = async (connectionId: string, body: string | undefined) => {
    if (!body) {
      return { error: 'Body not found' }
    }

    let parsedBody: { authToken: string; message: string }
    try {
      parsedBody = JSON.parse(body)
    } catch {
      return { error: 'Invalid request body' }
    }

    // This call should be found
    if (!validateSubscribeRequest(parsedBody)) {
      return { error: 'Invalid subscribe request format' }
    }

    return { success: true }
  }
}
