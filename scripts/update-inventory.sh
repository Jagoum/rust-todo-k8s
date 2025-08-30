#!/bin/bash
set -e

# Check if multipass is installed
if ! command -v multipass >/dev/null 2>&1; then
    echo "[ERROR] Multipass is not installed."
    if [[ "$(uname)" == "Darwin" ]]; then
        echo "To install Multipass on macOS, run:"
        echo "  brew install --cask multipass"
    elif [[ -f /etc/lsb-release ]] && grep -qi ubuntu /etc/lsb-release; then
        echo "To install Multipass on Ubuntu, run:"
        echo "  sudo snap install multipass"
    else
        echo "Please see https://multipass.run/ for installation instructions for your OS."
    fi
    exit 1
fi

# Configuration
VM_NAME_PREFIX="machine"
VM_COUNT=2
SSH_KEY=$HOME/.ssh/id_ed25519

# Ensure Ansible is installed or set up a virtual environment
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
VENV_DIR="$SCRIPT_DIR/.venv_ansible"

activate_venv() {
    # shellcheck disable=SC1090
    source "$VENV_DIR/bin/activate"
}

if command -v ansible-playbook >/dev/null 2>&1; then
    ANSIBLE_CMD="ansible-playbook"
    echo "[INFO] Using system Ansible: $(ansible-playbook --version | head -n1)"
elif [ -d "$VENV_DIR" ] && [ -x "$VENV_DIR/bin/ansible-playbook" ]; then
    echo "[INFO] Using existing virtualenv Ansible."
    activate_venv
    ANSIBLE_CMD="ansible-playbook"
else
    echo "[INFO] Ansible not found. Creating virtual environment and installing Ansible..."
    python3 -m venv "$VENV_DIR"
    activate_venv
    pip install --upgrade pip
    pip install ansible
    ANSIBLE_CMD="ansible-playbook"
fi

# 2. Check and start VMs
ALL_VMS=()
for i in $(seq 1 $VM_COUNT); do
    ALL_VMS+=("${VM_NAME_PREFIX}-${i}")
done

for VM_NAME in "${ALL_VMS[@]}"; do
    if ! multipass info "$VM_NAME" >/dev/null 2>&1; then
        echo "[ERROR] VM '$VM_NAME' does not exist. Please create it with Terraform first."
        exit 1
    fi

    STATE=$(multipass info "$VM_NAME" | grep State | awk '{print $2}')
    if [ "$STATE" != "Running" ]; then
        echo "[INFO] VM '$VM_NAME' is not running. Starting it..."
        multipass start "$VM_NAME"
    else
        echo "[INFO] VM '$VM_NAME' is already running."
    fi
done

# 3. Get VM IPs
echo "Waiting for VMs to obtain IP addresses..."
VM_IPS=()
for i in $(seq 1 $VM_COUNT); do
    VM_NAME="${VM_NAME_PREFIX}-${i}"
    IP=$(multipass info "$VM_NAME" | awk '/IPv4/ {print $2; exit}')
    if [ -z "$IP" ]; then
        echo "[ERROR] VM '$VM_NAME' failed to obtain IP address."
        multipass info "$VM_NAME"
        exit 1
    fi
    VM_IPS+=($IP)
done

# 4. Update inventory file
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
INVENTORY_FILE="$PROJECT_ROOT/infra/ansible/hosts.ini"

echo "[INFO] Updating Ansible inventory: $INVENTORY_FILE"
cat > "$INVENTORY_FILE" <<EOF
[k3s-master]
machine-1 ansible_host=${VM_IPS[0]} ansible_user=ubuntu

[k3s-worker]
machine-2 ansible_host=${VM_IPS[1]} ansible_user=ubuntu

[all:vars]
ansible_become=true
ansible_become_method=sudo
ansible_become_user=root
ansible_ssh_private_key_file=$SSH_KEY
EOF

echo "Inventory updated with VM IPs:"
for i in $(seq 1 $VM_COUNT); do
    echo "  VM (${VM_NAME_PREFIX}-${i}): ${VM_IPS[$i-1]}"
done

# At the end, print manual test and cleanup instructions
cat <<EOF

[INFO] VM inventory updated.

To manually test your VMs:
  1. SSH into a VM:
     multipass shell machine-1

To delete all Multipass VMs and free resources:
  multipass delete --all
  multipass purge

For more info, see: https://multipass.run/
EOF