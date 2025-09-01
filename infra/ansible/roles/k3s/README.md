Role: k3s

Overview
- Installs baseline Linux packages for development and provisioning.
- Disables swap and configures required kernel/network settings.
- Installs k3s server on the controller group host, and k3s agent on workers.
- Exposes kubeconfig on controller user and points to controller IP.
- Installs Rust toolchain (rustup, cargo, rustc) on controller user.

Inventory
- Expects Ansible inventory groups: controller and workers.
- The first host in group `controller` is treated as the server.

Variables
- ansible_user: Linux username to own kubeconfig and Rust toolchain (default: ubuntu).

Idempotence
- Uses creates: guards for k3s services and retries for API health.

Notes
- k3s installer fetched via https://get.k3s.io.
- Workers automatically use K3S_URL and K3S_TOKEN from the controller.
