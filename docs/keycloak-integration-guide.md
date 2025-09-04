# Keycloak Integration Guide

This guide provides step-by-step instructions to integrate your Rust Todo application with Keycloak for authentication and user management on your k3s cluster.

## Prerequisites

- k3s cluster running with Traefik ingress
- PostgreSQL database (CloudNativePG)
- Local registry at `192.168.82.89:5000` (adjust IP as needed)
- `kubectl` configured to access your cluster
- `curl` and `jq` for testing

## Step 1: Deploy Keycloak

### 1.1 Apply Keycloak manifests

```bash
# Apply the updated Keycloak deployment with persistent storage and ingress
kubectl apply -f k8s_setup/keycloak-ingres.yaml

# Wait for Keycloak to be ready
kubectl wait --for=condition=available --timeout=300s deployment/keycloak
```

### 1.2 Verify deployment

```bash
# Check pods
kubectl get pods -l app=keycloak

# Check ingress
kubectl get ingress keycloak-ingress

# Check persistent volume
kubectl get pvc keycloak-pvc
```

### 1.3 Access Keycloak admin console

Add to your `/etc/hosts`:
```
192.168.82.89 keycloak.local
```

Access: http://keycloak.local
- Username: admin
- Password: admin

## Step 2: Configure Keycloak Realm

### Option A: Automated setup (recommended)

```bash
# Make script executable
chmod +x scripts/setup-keycloak-realm.sh

# Run the setup script
./scripts/setup-keycloak-realm.sh
```

### Option B: Manual setup

1. **Create Realm:**
   - Go to http://keycloak.local
   - Login with admin/admin
   - Click "Master" dropdown → "Create realm"
   - Name: `todo-realm`
   - Click "Create"

2. **Create Client:**
   - Go to "Clients" → "Create client"
   - Client ID: `todo-client`
   - Client type: `OpenID Connect`
   - Click "Next"
   - Capability config: Enable "Client authentication"
   - Authentication flow: Enable "Standard flow" and "Direct access grants"
   - Click "Next"
   - Login settings:
     - Valid redirect URIs: `http://todo.local/*`, `http://localhost:3000/*`
     - Web origins: `http://todo.local`, `http://localhost:3000`
   - Click "Save"

3. **Get Client Secret:**
   - Go to "Clients" → "todo-client" → "Credentials" tab
   - Copy the "Client secret"

4. **Create User:**
   - Go to "Users" → "Create new user"
   - Username: `testuser`
   - Email: `test@example.com`
   - First name: `Test`
   - Last name: `User`
   - Click "Create"
   - Go to "Credentials" tab → "Set password"
   - Password: `password123`
   - Disable "Temporary"

5. **Create Role:**
   - Go to "Realm roles" → "Create role"
   - Role name: `user`
   - Click "Save"
   - Go to "Users" → "testuser" → "Role mapping" → "Assign role"
   - Select "user" role

## Step 3: Update Rust Application

### 3.1 Environment Variables

Add to your deployment environment:

```yaml
# In your rust-backend-deployment.yaml
env:
- name: KEYCLOAK_ISSUER_URL
  value: "http://keycloak.local/realms/todo-realm"
- name: KEYCLOAK_CLIENT_ID
  value: "todo-client"
```

### 3.2 Rebuild and redeploy

```bash
# Build new image
docker build -t 192.168.82.89:5000/todo-app-backend:keycloak app/
docker push 192.168.82.89:5000/todo-app-backend:keycloak

# Update deployment
kubectl set image deployment/rust-todo-backend todo-app=192.168.82.89:5000/todo-app-backend:keycloak

# Wait for rollout
kubectl rollout status deployment/rust-todo-backend
```

## Step 4: Test the Integration

### 4.1 Run test script

```bash
# Make executable
chmod +x scripts/test-keycloak-integration.sh

# Run tests
./scripts/test-keycloak-integration.sh
```

### 4.2 Manual testing

1. **Get access token:**
```bash
curl -X POST http://keycloak.local/realms/todo-realm/protocol/openid-connect/token \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "client_id=todo-client" \
  -d "client_secret=YOUR_CLIENT_SECRET" \
  -d "username=testuser" \
  -d "password=password123" \
  -d "grant_type=password"
```

2. **Use token to access API:**
```bash
# Replace TOKEN with actual token from step 1
curl -H "Authorization: Bearer TOKEN" \
     http://api.todo.local/todos
```

3. **Create a todo:**
```bash
curl -X POST http://api.todo.local/todos \
  -H "Authorization: Bearer TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"title": "My first Keycloak-protected todo"}'
```

## Step 5: Frontend Integration (Optional)

If you want to integrate the React frontend with Keycloak:

### 5.1 Install Keycloak JS adapter

```bash
cd frontend
npm install keycloak-js
```

### 5.2 Configure Keycloak in frontend

Create `frontend/src/keycloak.ts`:

```typescript
import Keycloak from 'keycloak-js';

const keycloak = new Keycloak({
  url: 'http://keycloak.local',
  realm: 'todo-realm',
  clientId: 'todo-client'
});

export default keycloak;
```

### 5.3 Update App.tsx

```typescript
import { useEffect, useState } from 'react';
import keycloak from './keycloak';

function App() {
  const [authenticated, setAuthenticated] = useState(false);

  useEffect(() => {
    keycloak.init({ onLoad: 'login-required' }).then((auth) => {
      setAuthenticated(auth);
    });
  }, []);

  if (!authenticated) {
    return <div>Loading...</div>;
  }

  return (
    <div>
      <h1>Welcome, {keycloak.tokenParsed?.preferred_username}</h1>
      {/* Your todo app components */}
    </div>
  );
}

export default App;
```

## Troubleshooting

### Common Issues

1. **Keycloak not accessible:**
   - Check ingress: `kubectl describe ingress keycloak-ingress`
   - Check service: `kubectl get svc keycloak`
   - Check pod logs: `kubectl logs -l app=keycloak`

2. **Token validation fails:**
   - Verify environment variables in backend deployment
   - Check Keycloak logs for JWKS endpoint errors
   - Ensure realm and client configuration matches

3. **Database connection issues:**
   - Check PostgreSQL cluster: `kubectl get clusters.postgresql.cnpg.io`
   - Verify secret: `kubectl get secret my-postgres-secret`

4. **CORS issues:**
   - Ensure Keycloak client has correct redirect URIs and web origins

### Debug Commands

```bash
# Check Keycloak logs
kubectl logs -l app=keycloak -f

# Check backend logs
kubectl logs -l app=rust-todo-backend -f

# Test Keycloak endpoints
curl http://keycloak.local/realms/todo-realm/.well-known/openid-configuration

# Test JWKS endpoint
curl http://keycloak.local/realms/todo-realm/protocol/openid-connect/certs
```

## Security Considerations

1. **Use HTTPS in production:**
   - Configure TLS certificates for ingress
   - Update Keycloak URLs to use HTTPS

2. **Secure client secrets:**
   - Store client secrets in Kubernetes secrets
   - Use different secrets for different environments

3. **Token validation:**
   - Always validate issuer, audience, and expiration
   - Implement token refresh logic in frontend

4. **Database security:**
   - Use strong passwords for PostgreSQL
   - Enable SSL for database connections

## Next Steps

- Implement user registration through Keycloak
- Add role-based access control (RBAC)
- Configure Keycloak themes and customization
- Set up user federation with external providers
- Implement logout and token refresh

This integration provides a solid foundation for secure authentication in your k3s-deployed application.
