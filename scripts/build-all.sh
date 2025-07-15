#!/bin/bash
set -e

# Cross-platform build script for dbx-ignore
# Creates binaries for macOS, Linux, and Windows in ./bin/

echo "🔨 Building dbx-ignore for all platforms..."

# Clean and create bin directory
rm -rf bin
mkdir -p bin

# Install required targets (if not already installed)
echo "📦 Installing cross-compilation targets..."
rustup target add x86_64-apple-darwin     # macOS Intel
rustup target add aarch64-apple-darwin    # macOS Apple Silicon  
rustup target add x86_64-unknown-linux-gnu # Linux x64
rustup target add x86_64-pc-windows-gnu    # Windows x64

# Build for each platform
echo "🍎 Building for macOS (Intel)..."
cargo build --release --target x86_64-apple-darwin
cp target/x86_64-apple-darwin/release/dbx-ignore bin/dbx-ignore-macos-intel

echo "🍎 Building for macOS (Apple Silicon)..."
cargo build --release --target aarch64-apple-darwin
cp target/aarch64-apple-darwin/release/dbx-ignore bin/dbx-ignore-macos-arm64

echo "🐧 Building for Linux..."
cargo build --release --target x86_64-unknown-linux-gnu
cp target/x86_64-unknown-linux-gnu/release/dbx-ignore bin/dbx-ignore-linux-x64

echo "🪟 Building for Windows..."
cargo build --release --target x86_64-pc-windows-gnu
cp target/x86_64-pc-windows-gnu/release/dbx-ignore.exe bin/dbx-ignore-windows-x64.exe

# Create universal macOS binary
echo "🔄 Creating universal macOS binary..."
lipo -create \
    bin/dbx-ignore-macos-intel \
    bin/dbx-ignore-macos-arm64 \
    -output bin/dbx-ignore-macos-universal

echo "✅ All builds complete!"
echo ""
echo "📁 Generated binaries:"
ls -la bin/
echo ""
echo "🎯 Universal binaries for distribution:"
echo "  - bin/dbx-ignore-macos-universal (macOS)"
echo "  - bin/dbx-ignore-linux-x64 (Linux)"
echo "  - bin/dbx-ignore-windows-x64.exe (Windows)"