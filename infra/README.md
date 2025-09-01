# Kubernetes Lab Setup

![Kubernetes Logo](https://raw.githubusercontent.com/cncf/artwork/master/projects/kubernetes/horizontal/color/kubernetes-horizontal-color.png)

## Overview

This repository provides a comprehensive lab environment for setting up a multi-node Kubernetes cluster using [Multipass](https://multipass.run/) for virtual machine provisioning, [Terraform](https://www.terraform.io/) for infrastructure as code, and [Ansible](https://www.ansible.com/) for configuration management. The goal is to create a repeatable, disposable, and educational Kubernetes cluster for learning, testing, and demonstration purposes.

## Features

*   **Automated VM Provisioning:** Utilizes Terraform with the Multipass provider to create a Kubernetes control plane node and multiple worker nodes.
*   **Automated Cluster Configuration:** Employs Ansible playbooks to install and configure all necessary Kubernetes components (`kubeadm`, `kubelet`, `kubectl`), container runtime (Containerd by default, configurable for Docker), and CNI (Calico).
*   **Idempotent Setup:** Ansible ensures that the configuration can be run multiple times without causing unintended side effects.
*   **Disposable Environment:** Easily tear down the entire lab environment with a single Terraform command.
*   **Educational:** Provides a hands-on environment to understand Kubernetes architecture and deployment processes.

## Technologies Used

*   **[Kubernetes](https://kubernetes.io/):** Container orchestration platform.
*   **[Multipass](https://multipass.run/):** Lightweight VM manager for Linux, Windows, and macOS.
*   **[Terraform](https://www.terraform.io/):** Infrastructure as Code tool for provisioning VMs.
*   **[Ansible](https://www.ansible.com/):** Automation engine for configuration management.
*   **[Containerd](https://containerd.io/):** Default container runtime (can be switched to Docker).
*   **[Calico](https://www.tigera.io/project-calico/):** Container Network Interface (CNI) for Pod networking.

## Prerequisites

Before you begin, ensure you have the following installed on your local machine:

*   **Multipass:** [Installation Guide](https://multipass.run/docs/install)
*   **Terraform:** [Installation Guide](https://learn.hashicorp.com/tutorials/terraform/install-cli)
*   **Ansible:** [Installation Guide](https://docs.ansible.com/ansible/latest/installation_guide/intro_installation.html)
    *   Recommended installation via `pip`: `pip install ansible`
*   **`jq`:** A lightweight and flexible command-line JSON processor.
    *   macOS: `brew install jq`
    *   Ubuntu/Debian: `sudo apt-get install jq`
*   **SSH Key Pair:** Ensure you have an SSH key pair (`id_rsa` or `id_ed25519`) in your `~/.ssh/` directory. The setup script will automatically detect and use it.

## Lab Setup Instructions

Follow these steps to set up your Kubernetes lab:

### 1. Clone the Repository

```bash
git clone https://github.com/Jagoum/kubernetes_lab.git
cd kubernetes_lab
```

### 2. Provision Virtual Machines with Terraform

This step creates the `controller-node` and `worker-X` VMs using Multipass.

```bash
# Initialize Terraform (downloads Multipass provider)
terraform init

# Review the plan (optional, but recommended)
terraform plan

# Apply the configuration to create VMs
terraform apply
```

Confirm the `terraform apply` prompt by typing `yes`.

### 3. Prepare Ansible Environment

This script dynamically generates the Ansible inventory file and configures SSH access on all your Multipass VMs, allowing Ansible to connect without password prompts.

```bash
# Make the script executable
chmod +x prepare_ansible.sh

# Run the preparation script
./prepare_ansible.sh
```

Review the output of this script for any errors. It will confirm the generated inventory and SSH configuration status for each VM.

### 4. Install Kubernetes with Ansible

This step runs the Ansible playbooks to install and configure Kubernetes components on your VMs.

```bash
ansible-playbook -i ansible/inventory ansible/playbook.yml
```

This process may take several minutes. Ansible will handle:
*   Disabling swap.
*   Synchronizing system time.
*   Installing Containerd (or Docker if configured).
*   Installing `kubeadm`, `kubelet`, `kubectl`.
*   Initializing the Kubernetes control plane on `controller-node`.
*   Installing Calico CNI.
*   Joining worker nodes to the cluster.

### 5. Verify the Kubernetes Cluster

Once the Ansible playbook completes, you can verify the cluster status. Connect to your `controller-node` via `multipass exec` and check the nodes:

```bash
multipass exec controller-node -- kubectl get nodes
```

Expected Output (all nodes should be `Ready`):

```
NAME              STATUS   ROLES           AGE   VERSION
controller-node   Ready    control-plane   <AGE>   <VERSION>
worker-0          Ready    <none>          <AGE>   <VERSION>
worker-1          Ready    <none>          <AGE>   <VERSION>
worker-2          Ready    <none>          <AGE>   <VERSION>
```

To check the status of core Kubernetes pods:

```bash
multipass exec controller-node -- kubectl get pods -A
```

## Deploying a Sample Application (Nginx)

To test your cluster, you can deploy a simple Nginx web server:

1.  **Create `nginx-app.yaml`:**

    ```yaml
    apiVersion: apps/v1
    kind: Deployment
    metadata:
      name: nginx-deployment
      labels:
        app: nginx
    spec:
      replicas: 3
      selector:
        matchLabels:
          app: nginx
      template:
        metadata:
          labels:
            app: nginx
        spec:
          containers:
          - name: nginx
            image: nginx:1.14.2
            ports:
            - containerPort: 80
    ---
    apiVersion: v1
    kind: Service
    metadata:
      name: nginx-service
    spec:
      selector:
        app: nginx
      ports:
        - protocol: TCP
          port: 80
          targetPort: 80
          nodePort: 30080 # Accessible from host on this port
      type: NodePort
    ```

2.  **Deploy the application:**

    ```bash
    kubectl apply -f nginx-app.yaml
    ```

3.  **Verify Pods and Service:**

    ```bash
    kubectl get pods -o wide
    kubectl get services
    ```

4.  **Access Nginx:**

    Get the IP address of any worker node (e.g., `multipass info worker-0 | grep IPv4`). Then, open your web browser and navigate to `http://<WORKER_NODE_IP>:30080`.

    You should see the Nginx welcome page.

## Cleaning Up the Lab

To destroy all the VMs and clean up the Terraform state:

```bash
terraform destroy
```

Confirm the prompt by typing `yes`.

## Troubleshooting

*   **SSH Connection Issues:** Ensure your SSH key is correctly added to the VMs using `./prepare_ansible.sh`. Verify the `ansible/inventory` file points to the correct private key.
*   **`kubeadm` Preflight Errors:** If `kubeadm` fails, check the error messages carefully. Common issues include disabled swap, `ip_forward` not enabled, or cgroup driver misconfiguration. The Ansible playbooks are designed to handle these, but manual intervention might be needed if a previous run failed partially.
*   **`kubectl` Connection Refused:** Ensure you are running `kubectl` as the `ubuntu` user on the `controller-node` (e.g., `multipass exec controller-node -- kubectl get nodes`). If running locally, ensure your `~/.kube/config` is correctly set up to point to the controller's API server.
*   **Metrics Server `API not available`:** Give it a few minutes after deployment. Check the `metrics-server` pod logs in the `kube-system` namespace for specific errors (`kubectl logs -n kube-system <pod-name>`).

## Contributing

Feel free to fork this repository, open issues, or submit pull requests to improve the lab environment.
