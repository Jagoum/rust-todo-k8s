#!/bin/bash

# This script configures passwordless SSH access to Multipass VMs for Ansible.

# --- Configuration ---
# Your public SSH key file
# IMPORTANT: Make sure this path points to your PUBLIC key (.pub)
PUBLIC_KEY_FILE="$HOME/.ssh/id_ed25519.pub"

# Names of your Multipass instances
# Add or remove names as needed to match your setup
INSTANCE_NAMES=("controller-node" "worker-1")

# --- Script Logic ---

# 1. Check if the public key file exists
if [ ! -f "$PUBLIC_KEY_FILE" ]; then
    echo "ERROR: Public key file not found at $PUBLIC_KEY_FILE"
    echo "Please make sure the path is correct."
    exit 1
fi

# 2. Read the public key content
PUBLIC_KEY_CONTENT=$(cat "$PUBLIC_KEY_FILE")
if [ -z "$PUBLIC_KEY_CONTENT" ]; then
    echo "ERROR: The public key file is empty."
    exit 1
fi

echo "Public key successfully read."
echo ""

# 3. Loop through each instance and configure SSH
for name in "${INSTANCE_NAMES[@]}"; do
    echo "--- Configuring SSH for instance: $name ---"

    # Check if the instance is running
    if ! multipass info "$name" | grep -q "State: *Running"; then
        echo "WARNING: Instance '$name' is not running. Skipping."
        echo ""
        continue
    fi

    # Use multipass exec to run the configuration commands on the instance
    # The commands are passed as a single string to 'bash -c'
    multipass exec "$name" -- bash -c "
        echo 'Creating .ssh directory...'
        mkdir -p /home/ubuntu/.ssh

        echo 'Adding public key to authorized_keys...'
        # Using a temporary file to avoid issues with shell interpretation of the key
        echo '$PUBLIC_KEY_CONTENT' >> /home/ubuntu/.ssh/authorized_keys

        echo 'Setting correct permissions...'
        chmod 700 /home/ubuntu/.ssh
        chmod 600 /home/ubuntu/.ssh/authorized_keys
        chown -R ubuntu:ubuntu /home/ubuntu/.ssh

        echo 'Configuration for $name complete.'
    "
    echo "--- Finished for instance: $name ---"
    echo ""
done

echo "All instances have been configured."
echo "You should now be able to run your Ansible playbook."
