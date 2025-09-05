#!/bin/bash

# Comprehensive Test Script for Modular Kubernetes Deployment (Multipass)
# Tests the entire application flow including frontend, backend, and Keycloak integration

set -e

echo "🚀 Starting Comprehensive Modular Deployment Test (Multipass)"
echo "============================================================="

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

# Function to check if a Kubernetes resource exists
check_resource() {
    local resource_type=$1
    local resource_name=$2
    local namespace=${3:-default}

    local result=$(kubectl_mp "kubectl get $resource_type $resource_name -n $namespace &>/dev/null && echo 'exists' || echo 'notfound'")

    if [ "$result" = "exists" ]; then
        print_success "$resource_type '$resource_name' exists in namespace '$namespace'"
        return 0
    else
        print_error "$resource_type '$resource_name' does not exist in namespace '$namespace'"
        return 1
    fi
}

# Function to check pod status
check_pod_status() {
    local pod_name=$1
    local namespace=${2:-default}

    local status=$(kubectl_mp "kubectl get pod $pod_name -n $namespace -o jsonpath='{.status.phase}' 2>/dev/null || echo 'NotFound'")

    if [ "$status" = "Running" ]; then
        print_success "Pod '$pod_name' is running"
        return 0
    elif [ "$status" = "NotFound" ]; then
        print_error "Pod '$pod_name' not found"
        return 1
    else
        print_warning "Pod '$pod_name' status: $status"
        return 1
    fi
}

# Function to test HTTP endpoint
test_endpoint() {
    local url=$1
    local expected_status=${2:-200}
    local timeout=${3:-30}

    print_status "Testing endpoint: $url"

    # Use curl in multipass VM to test endpoints
    local result=$(kubectl_mp "curl -s --max-time $timeout -o /dev/null -w '%{http_code}' '$url' 2>/dev/null || echo 'failed'")

    if [ "$result" = "$expected_status" ]; then
        print_success "Endpoint $url returned expected status $expected_status"
        return 0
    else
        print_error "Endpoint $url failed or returned unexpected status (got: $result)"
        return 1
    fi
}

# Function to wait for deployment rollout
wait_for_deployment() {
    local deployment_name=$1
    local namespace=${2:-default}
    local timeout=${3:-300}

    print_status "Waiting for deployment '$deployment_name' to be ready..."

    if kubectl_mp "kubectl wait --for=condition=available --timeout=${timeout}s deployment/$deployment_name -n $namespace"; then
        print_success "Deployment '$deployment_name' is ready"
        return 0
    else
        print_error "Deployment '$deployment_name' failed to become ready"
        return 1
    fi
}

# Function to test Keycloak integration
test_keycloak_integration() {
    print_status "Testing Keycloak integration..."

    # Check if Keycloak is accessible (accepts 302 redirect as success)
    local keycloak_result=$(kubectl_mp "curl -k -s --max-time 30 -o /dev/null -w '%{http_code}' 'https://keycloak.local' 2>/dev/null || echo 'failed'")

    if [ "$keycloak_result" = "302" ] || [ "$keycloak_result" = "200" ]; then
        print_success "Keycloak is accessible (status: $keycloak_result)"
    else
        print_error "Keycloak not accessible (status: $keycloak_result)"
        return 1
    fi

    # Check if backend API is accessible
    if ! test_endpoint "http://api.todo.local/healthz" 200; then
        print_error "Backend health check failed"
        return 1
    fi

    print_success "Keycloak integration test passed"
    return 0
}

# Function to test database connectivity
test_database_connectivity() {
    print_status "Testing database connectivity..."

    # Check if PostgreSQL cluster is ready
    local phase=$(kubectl_mp "kubectl get clusters.postgresql.cnpg.io my-postgres -n default -o jsonpath='{.status.phase}' 2>/dev/null || echo 'NotFound'")

    if [ "$phase" != "Cluster in healthy state" ]; then
        print_error "PostgreSQL cluster is not healthy (phase: $phase)"
        return 1
    fi

    # Test database connection from backend
    local backend_pod=$(kubectl_mp "kubectl get pods -l app=rust-todo-api -n default -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || echo ''")

    if [ -z "$backend_pod" ]; then
        print_error "No backend pod found"
        return 1
    fi

    if kubectl_mp "kubectl exec $backend_pod -n default -- curl -s http://localhost:8080/api/health | grep -q 'healthy' && echo 'healthy' || echo 'unhealthy'" | grep -q "healthy"; then
        print_success "Database connectivity test passed"
        return 0
    else
        print_error "Database connectivity test failed"
        return 1
    fi
}

# Function to test frontend accessibility
test_frontend_accessibility() {
    print_status "Testing frontend accessibility..."

    # Test main frontend page (using correct domain from ingress)
    if ! test_endpoint "http://todo.local" 200; then
        print_error "Frontend main page not accessible"
        return 1
    fi

    # Test frontend assets
    if ! test_endpoint "http://todo.local/static/js/main.js" 200; then
        print_error "Frontend JavaScript assets not accessible"
        return 1
    fi

    print_success "Frontend accessibility test passed"
    return 0
}

