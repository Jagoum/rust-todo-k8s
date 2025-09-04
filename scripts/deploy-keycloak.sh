#!/bin/bash

# Deploy Keycloak to k3s cluster
# Run this script inside your controller-node multipass VM

set -e

echo "🚀 Deploying Keycloak to k3s cluster..."

# Check cluster status
echo "📊 Checking cluster status..."
kubectl get nodes
kubectl get pods -A

# Apply CloudNativePG operator first (if not already applied)
echo "📦 Applying CloudNativePG operator..."
kubectl apply -f k8s_setup/cnpg-operator.yaml

# Wait for operator to be ready
echo "⏳ Waiting for CloudNativePG operator..."
kubectl wait --for=condition=available --timeout=300s deployment/cnpg-controller-manager -n cnpg-system || echo "Operator may still be starting..."

# Apply PostgreSQL cluster
echo "🐘 Applying PostgreSQL cluster..."
kubectl apply -f k8s_setup/cnpg.yaml

# Wait for PostgreSQL to be ready
# echo "⏳ Waiting for PostgreSQL cluster..."
# kubectl wait --for=condition=ready --timeout=300s pod -l postgresql.cnpg.io/cluster=my-postgres

# Create Keycloak database secret (if not exists)
echo "🔐 Creating Keycloak database secret..."
kubectl apply -f - <<EOF
apiVersion: v1
kind: Secret
metadata:
  name: my-postgres-secret
type: Opaque
stringData:
  password: postgres
EOF

# Apply Keycloak deployment
echo "🔑 Applying Keycloak deployment..."
kubectl apply -f k8s_setup/keycloak-ingres.yaml

# Wait for Keycloak to be ready
echo "⏳ Waiting for Keycloak deployment..."
kubectl wait --for=condition=available --timeout=300s deployment/keycloak

# Check all resources
echo "📋 Checking deployed resources..."
kubectl get pods
kubectl get svc
kubectl get pvc
kubectl get ingress

echo "✅ Keycloak deployment completed!"
echo ""
echo "🌐 Access Keycloak at: http://keycloak.local"
echo "👤 Admin credentials: admin / admin"
echo ""
echo "📝 Next steps:"
echo "1. Add keycloak.local to your /etc/hosts file"
echo "2. Run the realm setup script: ./scripts/setup-keycloak-realm.sh"
echo "3. Test authentication: ./scripts/test-keycloak-integration.sh"
