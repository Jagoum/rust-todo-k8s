# Kubernetes Setup Workflow and Architecture

This document provides a comprehensive overview of the Kubernetes deployment setup for the Rust Todo application with Keycloak authentication and PostgreSQL database.

## Architecture Overview

```mermaid
graph TB
    subgraph "External Traffic"
        User[User] --> Ingress[Traefik Ingress]
    end

    subgraph "Kubernetes Cluster"
        Ingress --> Frontend[React Frontend<br/>todo.local]
        Ingress --> Backend[Rust Backend API<br/>api.todo.local]
        Ingress --> Keycloak[Keycloak IAM<br/>keycloak.local]

        Frontend --> Backend
        Backend --> Keycloak
        Backend --> Postgres[(PostgreSQL<br/>CNPG Cluster)]
        Keycloak --> Postgres
    end

    subgraph "Storage"
        Postgres --> PVC[Persistent Volume<br/>1Gi per instance]
    end

    subgraph "Secrets & Config"
        Backend --> DB_Secret[Database Secret]
        Keycloak --> KC_DB_Secret[Keycloak DB Secret]
        Keycloak --> TLS_Secret[TLS Certificate]
    end
```

## Component Details

### 1. Database Layer (CNPG)

```mermaid
graph TD
    A[CNPG Operator] --> B[PostgreSQL Cluster]
    B --> C[3 Instances]
    B --> D[Automatic Failover]
    B --> E[Backup & Recovery]

    C --> F[Read-Write Service<br/>my-postgres-rw]
    C --> G[Read-Only Service<br/>my-postgres-ro]

    F --> H[Rust Backend]
    F --> I[Keycloak]
```

**Key Features:**
- **High Availability**: 3 PostgreSQL instances with automatic failover
- **Storage**: 1Gi persistent volume per instance
- **Backup**: Automated backup capabilities through CNPG
- **Scaling**: Horizontal scaling support

### 2. Authentication Layer (Keycloak)

```mermaid
graph TD
    A[User Login] --> B[Keycloak UI<br/>keycloak.local]
    B --> C[Authentication]
    C --> D[JWT Token Generation]
    D --> E[Rust Backend]

    E --> F[Token Validation]
    F --> G{Valid?}
    G -->|Yes| H[Access Granted]
    G -->|No| I[Access Denied]

    J[Database] --> K[User Sessions]
    J --> L[Realm Configuration]
```

**Key Features:**
- **Production Mode**: HTTPS enabled with TLS certificates
- **Database Integration**: Stores user data in PostgreSQL
- **JWT Tokens**: Issues and validates JSON Web Tokens
- **Realm Management**: Configurable authentication realms

### 3. Application Layer

```mermaid
graph TD
    A[React Frontend<br/>todo.local] --> B[User Interface]
    B --> C[API Calls to Backend]
    C --> D[Rust Backend API<br/>api.todo.local]

    D --> E[Authentication Middleware]
    E --> F{Token Valid?}
    F -->|Yes| G[Business Logic]
    F -->|No| H[401 Unauthorized]

    G --> I[Database Operations]
    I --> J[PostgreSQL Cluster]

    K[Todo CRUD] --> L[Create]
    K --> M[Read]
    K --> N[Update]
    K --> O[Delete]
```

## Deployment Workflow

```mermaid
flowchart TD
    A[Start] --> B[Install Prerequisites]
    B --> C[Deploy CNPG Operator]
    C --> D[Create PostgreSQL Cluster]
    D --> E[Deploy Keycloak]
    E --> F[Configure TLS Certificates]
    F --> G[Deploy Rust Backend]
    G --> H[Deploy React Frontend]
    H --> I[Configure Ingress Rules]
    I --> J[Update DNS/Hosts]
    J --> K[Test Application]
    K --> L{Working?}
    L -->|Yes| M[Production Ready]
    L -->|No| N[Troubleshoot]
    N --> K
```

## Detailed Deployment Steps

### Step 1: Prerequisites
```bash
# Install kubectl and helm
# Ensure Kubernetes cluster is running
# Install Traefik ingress controller
kubectl apply -f https://raw.githubusercontent.com/traefik/traefik/v2.5/docs/content/reference/dynamic-configuration/kubernetes-crd-definition-v1.yml
kubectl apply -f https://raw.githubusercontent.com/traefik/traefik/v2.5/docs/content/reference/dynamic-configuration/kubernetes-crd-rbac.yml
kubectl apply -f https://raw.githubusercontent.com/traefik/traefik/v2.5/docs/content/reference/dynamic-configuration/kubernetes-crd.yml
```

