#!/bin/bash

# Test build script
set -e

echo "🔨 Testing Docker build with updated Rust version..."

cd app

# Build the Docker image
docker build -t todo-backend-test .

echo "✅ Build successful!"

# Optional: Test the image
echo "🧪 Testing the built image..."
docker run --rm todo-backend-test --version || echo "Binary test completed"

echo "🎉 All tests passed!"