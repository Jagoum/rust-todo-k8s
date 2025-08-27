# Rust Todo App on Kubernetes

This project is a comprehensive example of how to build and deploy a Rust-based todo application on Kubernetes. It includes the infrastructure setup using Terraform and Ansible, Kubernetes manifests, the Rust application itself, Dockerfiles for containerization, and various helper scripts.

## Project Structure

The project is organized into the following directories:

```
rust-todo-k8s/
├── infra/
│ ├── terraform/
│ │ ├── main.tf
│ │ └── variables.tf
│ └── ansible/
│ ├── hosts.ini
│ ├── playbook.yml
│ └── roles/
├── k8s/
│ ├── namespaces.yaml
│ ├── registry.yaml
│ ├── cnpg/
│ ├── keycloak/
│ └── app/
│ ├── deployment.yaml
│ ├── service.yaml
│ └── ingress.yaml
├── app/
│ ├── Cargo.toml
│ └── src/
│ ├── main.rs
│ ├── auth.rs
│ ├── db.rs
│ └── handlers.rs
├── docker/
│ ├── Dockerfile
│ └── registry-compose.yaml
├── scripts/
│ ├── build-and-load.sh
│ ├── install-argocd.sh
│ └── smoke-test.sh
└── docs/
└── diagrams.mmd
```

- **`infra/`**: Contains the infrastructure as code.
  - **`terraform/`**: Terraform scripts for provisioning the underlying infrastructure (e.g., VMs using Multipass).
  - **`ansible/`**: Ansible playbooks for configuring the provisioned infrastructure.
- **`k8s/`**: Kubernetes manifests for deploying the application and its dependencies.
  - **`cnpg/`**: Manifests for the CloudNativePG PostgreSQL operator.
  - **`keycloak/`**: Manifests for Keycloak for authentication.
  - **`app/`**: Manifests for the Rust todo application.
- **`app/`**: The Rust todo application source code.
- **`docker/`**: Docker-related files.
  - **`Dockerfile`**: For building the Rust application container image.
  - **`registry-compose.yaml`**: A Docker Compose file for running a local Docker registry.
- **`scripts/`**: Helper scripts for building, deploying, and testing the application.
- **`docs/`**: Project documentation, including Mermaid diagrams.

## Getting Started

To get started with this project, you will need to have the following tools installed:

- Terraform
- Ansible
- Docker
- kubectl
- A Kubernetes cluster (e.g., k3s, minikube)

1.  **Provision the infrastructure:**
    ```bash
    cd infra/terraform
    terraform init
    terraform apply
    ```

2.  **Configure the infrastructure:**
    ```bash
    cd infra/ansible
    ansible-playbook -i hosts.ini playbook.yml
    ```

3.  **Deploy the application to Kubernetes:**
    ```bash
    kubectl apply -f k8s/
    ```

## Usage

Once the application is deployed, you can access it through the Ingress endpoint defined in `k8s/app/ingress.yaml`.

## Contributing

Contributions are welcome! Please feel free to open an issue or submit a pull request.

## License

This project is licensed under the terms of the LICENSE file.