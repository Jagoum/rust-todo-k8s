#!/bin/bash

# Modular Kubernetes Deployment Script for Multipass VM
# Deploys all components in the correct order with proper dependencies

set -e

echo "🚀 Starting Modular Kubernetes Deployment (Multipass)"
echo "==================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to execute kubectl command in multipass VM
kubectl_mp() {
    local cmd="cd /home/ubuntu/Projects/rust-todo-k8s && $1"
    multipass exec controller-node -- bash -c "$cmd"
}

# Function to copy file to multipass VM
copy_to_mp() {
    local local_file=$1
    local remote_path=$2
    multipass copy-files "$local_file" "controller-node:$remote_path"
}

# Function to apply Kubernetes manifest via multipass
apply_manifest() {
    local file_path=$1
    local description=$2

    print_status "Applying $description: $file_path"

    if [ -f "$file_path" ]; then
        # Copy file to multipass VM
        copy_to_mp "$file_path" "/home/ubuntu/Projects/rust-todo-k8s/$(basename "$file_path")"

        # Apply the manifest
        if kubectl_mp "kubectl apply -f $(basename "$file_path")"; then
            print_success "$description applied successfully"
        else
            print_error "Failed to apply $description"
            return 1
        fi
    else
        print_error "File not found: $file_path"
        return 1
    fi
}

# Function to wait for deployment
wait_for_deployment() {
    local deployment_name=$1
    local namespace=${2:-default}
    local timeout=${3:-300}

    print_status "Waiting for deployment '$deployment_name' to be ready..."

    if kubectl_mp "kubectl wait --for=condition=available --timeout=${timeout}s deployment/$deployment_name -n $namespace"; then
        print_success "Deployment '$deployment_name' is ready"
    else
        print_error "Deployment '$deployment_name' failed to become ready"
        return 1
    fi
}

# Function to wait for PostgreSQL cluster
wait_for_postgres() {
    local cluster_name=${1:-my-postgres}
    local namespace=${2:-default}
    local timeout=${3:-600}

    print_status "Waiting for PostgreSQL cluster '$cluster_name' to be ready..."

    local start_time=$(date +%s)
    while true; do
        local current_time=$(date +%s)
        local elapsed=$((current_time - start_time))

        if [ $elapsed -gt $timeout ]; then
            print_error "Timeout waiting for PostgreSQL cluster '$cluster_name'"
            return 1
        fi

        local phase=$(kubectl_mp "kubectl get clusters.postgresql.cnpg.io $cluster_name -n $namespace -o jsonpath='{.status.phase}' 2>/dev/null || echo 'NotFound'")

        if [ "$phase" = "Cluster in healthy state" ]; then
            print_success "PostgreSQL cluster '$cluster_name' is healthy"
            return 0
        elif [ "$phase" = "NotFound" ]; then
            print_warning "PostgreSQL cluster '$cluster_name' not found, waiting..."
        else
            print_status "PostgreSQL cluster phase: $phase (elapsed: ${elapsed}s)"
        fi

        sleep 10
    done
}

