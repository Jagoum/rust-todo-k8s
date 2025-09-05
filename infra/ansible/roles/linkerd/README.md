# Linkerd Service Mesh Role

This Ansible role automates the installation and configuration of Linkerd service mesh on a Kubernetes cluster.

## Features

- Installs Linkerd CLI
- Installs Linkerd CRDs
- Installs Linkerd control plane
- Configures default namespace for automatic injection
- Verifies installation with health checks

## Requirements

- Kubernetes cluster with kubectl configured
- Internet access for downloading Linkerd CLI
- Ansible user with sudo privileges (optional, depending on installation method)

## Usage

Run the Linkerd deployment playbook:

```bash
ansible-playbook -i inventory deploy-linkerd.yml
```

## Variables

- `ansible_user`: The user account to install Linkerd CLI for (default: ubuntu)

## Dependencies

- kubectl configured and accessible
- Kubernetes cluster running
- Internet connectivity for CLI download

## Installation Process

1. **CLI Installation**: Downloads and installs Linkerd CLI to user's home directory
2. **PATH Configuration**: Adds Linkerd CLI to user's PATH
3. **CRDs Installation**: Installs Linkerd Custom Resource Definitions
4. **Control Plane**: Installs Linkerd control plane components
5. **Namespace Annotation**: Annotates default namespace for automatic proxy injection
6. **Health Check**: Verifies Linkerd installation is working correctly

## Files Structure

```
roles/linkerd/
├── tasks/
│   └── main.yml          # Main installation tasks
├── README.md             # This documentation
└── ...                   # Other standard Ansible role directories
```

## Post-Installation

After successful installation:

- Linkerd CLI will be available in the user's PATH
- Default namespace is annotated for automatic proxy injection
- All new pods in default namespace will automatically get Linkerd proxies
- Use `linkerd check` to verify the installation
- Use `linkerd dashboard` to access the Linkerd dashboard

## Troubleshooting

- Ensure kubectl is properly configured and can access the cluster
- Check that the user has write permissions to their home directory
- Verify internet connectivity for CLI download
- Run `linkerd check` to diagnose any issues

## Integration with Application Deployment

This role works well with the kubernetes-deployments role. After installing Linkerd, redeploy your applications to inject the service mesh proxies:

```bash
kubectl rollout restart deployment/your-deployment-name
