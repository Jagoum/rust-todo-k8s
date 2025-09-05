#!/bin/bash
set -e

echo "Redeploying app with Linkerd injection..."

# Restart deployments to inject sidecars
kubectl rollout restart deployment/rust-todo-api -n default
kubectl rollout restart deployment/react-todo-ui -n default
kubectl rollout restart deployment/keycloak -n default

echo "Waiting for deployments to be ready..."
kubectl wait --for=condition=available --timeout=300s deployment/rust-todo-api -n default
kubectl wait --for=condition=available --timeout=300s deployment/react-todo-ui -n default
kubectl wait --for=condition=available --timeout=300s deployment/keycloak -n default

echo "App redeployed with Linkerd injection."
