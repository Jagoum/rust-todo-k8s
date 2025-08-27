#!/bin/bash

set -e

# Get the IP of the control-plane VM
control_ip=$(multipass info control-plane | grep IPv4 | awk '{print $2}')

# Get the IPs of the worker VMs
worker_1_ip=$(multipass info worker-1 | grep IPv4 | awk '{print $2}')
worker_2_ip=$(multipass info worker-2 | grep IPv4 | awk '{print $2}')

# Create the inventory file
cat > infra/ansible/hosts.ini << EOL
[control]
control-plane ansible_host=$control_ip

[workers]
worker-1 ansible_host=$worker_1_ip
worker-2 ansible_host=$worker_2_ip
EOL

echo "Inventory file created successfully at infra/ansible/hosts.ini"
