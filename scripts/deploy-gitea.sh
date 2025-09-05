#!/bin/bash
set -e

echo "Deploying Gitea..."

kubectl apply -f k8s_setup/gitea/pvc.yaml
kubectl apply -f k8s_setup/gitea/deployment.yaml
kubectl apply -f k8s_setup/gitea/service.yaml
kubectl apply -f k8s_setup/gitea/ingress.yaml

echo "Gitea deployed. Please add '192.168.82.89 gitea.local' to your /etc/hosts file if not already done."
