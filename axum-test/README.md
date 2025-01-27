Includes some debug functionality using Axum middleware to gather streaming byte responses, log them for debug purposes, and then continue to send those bytes to the client.

Gathering the streaming bytes is a blocking operation, which means this approach has only limited utility for debugging because the responses are no longer really streaming.