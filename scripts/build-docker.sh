#!/bin/bash
set -e

echo "🐳 Building Linux and Windows binaries using Docker..."

# Check if Docker is available
if ! command -v docker &> /dev/null; then
    echo "❌ Docker not found. Please install Docker to build Linux/Windows binaries."
    exit 1
fi

# Create bin directory
mkdir -p bin

# Build the Docker image and extract binaries
echo "📦 Building cross-compilation Docker image..."
docker build -f Dockerfile.cross -t dbx-ignore-cross .

echo "📤 Extracting binaries..."
docker run --rm -v "$(pwd)/bin:/host" dbx-ignore-cross

echo "✅ Docker build complete!"
echo ""
echo "📁 Generated binaries:"
ls -la bin/ | grep -E "(linux|windows)"