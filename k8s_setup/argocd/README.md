# ArgoCD Deployment

This directory contains Kubernetes manifests to deploy ArgoCD on your cluster.

## Files

- `namespace.yaml`: Creates the `argocd` namespace.
- `install.yaml`: ArgoCD Application manifest to install ArgoCD from the official repository.

## Deployment Steps

1. Ensure you have ArgoCD CRDs installed (if required).
2. Apply the namespace:
   ```
   kubectl apply -f namespace.yaml
   ```
3. Apply the ArgoCD installation:
   ```
   kubectl apply -f install.yaml
   ```
4. Wait for ArgoCD to be deployed and synced.
5. Access ArgoCD UI (default credentials: admin/admin123 or check logs).

## Using the Deploy Script

Alternatively, use the provided script:
```
./scripts/deploy-argocd.sh
```

This script will apply the manifests in the correct order.
