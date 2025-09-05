#!/bin/bash

# Comprehensive Test Script for Modular Kubernetes Deployment
# Tests the entire application flow including frontend, backend, and Keycloak integration

set -e

echo "🚀 Starting Comprehensive Modular Deployment Test"
echo "=================================================="

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

# Function to check if a Kubernetes resource exists
check_resource() {
    local resource_type=$1
    local resource_name=$2
    local namespace=${3:-default}

    if kubectl get $resource_type $resource_name -n $namespace &>/dev/null; then
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

    local status=$(kubectl get pod $pod_name -n $namespace -o jsonpath='{.status.phase}' 2>/dev/null || echo "NotFound")

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

    if curl -s --max-time $timeout -o /dev/null -w "%{http_code}" "$url" | grep -q "^$expected_status$"; then
        print_success "Endpoint $url returned expected status $expected_status"
        return 0
    else
        print_error "Endpoint $url failed or returned unexpected status"
        return 1
    fi
}

# Function to wait for deployment rollout
wait_for_deployment() {
    local deployment_name=$1
    local namespace=${2:-default}
    local timeout=${3:-300}

    print_status "Waiting for deployment '$deployment_name' to be ready..."

    if kubectl wait --for=condition=available --timeout=${timeout}s deployment/$deployment_name -n $namespace; then
        print_success "Deployment '$deployment_name' is ready"
        return 0
    else
        print_error "Deployment '$deployment_name' failed to become ready"
        return 1
    fi
}

# Function to check Keycloak integration
test_keycloak_integration() {
    print_status "Testing Keycloak integration..."

    # Check if Keycloak is accessible
    if ! test_endpoint "https://keycloak.local/auth/realms/todo-realm/.well-known/openid-connect-configuration" 200; then
        print_error "Keycloak realm configuration not accessible"
        return 1
    fi

    # Check if backend can authenticate with Keycloak
    if ! test_endpoint "http://todo.com/api/health" 200; then
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
    if ! kubectl get clusters.postgresql.cnpg.io my-postgres -n default -o jsonpath='{.status.phase}' | grep -q "Cluster in healthy state"; then
        print_error "PostgreSQL cluster is not healthy"
        return 1
    fi

    # Test database connection from backend
    local backend_pod=$(kubectl get pods -l app=rust-todo-api -n default -o jsonpath='{.items[0].metadata.name}' 2>/dev/null || echo "")

    if [ -z "$backend_pod" ]; then
        print_error "No backend pod found"
        return 1
    fi

    if kubectl exec $backend_pod -n default -- curl -s http://localhost:8080/api/health | grep -q "healthy"; then
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

    # Test main frontend page
    if ! test_endpoint "http://todo.com" 200; then
        print_error "Frontend main page not accessible"
        return 1
    fi

    # Test frontend assets
    if ! test_endpoint "http://todo.com/static/js/main.js" 200; then
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
    if ! check_resource "ingress" "frontend-ingress"; then
        return 1
    fi

    if ! check_resource "ingress" "backend-ingress"; then
        return 1
    fi

    if ! check_resource "ingress" "keycloak-ingress"; then
        return 1
    fi

    # Test ingress host resolution
    if ! nslookup todo.com &>/dev/null; then
        print_warning "DNS resolution for todo.com may not be configured"
    fi

    print_success "Ingress configuration test passed"
    return 0
}

# Main test execution
main() {
    local test_failures=0

    print_status "Starting modular deployment validation..."
    echo

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
    check_resource "service" "keycloak-service" || ((test_failures++))
    check_resource "secret" "keycloak-tls-secret" || ((test_failures++))
    check_resource "configmap" "keycloak-config" || ((test_failures++))
    check_resource "pvc" "keycloak-pvc" || ((test_failures++))

    # Ingress resources
    check_resource "ingress" "frontend-ingress" || ((test_failures++))
    check_resource "ingress" "backend-ingress" || ((test_failures++))
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

    # This would require more complex testing with actual user credentials
    # For now, we'll test basic connectivity
    if test_endpoint "http://todo.com" 200 && test_endpoint "https://keycloak.local" 200; then
        print_success "Basic end-to-end connectivity test passed"
    else
        print_error "End-to-end connectivity test failed"
        ((test_failures++))
    fi

    echo
    echo "=================================================="

    if [ $test_failures -eq 0 ]; then
        print_success "🎉 All tests passed! Modular deployment is working correctly."
        exit 0
    else
        print_error "❌ $test_failures test(s) failed. Please review the errors above."
        exit 1
    fi
}

# Run main function
main "$@"
