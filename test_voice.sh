#!/bin/bash

# Generate a simple test WAV file (16kHz mono) with "hello"
# Using macOS 'say' command to generate audio
say -o test_audio.aiff "hello how are you"
# Convert to 16kHz mono WAV
ffmpeg -i test_audio.aiff -ar 16000 -ac 1 test_audio.wav -y 2>/dev/null
rm test_audio.aiff

# Generate UUID for session
SESSION_ID=$(uuidgen)

# Test voice chat endpoint
echo "Testing POST /voice-chat with session: $SESSION_ID"
curl -X POST http://localhost:3000/voice-chat \
  -H "Authorization: Bearer cc4849ab20051e87ddab1dc340263daca0e5a8799aae23853604937e8f7de881" \
  -F "audio=@test_audio.wav" \
  -F "voice_session_id=$SESSION_ID" \
  --output response.mp3 \
  -w "\nHTTP Status: %{http_code}\n"

if [ -f response.mp3 ]; then
  echo "✓ Received MP3 response ($(du -h response.mp3 | cut -f1))"
  echo "Play with: afplay response.mp3"
else
  echo "✗ No response file received"
fi
