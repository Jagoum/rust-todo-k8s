#!/bin/bash
set -e

echo "Installing Linkerd..."

# Add Linkerd to PATH
export PATH="/home/ubuntu/.linkerd2/bin:$PATH"
LINKERD_PATH="linkerd"
KUBECTL_PATH="kubectl"

# Check if Linkerd CLI exists and is executable
if ! command -v $LINKERD_PATH &> /dev/null; then
  echo "Linkerd CLI not found in PATH"
  exit 1
fi

# Install Linkerd control plane
$LINKERD_PATH install | $KUBECTL_PATH apply -f -

echo "Waiting for Linkerd to be ready..."
$KUBECTL_PATH wait --for=condition=available --timeout=300s deployment/linkerd-controller -n linkerd

echo "Linkerd installed successfully."

# Annotate default namespace for injection
$KUBECTL_PATH annotate namespace default linkerd.io/inject=enabled

echo "Default namespace annotated for Linkerd injection."