# Main deployment function
main() {
    print_status "Starting modular deployment process via multipass..."
    echo

    # Check if multipass is available
    if ! command -v multipass &> /dev/null; then
        print_error "multipass is not installed or not in PATH"
        exit 1
    fi

    # Check if controller-node VM exists
    if ! multipass list | grep -q "controller-node"; then
        print_error "controller-node multipass VM not found"
        print_status "Please ensure the controller-node VM is running"
        exit 1
    fi

    # Step 1: Install CloudNativePG Operator
    print_status "=== STEP 1: Installing CloudNativePG Operator ==="
    apply_manifest "k8s_setup/database/cnpg-operator.yaml" "CloudNativePG Operator CRDs"

    # Wait for operator to be ready
    print_status "Waiting for CloudNativePG operator to be ready..."
    sleep 30

    echo

    # Step 2: Deploy Database Components
    print_status "=== STEP 2: Deploying Database Components ==="
    apply_manifest "k8s_setup/database/db-secret.yaml" "Database Secret"
    apply_manifest "k8s_setup/database/postgres-cluster.yaml" "PostgreSQL Cluster"

    # Wait for PostgreSQL cluster to be healthy
    wait_for_postgres "my-postgres"

    echo

    # Step 3: Deploy Keycloak Components
    print_status "=== STEP 3: Deploying Keycloak Components ==="
    apply_manifest "k8s_setup/keycloak/keycloak-db-secret.yaml" "Keycloak Database Secret"
    apply_manifest "k8s_setup/keycloak/keycloak-pvc.yaml" "Keycloak Persistent Volume Claim"
    apply_manifest "k8s_setup/keycloak/keycloak-configmap.yaml" "Keycloak ConfigMap"
    apply_manifest "k8s_setup/keycloak/keycloak-deployment.yaml" "Keycloak Deployment"
    apply_manifest "k8s_setup/keycloak/keycloak-service.yaml" "Keycloak Service"
    apply_manifest "k8s_setup/keycloak/keycloak-tls-secret.yaml" "Keycloak TLS Secret"
    apply_manifest "k8s_setup/keycloak/keycloak-middleware.yaml" "Keycloak Middleware"
    apply_manifest "k8s_setup/keycloak/keycloak-redirect-middleware.yaml" "Keycloak Redirect Middleware"
    apply_manifest "k8s_setup/keycloak/keycloak-ingress.yaml" "Keycloak Ingress"

    # Wait for Keycloak deployment
    wait_for_deployment "keycloak"

    echo

    # Step 4: Deploy Backend Components
    print_status "=== STEP 4: Deploying Backend Components ==="
    apply_manifest "k8s_setup/backend/deployment.yaml" "Backend Deployment"
    apply_manifest "k8s_setup/backend/service.yaml" "Backend Service"

    # Wait for backend deployment
    wait_for_deployment "rust-todo-api"

    echo

    # Step 5: Deploy Frontend Components
    print_status "=== STEP 5: Deploying Frontend Components ==="
    apply_manifest "k8s_setup/frontend/deployment.yaml" "Frontend Deployment"
    apply_manifest "k8s_setup/frontend/service.yaml" "Frontend Service"

    # Wait for frontend deployment
    wait_for_deployment "react-todo-ui"

    echo

    # Step 6: Deploy Ingress Resources
    print_status "=== STEP 6: Deploying Ingress Resources ==="
    apply_manifest "k8s_setup/ingress/backend-ingress.yaml" "Backend Ingress"
    apply_manifest "k8s_setup/ingress/frontend-ingress.yaml" "Frontend Ingress"

    echo

    # Step 7: Setup Keycloak Realm
    print_status "=== STEP 7: Setting up Keycloak Realm ==="
    if [ -f "scripts/setup-keycloak-realm.sh" ]; then
        print_status "Copying Keycloak realm setup script to multipass..."
        copy_to_mp "scripts/setup-keycloak-realm.sh" "/home/ubuntu/Projects/rust-todo-k8s/setup-keycloak-realm.sh"

        print_status "Running Keycloak realm setup script..."
        if kubectl_mp "bash setup-keycloak-realm.sh"; then
            print_success "Keycloak realm setup completed"
        else
            print_warning "Keycloak realm setup may have issues, but continuing..."
        fi
    else
        print_warning "Keycloak realm setup script not found, skipping..."
    fi

    echo
    echo "==================================================="

    # Final verification
    print_status "=== DEPLOYMENT SUMMARY ==="
    print_status "Checking deployed resources..."

    # Check key resources
    local resources_to_check=(
        "secret/db-secret"
        "cluster.postgresql.cnpg.io/my-postgres"
        "deployment/rust-todo-api"
        "deployment/react-todo-ui"
        "deployment/keycloak"
        "service/backend"
        "service/react-todo-ui-service"
        "service/keycloak"
        "ingress/react-todo-ui-ingress"
        "ingress/rust-todo-api-ingress"
        "ingress/keycloak-ingress"
    )

    local failed_checks=0
    for resource in "${resources_to_check[@]}"; do
        if kubectl_mp "kubectl get $resource -n default &>/dev/null && echo 'exists' || echo 'notfound'" | grep -q "exists"; then
            print_success "✓ $resource"
        else
            print_error "✗ $resource"
            ((failed_checks++))
        fi
    done

    echo

    if [ $failed_checks -eq 0 ]; then
        print_success "🎉 Modular deployment completed successfully!"
        print_status "Your application should be accessible at:"
        print_status "  - Frontend: http://todo.com"
        print_status "  - Backend API: http://todo.com/api"
        print_status "  - Keycloak: https://keycloak.local"
        echo
        print_status "Run './scripts/test-modular-deployment-multipass.sh' to verify the deployment"
    else
        print_error "❌ $failed_checks resource(s) failed to deploy properly"
        print_status "Check the errors above and try again"
        exit 1
    fi
}

# Run main deployment
main "$@"
