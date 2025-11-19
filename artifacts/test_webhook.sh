#!/bin/bash

# Test script for webhook endpoint

echo "Testing webhook endpoint with sample data..."

# Send the webhook
curl -X POST \
  http://localhost:7876/webhook/glitchtip \
  -H "Content-Type: application/json" \
  -d @glitchtip.webhook.json

echo ""