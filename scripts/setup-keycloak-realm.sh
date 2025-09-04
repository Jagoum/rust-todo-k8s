#!/bin/bash
set -e

# Keycloak setup script for Todo App
# This script creates a realm, client, and users for the todo application

KEYCLOAK_URL="http://localhost:8080"
ADMIN_USER="admin"
ADMIN_PASSWORD="admin"
REALM_NAME="todo-realm"
CLIENT_ID="todo-client"
CLIENT_SECRET="todo-client-secret"

echo "Waiting for Keycloak to be ready..."
until curl -s "${KEYCLOAK_URL}/realms/master" > /dev/null; do
  echo "Keycloak not ready yet, waiting..."
  sleep 5
done

echo "Getting admin token..."
ADMIN_TOKEN=$(curl -s -X POST "${KEYCLOAK_URL}/realms/master/protocol/openid-connect/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "username=${ADMIN_USER}" \
  -d "password=${ADMIN_PASSWORD}" \
  -d "grant_type=password" \
  -d "client_id=admin-cli" | jq -r '.access_token')

if [ "$ADMIN_TOKEN" = "null" ] || [ -z "$ADMIN_TOKEN" ]; then
  echo "Failed to get admin token"
  exit 1
fi

echo "Creating realm: ${REALM_NAME}"
curl -s -X POST "${KEYCLOAK_URL}/admin/realms" \
  -H "Authorization: Bearer ${ADMIN_TOKEN}" \
  -H "Content-Type: application/json" \
  -d "{
    \"realm\": \"${REALM_NAME}\",
    \"enabled\": true,
    \"registrationAllowed\": false,
    \"loginWithEmailAllowed\": true,
    \"duplicateEmailsAllowed\": false,
    \"resetPasswordAllowed\": true,
    \"editUsernameAllowed\": false,
    \"bruteForceProtected\": true
  }"

echo "Creating client: ${CLIENT_ID}"
CLIENT_RESPONSE=$(curl -s -X POST "${KEYCLOAK_URL}/admin/realms/${REALM_NAME}/clients" \
  -H "Authorization: Bearer ${ADMIN_TOKEN}" \
  -H "Content-Type: application/json" \
  -d "{
    \"clientId\": \"${CLIENT_ID}\",
    \"enabled\": true,
    \"protocol\": \"openid-connect\",
    \"clientAuthenticatorType\": \"client-secret\",
    \"secret\": \"${CLIENT_SECRET}\",
    \"directAccessGrantsEnabled\": true,
    \"serviceAccountsEnabled\": true,
    \"implicitFlowEnabled\": false,
    \"standardFlowEnabled\": true,
    \"publicClient\": false,
    \"redirectUris\": [
      \"http://todo.local/*\",
      \"http://localhost:3000/*\",
      \"http://api.todo.local/*\",
      \"http://localhost:8080/*\"
    ],
    \"webOrigins\": [
      \"http://todo.local\",
      \"http://localhost:3000\",
      \"http://api.todo.local\",
      \"http://localhost:8080\"
    ]
  }")

echo "Creating user: testuser"
curl -s -X POST "${KEYCLOAK_URL}/admin/realms/${REALM_NAME}/users" \
  -H "Authorization: Bearer ${ADMIN_TOKEN}" \
  -H "Content-Type: application/json" \
  -d "{
    \"username\": \"testuser\",
    \"enabled\": true,
    \"emailVerified\": true,
    \"firstName\": \"Test\",
    \"lastName\": \"User\",
    \"email\": \"test@example.com\",
    \"credentials\": [{
      \"type\": \"password\",
      \"value\": \"password123\",
      \"temporary\": false
    }]
  }"

echo "Creating user: adminuser"
curl -s -X POST "${KEYCLOAK_URL}/admin/realms/${REALM_NAME}/users" \
  -H "Authorization: Bearer ${ADMIN_TOKEN}" \
  -H "Content-Type: application/json" \
  -d "{
    \"username\": \"adminuser\",
    \"enabled\": true,
    \"emailVerified\": true,
    \"firstName\": \"Admin\",
    \"lastName\": \"User\",
    \"email\": \"admin@example.com\",
    \"credentials\": [{
      \"type\": \"password\",
      \"value\": \"admin123\",
      \"temporary\": false
    }]
  }"

echo "Keycloak realm and users setup complete!"
echo ""
echo "Realm: ${REALM_NAME}"
echo "Client ID: ${CLIENT_ID}"
echo "Client Secret: ${CLIENT_SECRET}"
echo ""
echo "Test Users:"
echo "  testuser / password123"
echo "  adminuser / admin123"
echo ""
echo "Keycloak Admin Console: ${KEYCLOAK_URL}/admin"
echo "Realm Settings: ${KEYCLOAK_URL}/admin/master/console/#/realms/${REALM_NAME}"
