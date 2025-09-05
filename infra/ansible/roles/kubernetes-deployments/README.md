# Kubernetes Deployments Role

This Ansible role automates the deployment of Kubernetes manifests for the Rust Todo application stack.

## Components Deployed

- **Backend**: Rust Todo API deployment and service
- **Frontend**: React Todo UI deployment and service
- **Keycloak**: Authentication server deployment, service, and ingress
- **Database**: CloudNativePG PostgreSQL cluster and secrets
- **Ingress**: Frontend and backend ingress configurations

## Requirements

- Kubernetes cluster with kubectl configured
- Ansible kubernetes.core collection installed
- Access to the k8s_setup directory with manifest files

## Usage

Run the deployment playbook:

```bash
ansible-playbook -i inventory deploy-kubernetes.yml
```

## Variables

This role uses the existing Kubernetes manifest files from the `k8s_setup/` directory. No additional variables are required as the manifests contain all necessary configuration.

## Dependencies

- kubernetes.core Ansible collection
- kubectl configured and accessible
- Kubernetes cluster running

## Files Structure

```
k8s_setup/
├── backend/
│   ├── deployment.yaml
│   └── service.yaml
├── frontend/
│   ├── deployment.yaml
│   └── service.yaml
├── keycloak/
│   ├── keycloak-deployment.yaml
│   ├── keycloak-service.yaml
│   └── keycloak-ingress.yaml
├── database/
│   ├── postgres-cluster.yaml
│   └── db-secret.yaml
└── ingress/
    ├── frontend-ingress.yaml
    └── backend-ingress.yaml
```

## Notes

- Ensure all required secrets (db-secret, jwt-secret, etc.) are created before running this role
- The role assumes the k8s_setup directory is accessible from the Ansible controller
- All manifests are applied in the default namespace unless specified otherwise
