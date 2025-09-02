#!/bin/bash

# Build and push script for todo app
set -e

REGISTRY="registry.local:5000"
APP_NAME="todo-app"
VERSION=${1:-latest}

echo "Building and pushing Todo App images to local registry..."

# Build backend image
echo "Building backend image..."
docker build -t ${REGISTRY}/${APP_NAME}-backend:${VERSION} ./app

# Build frontend image
echo "Building frontend image..."
docker build -t ${REGISTRY}/${APP_NAME}-frontend:${VERSION} ./frontend

# Test images locally first
echo "Testing images locally..."
docker-compose -f docker-compose.test.yml up -d --build
sleep 10

# Check if services are healthy
echo "Checking service health..."
if curl -f http://localhost:8080/healthz > /dev/null 2>&1; then
    echo "✅ Backend is healthy"
else
    echo "❌ Backend health check failed"
    docker-compose -f docker-compose.test.yml logs backend
    exit 1
fi

if curl -f http://localhost:3000 > /dev/null 2>&1; then
    echo "✅ Frontend is healthy"
else
    echo "❌ Frontend health check failed"
    docker-compose -f docker-compose.test.yml logs frontend
    exit 1
fi

# Stop test containers
docker-compose -f docker-compose.test.yml down

# Push to registry
echo "Pushing images to registry..."
docker push ${REGISTRY}/${APP_NAME}-backend:${VERSION}
docker push ${REGISTRY}/${APP_NAME}-frontend:${VERSION}

echo "✅ Successfully built and pushed images:"
echo "  - ${REGISTRY}/${APP_NAME}-backend:${VERSION}"
echo "  - ${REGISTRY}/${APP_NAME}-frontend:${VERSION}"

# Tag as latest if version is not latest
if [ "$VERSION" != "latest" ]; then
    docker tag ${REGISTRY}/${APP_NAME}-backend:${VERSION} ${REGISTRY}/${APP_NAME}-backend:latest
    docker tag ${REGISTRY}/${APP_NAME}-frontend:${VERSION} ${REGISTRY}/${APP_NAME}-frontend:latest
    docker push ${REGISTRY}/${APP_NAME}-backend:latest
    docker push ${REGISTRY}/${APP_NAME}-frontend:latest
    echo "✅ Also tagged and pushed as latest"
fi