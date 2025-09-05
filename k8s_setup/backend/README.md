# Backend Deployment

This directory contains Kubernetes manifests for deploying the Rust Todo API backend.

## Files

- `deployment.yaml`: Deployment manifest for the Rust Todo API backend.
- `service.yaml`: Service manifest exposing the backend deployment.

## Deployment Steps

1. Ensure your Kubernetes cluster is running and accessible.
2. Apply the backend deployment:
   ```
   kubectl apply -f deployment.yaml
   ```
3. Apply the backend service:
   ```
   kubectl apply -f service.yaml
   ```
4. Verify the deployment and service are running:
   ```
   kubectl get pods,svc -l app=rust-todo-api
   ```

## Environment Variables

- `DATABASE_URL`: Provided via Kubernetes secret `db-secret`.
- `JWT_SECRET`: Provided via Kubernetes secret `jwt-secret`.

Make sure these secrets are created in the cluster before deploying.

## Using the Deploy Script

You can also use the modular deploy scripts to deploy the backend along with other components:
- `scripts/deploy-modular.sh`
- `scripts/deploy-modular-multipass.sh` (for multipass VM)

These scripts handle deployment order and dependencies.