### Step 2: Database Setup
```bash
# Deploy CNPG operator
kubectl apply -f k8s_setup/database/cnpg-operator.yaml

# Create PostgreSQL cluster
kubectl apply -f k8s_setup/database/cnpg.yaml

# Wait for cluster to be ready
kubectl wait --for=condition=ready pod -l postgresql.cnpg.io/cluster=my-postgres --timeout=300s
```

### Step 3: Keycloak Setup
```bash
# Generate TLS certificates
openssl req -x509 -newkey rsa:4096 -keyout k8s_setup/certificates/tls.key -out k8s_setup/certificates/tls.crt -days 365 -nodes -subj "/CN=keycloak.local"

# Create TLS secret
kubectl create secret tls keycloak-tls --cert=k8s_setup/certificates/tls.crt --key=k8s_setup/certificates/tls.key

# Deploy Keycloak components
kubectl apply -f k8s_setup/keycloak/

# Wait for Keycloak to be ready
kubectl wait --for=condition=available --timeout=300s deployment/keycloak
```

### Step 4: Application Deployment
```bash
# Deploy Rust backend
kubectl apply -f k8s_setup/rust-todo/rust-backend-deployment.yaml

# Deploy React frontend
kubectl apply -f k8s_setup/rust-todo/react-frontend-deployment.yaml

# Configure ingress
kubectl apply -f k8s_setup/rust-todo/todo-ingress.yaml
```

### Step 5: DNS Configuration
```bash
# Update /etc/hosts file
echo "192.168.82.89 keycloak.local todo.local api.todo.local" | sudo tee -a /etc/hosts
```

## Service Dependencies

```mermaid
graph TD
    A[Rust Backend] --> B[PostgreSQL<br/>my-postgres-rw:5432]
    A --> C[Keycloak<br/>keycloak:8080]
    A --> D[JWT Secret<br/>jwt-secret]

    E[React Frontend] --> F[Rust Backend<br/>backend:8080]

    G[Keycloak] --> H[PostgreSQL<br/>my-postgres-rw:5432]
    G --> I[TLS Secret<br/>keycloak-tls]

    J[Ingress] --> K[React Frontend<br/>react-todo-ui-service:3000]
    J --> L[Rust Backend<br/>backend:8080]
    J --> M[Keycloak<br/>keycloak:8080]
```

## Monitoring and Troubleshooting

### Health Checks
```bash
# Check pod status
kubectl get pods

# Check services
kubectl get services

# Check ingress
kubectl get ingress

# View logs
kubectl logs -f deployment/rust-todo-api
kubectl logs -f deployment/keycloak
```

### Common Issues

1. **Certificate Errors**: Ensure TLS secret is properly created
2. **Database Connection**: Verify PostgreSQL cluster is running
3. **Ingress Not Working**: Check Traefik ingress controller
4. **Keycloak Login Issues**: Verify HTTPS configuration

### Scaling Considerations

```mermaid
graph TD
    A[Load Increase] --> B{Monitor Metrics}
    B --> C{CPU > 80%?}
    C -->|Yes| D[Scale Application]
    C -->|No| E{Memory > 80%?}
    E -->|Yes| F[Scale Database]
    E -->|No| G[Monitor Database]

    D --> H[kubectl scale deployment rust-todo-api --replicas=3]
    F --> I[Increase PostgreSQL instances in CNPG cluster]
```

## Security Considerations

- **TLS Everywhere**: All services use HTTPS in production
- **Secret Management**: Database credentials stored in Kubernetes secrets
- **Network Policies**: Consider implementing network segmentation
- **RBAC**: Keycloak provides fine-grained access control
- **Audit Logging**: Enable audit logs for compliance

## Backup and Recovery

```mermaid
flowchart TD
    A[Scheduled Backup] --> B[CNPG Backup]
    B --> C[S3 Storage]
    C --> D[Backup Verification]

    E[Disaster Recovery] --> F[Restore from Backup]
    F --> G[Update Application Config]
    G --> H[Test Recovery]
    H --> I{Recovery Successful?}
    I -->|Yes| J[Resume Normal Operations]
    I -->|No| K[Troubleshoot Recovery]
```

This comprehensive setup provides a production-ready, scalable, and secure deployment of the Rust Todo application with Keycloak authentication and PostgreSQL database management.
