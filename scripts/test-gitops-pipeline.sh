#!/bin/bash
set -e

echo "Testing GitOps Pipeline..."

# Check if Gitea is running
echo "Checking Gitea..."
kubectl get pods -l app=gitea -n default

# Check if ArgoCD is running
echo "Checking ArgoCD..."
kubectl get pods -n argocd

# Check if Linkerd is running
echo "Checking Linkerd..."
kubectl get pods -n linkerd

# Check namespace annotations
echo "Checking namespace annotations..."
kubectl get namespace default --show-labels

# Check if app has Linkerd sidecars
echo "Checking app sidecars..."
kubectl get pods -n default

echo "GitOps pipeline test completed."
