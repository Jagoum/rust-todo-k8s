#!/bin/bash

# Fix registry configuration for k3s
# Run this in your controller-node VM

set -e

echo "🔧 Fixing registry configuration for k3s..."

# Method 1: Configure k3s registries.yaml for insecure registry
echo "📝 Creating k3s registries configuration..."
sudo mkdir -p /etc/rancher/k3s/

sudo tee /etc/rancher/k3s/registries.yaml > /dev/null <<EOF
mirrors:
  registry.local:5000:
    endpoint:
      - "http://registry.local:5000"
configs:
  registry.local:5000:
    tls:
      insecure_skip_verify: true
EOF

echo "🔄 Restarting k3s service..."
sudo systemctl restart k3s

echo "⏳ Waiting for k3s to restart..."
sleep 10

echo "📊 Checking cluster status..."
kubectl get nodes

echo "✅ Registry configuration updated!"
echo ""
echo "🔍 Test the registry access:"
echo "curl -k http://registry.local:5000/v2/_catalog"
echo ""
echo "🚀 Now try deploying Keycloak again:"
echo "kubectl apply -f k8s_setup/keycloak-ingres.yaml"
