#!/bin/bash
set -e

echo "Initializing Gitea repositories..."

# Wait for Gitea to be ready
echo "Waiting for Gitea to be ready..."
kubectl wait --for=condition=available --timeout=300s deployment/gitea -n default

# Get Gitea pod name
GITEA_POD=$(kubectl get pods -l app=gitea -o jsonpath='{.items[0].metadata.name}')

# Create app-source repo
kubectl exec $GITEA_POD -- su git -c "cd /data/gitea && git init --bare repositories/app-source.git"

# Create infra repo
kubectl exec $GITEA_POD -- su git -c "cd /data/gitea && git init --bare repositories/infra.git"

echo "Gitea repositories initialized."
echo "You can now clone them at:"
echo "git clone http://gitea.local/app-source.git"
echo "git clone http://gitea.local/infra.git"
