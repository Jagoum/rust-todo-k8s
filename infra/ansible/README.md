# Ansible Automation for Rust Todo Application

This directory contains Ansible playbooks and roles for automating the deployment and management of the complete Rust Todo application stack.

## Overview

The Ansible setup provides automated deployment for:

- **Kubernetes Cluster**: k3s cluster provisioning
- **Service Mesh**: Linkerd installation and configuration
- **Application Stack**: Backend, frontend, Keycloak, database, and ingress
- **GitOps**: ArgoCD and Gitea deployment (future roles)

## Directory Structure

```
infra/ansible/
├── roles/
│   ├── k3s/                    # Existing k3s cluster setup
│   ├── offline_registry_dns/   # Existing DNS configuration
│   ├── kubernetes-deployments/ # NEW: Application manifests deployment
│   └── linkerd/                # NEW: Service mesh installation
├── playbook.yml                # Main cluster provisioning playbook
├── deploy-kubernetes.yml       # NEW: Application deployment playbook
├── deploy-linkerd.yml          # NEW: Linkerd deployment playbook
├── deploy-full-stack.yml       # NEW: Complete stack deployment
├── inventory                   # Ansible inventory file
└── README.md                   # This documentation
```

## Prerequisites

- Ansible 2.9+
- Python 3
- kubectl configured for the target cluster
- Access to Kubernetes cluster
- Internet connectivity for downloading tools

## Installation

1. **Install required Ansible collections**:
   ```bash
   ansible-galaxy collection install kubernetes.core community.general
   ```

2. **Update inventory file** with your cluster details:
   ```ini
   [controller]
   your-controller-host ansible_host=192.168.x.x ansible_user=ubuntu

   [workers]
   worker1 ansible_host=192.168.x.x ansible_user=ubuntu
   worker2 ansible_host=192.168.x.x ansible_user=ubuntu
   ```

## Usage

### 1. Provision Kubernetes Cluster

```bash
ansible-playbook -i inventory playbook.yml
```

This sets up k3s cluster with all necessary components.

### 2. Deploy Linkerd Service Mesh

```bash
ansible-playbook -i inventory deploy-linkerd.yml
```

Installs and configures Linkerd service mesh.

### 3. Deploy Application Stack

```bash
ansible-playbook -i inventory deploy-kubernetes.yml
```

Deploys all Kubernetes manifests for the application stack.

### 4. Deploy Complete Stack (Recommended)

```bash
ansible-playbook -i inventory deploy-full-stack.yml
```

Runs Linkerd installation followed by application deployment with verification.

## Roles Description

### kubernetes-deployments

**Purpose**: Deploys all Kubernetes manifests for the application stack

**Components deployed**:
- Rust Todo API (backend)
- React Todo UI (frontend)
- Keycloak authentication server
- CloudNativePG PostgreSQL cluster
- Ingress configurations

**Requirements**: Kubernetes cluster with kubectl access

### linkerd

**Purpose**: Installs and configures Linkerd service mesh

**Features**:
- Downloads and installs Linkerd CLI
- Installs Linkerd CRDs and control plane
- Configures namespace for automatic injection
- Performs health checks

**Requirements**: Internet access, kubectl access

## Variables

### Common Variables

- `ansible_user`: SSH user for connecting to hosts (default: ubuntu)
- `ansible_python_interpreter`: Python interpreter path (default: /usr/bin/python3)

### Role-Specific Variables

Most roles use defaults and don't require additional variables. The Kubernetes manifests contain all necessary configuration.

## Secrets Management

Ensure the following Kubernetes secrets are created before deployment:

- `db-secret`: Database connection credentials
- `jwt-secret`: JWT signing secret
- `keycloak-db-secret`: Keycloak database credentials

## Troubleshooting

### Common Issues

1. **kubectl connection issues**:
   - Verify kubeconfig is properly configured
   - Check cluster connectivity

2. **Permission issues**:
   - Ensure ansible user has sudo privileges if needed
   - Check SSH key authentication

3. **Collection installation**:
   - Run `ansible-galaxy collection install kubernetes.core` if needed

### Debugging

Enable verbose output:
```bash
ansible-playbook -i inventory deploy-full-stack.yml -vvv
```

Check specific role execution:
```bash
ansible-playbook -i inventory deploy-linkerd.yml --tags linkerd
```

## Integration with Existing Scripts

The Ansible automation complements your existing shell scripts:

- `scripts/deploy-modular.sh`: Manual deployment
- `scripts/deploy-modular-multipass.sh`: Multipass-specific deployment
- Ansible playbooks: Automated, repeatable deployment

## Future Enhancements

Potential areas for additional automation:

- **ArgoCD Role**: Automate GitOps deployment
- **Gitea Role**: Automate Git server setup
- **Monitoring**: Prometheus/Grafana stack
- **Backup**: Automated database backups
- **Secrets Management**: Integration with external secret stores

## Contributing

When adding new roles:

1. Follow Ansible best practices
2. Include comprehensive README.md for each role
3. Test roles individually before integration
4. Update this main README with new role information

## Support

For issues with Ansible automation:

1. Check role-specific README files
2. Verify prerequisites are met
3. Run with verbose output for debugging
4. Check Ansible documentation for specific modules
