#!/bin/bash

# Test Keycloak Integration Script
# This script demonstrates how to acquire tokens and test the authentication flow

set -e

# Configuration
KEYCLOAK_URL="http://keycloak.local"
REALM_NAME="todo-realm"
CLIENT_ID="todo-client"
CLIENT_SECRET="todo-client-secret"
BACKEND_URL="http://api.todo.local"

echo "Testing Keycloak integration..."

# Function to get access token
get_token() {
    echo "Getting access token for testuser..."
    TOKEN_RESPONSE=$(curl -s -X POST "$KEYCLOAK_URL/realms/$REALM_NAME/protocol/openid-connect/token" \
      -H "Content-Type: application/x-www-form-urlencoded" \
      -d "client_id=$CLIENT_ID" \
      -d "client_secret=$CLIENT_SECRET" \
      -d "username=testuser" \
      -d "password=password123" \
      -d "grant_type=password")

    ACCESS_TOKEN=$(echo $TOKEN_RESPONSE | jq -r '.access_token')
    if [ "$ACCESS_TOKEN" = "null" ] || [ -z "$ACCESS_TOKEN" ]; then
        echo "Failed to get access token"
        echo "Response: $TOKEN_RESPONSE"
        exit 1
    fi
    echo "Access token acquired successfully"
}

# Test 1: Health check (no auth required)
echo ""
echo "Test 1: Health check"
curl -s "$BACKEND_URL/healthz" | jq .

# Test 2: Try to access protected route without token
echo ""
echo "Test 2: Access protected route without token"
curl -s -w "\nHTTP Status: %{http_code}\n" "$BACKEND_URL/todos" | jq .

# Test 3: Get token and access protected route
echo ""
echo "Test 3: Get token and access protected route"
get_token

echo "Accessing /todos with token..."
curl -s -H "Authorization: Bearer $ACCESS_TOKEN" \
     -w "\nHTTP Status: %{http_code}\n" \
     "$BACKEND_URL/todos" | jq .

# Test 4: Create a todo
echo ""
echo "Test 4: Create a todo"
CREATE_RESPONSE=$(curl -s -X POST "$BACKEND_URL/todos" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title": "Test todo from Keycloak integration"}' \
  -w "\nHTTP Status: %{http_code}\n")

echo "Create response:"
echo $CREATE_RESPONSE | jq .

# Extract todo ID
TODO_ID=$(echo $CREATE_RESPONSE | jq -r '.id')

# Test 5: List todos again
echo ""
echo "Test 5: List todos after creation"
curl -s -H "Authorization: Bearer $ACCESS_TOKEN" \
     "$BACKEND_URL/todos" | jq .

# Test 6: Update todo
if [ "$TODO_ID" != "null" ] && [ -n "$TODO_ID" ]; then
    echo ""
    echo "Test 6: Update todo"
    curl -s -X PUT "$BACKEND_URL/todos/$TODO_ID" \
      -H "Authorization: Bearer $ACCESS_TOKEN" \
      -H "Content-Type: application/json" \
      -d '{"title": "Updated test todo", "completed": true}' \
      -w "\nHTTP Status: %{http_code}\n" | jq .
fi

# Test 7: Test with invalid token
echo ""
echo "Test 7: Test with invalid token"
curl -s -H "Authorization: Bearer invalid.token.here" \
     -w "\nHTTP Status: %{http_code}\n" \
     "$BACKEND_URL/todos" | jq .

echo ""
echo "Keycloak integration tests completed!"
echo ""
echo "To inspect the JWT token, you can decode it at https://jwt.io"
echo "Or use: echo '$ACCESS_TOKEN' | jq -R 'split(\".\") | .[1] | @base64d | fromjson'"