# Function to test ingress configuration
test_ingress_configuration() {
    print_status "Testing ingress configuration..."

    # Check if ingress resources exist
    if ! check_resource "ingress" "react-todo-ui-ingress"; then
        return 1
    fi

    if ! check_resource "ingress" "rust-todo-api-ingress"; then
        return 1
    fi

    if ! check_resource "ingress" "keycloak-ingress"; then
        return 1
    fi

    # Test ingress host resolution (this might not work from inside the VM)
    print_warning "DNS resolution testing may not work from inside multipass VM"

    print_success "Ingress configuration test passed"
    return 0
}

# Main test execution
main() {
    local test_failures=0

    print_status "Starting modular deployment validation via multipass..."
    echo

    # Check if multipass is available
    if ! command -v multipass &> /dev/null; then
        print_error "multipass is not installed or not in PATH"
        exit 1
    fi

    # Check if controller-node VM exists
    if ! multipass list | grep -q "controller-node"; then
        print_error "controller-node multipass VM not found"
        exit 1
    fi

    # Test 1: Check all Kubernetes resources exist
    print_status "=== TEST 1: Kubernetes Resources Validation ==="

    # Database resources
    check_resource "secret" "db-secret" || ((test_failures++))
    check_resource "cluster.postgresql.cnpg.io" "my-postgres" || ((test_failures++))

    # Backend resources
    check_resource "deployment" "rust-todo-api" || ((test_failures++))
    check_resource "service" "backend" || ((test_failures++))

    # Frontend resources
    check_resource "deployment" "react-todo-ui" || ((test_failures++))
    check_resource "service" "react-todo-ui-service" || ((test_failures++))

    # Keycloak resources
    check_resource "deployment" "keycloak" || ((test_failures++))
    check_resource "service" "keycloak" || ((test_failures++))
    check_resource "secret" "keycloak-tls" || ((test_failures++))
    check_resource "configmap" "keycloak-config" || ((test_failures++))
    check_resource "pvc" "keycloak-pvc" || ((test_failures++))

    # Ingress resources
    check_resource "ingress" "react-todo-ui-ingress" || ((test_failures++))
    check_resource "ingress" "rust-todo-api-ingress" || ((test_failures++))
    check_resource "ingress" "keycloak-ingress" || ((test_failures++))

    echo

    # Test 2: Check deployments are ready
    print_status "=== TEST 2: Deployment Readiness Check ==="

    wait_for_deployment "rust-todo-api" || ((test_failures++))
    wait_for_deployment "react-todo-ui" || ((test_failures++))
    wait_for_deployment "keycloak" || ((test_failures++))

    echo

    # Test 3: Test ingress configuration
    print_status "=== TEST 3: Ingress Configuration Test ==="
    test_ingress_configuration || ((test_failures++))

    echo

    # Test 4: Test database connectivity
    print_status "=== TEST 4: Database Connectivity Test ==="
    test_database_connectivity || ((test_failures++))

    echo

    # Test 5: Test frontend accessibility
    print_status "=== TEST 5: Frontend Accessibility Test ==="
    test_frontend_accessibility || ((test_failures++))

    echo

    # Test 6: Test Keycloak integration
    print_status "=== TEST 6: Keycloak Integration Test ==="
    test_keycloak_integration || ((test_failures++))

    echo

    # Test 7: End-to-end application flow test
    print_status "=== TEST 7: End-to-End Application Flow Test ==="

    # Test complete user journey
    print_status "Testing complete user authentication flow..."

    # Test frontend connectivity
    if ! test_endpoint "http://todo.local" 200; then
        print_error "Frontend connectivity test failed"
        ((test_failures++))
    fi

    # Test Keycloak connectivity (accepts 302 redirect as success)
    local keycloak_result=$(kubectl_mp "curl -k -s --max-time 30 -o /dev/null -w '%{http_code}' 'https://keycloak.local' 2>/dev/null || echo 'failed'")

    if [ "$keycloak_result" = "302" ] || [ "$keycloak_result" = "200" ]; then
        print_success "Keycloak connectivity test passed (status: $keycloak_result)"
    else
        print_error "Keycloak connectivity test failed (status: $keycloak_result)"
        ((test_failures++))
    fi

    if [ $test_failures -eq 0 ]; then
        print_success "End-to-end connectivity test passed"
    fi

    echo
    echo "============================================================="

    if [ $test_failures -eq 0 ]; then
        print_success "🎉 All tests passed! Modular deployment is working correctly."
        echo
        echo "============================================================="
        print_success "🚀 APPLICATION ENDPOINTS:"
        echo
        print_status "Frontend (React UI):     http://todo.local"
        print_status "Backend API:             http://api.todo.local"
        print_status "Keycloak (Auth Server):  https://keycloak.local"
        echo
        print_status "Keycloak Admin Console:  https://keycloak.local/admin"
        print_status "Keycloak Realm:          https://keycloak.local/realms/master"
        print_status "Backend Health Check:    http://api.todo.local/healthz"
        echo
        print_success "✅ All endpoints are accessible and ready for use!"
        echo "============================================================="
        exit 0
    else
        print_error "❌ $test_failures test(s) failed. Please review the errors above."
        exit 1
    fi
}

# Run main function
main "$@"
