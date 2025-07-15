#!/bin/bash
set -e

# Simple cross-platform build script that works around OpenSSL issues
echo "🔨 Building dbx-ignore distribution binaries..."

# Clean and create bin directory
rm -rf bin
mkdir -p bin

# Build for current platform first (this always works)
echo "🏠 Building for current platform..."
cargo build --release

# Try to build for other macOS target (Intel <-> ARM)
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo "🍎 Attempting macOS cross-compilation..."
    
    # Detect current architecture and build for the other one
    if [[ "$(uname -m)" == "x86_64" ]]; then
        echo "  Current: Intel, building for Apple Silicon..."
        if cargo build --release --target aarch64-apple-darwin 2>/dev/null; then
            cp target/aarch64-apple-darwin/release/dbx-ignore bin/dbx-ignore-macos-arm64
            echo "  ✅ Apple Silicon build successful"
        else
            echo "  ⚠️  Apple Silicon build failed (may need Xcode tools)"
        fi
        cp target/release/dbx-ignore bin/dbx-ignore-macos-intel
    else
        echo "  Current: Apple Silicon, building for Intel..."
        if cargo build --release --target x86_64-apple-darwin 2>/dev/null; then
            cp target/x86_64-apple-darwin/release/dbx-ignore bin/dbx-ignore-macos-intel
            echo "  ✅ Intel build successful"
        else
            echo "  ⚠️  Intel build failed (may need Xcode tools)"
        fi
        cp target/release/dbx-ignore bin/dbx-ignore-macos-arm64
    fi
    
    # Create universal binary if both exist
    if [[ -f "bin/dbx-ignore-macos-intel" && -f "bin/dbx-ignore-macos-arm64" ]]; then
        echo "🔄 Creating universal macOS binary..."
        lipo -create \
            bin/dbx-ignore-macos-intel \
            bin/dbx-ignore-macos-arm64 \
            -output bin/dbx-ignore-macos-universal
        echo "  ✅ Universal binary created"
    fi
fi

echo ""
echo "✅ Build complete!"
echo ""
echo "📁 Generated binaries:"
ls -la bin/
echo ""
echo "💡 For development/testing: Use ./target/release/dbx-ignore"
echo "💡 For distribution: Use binaries from ./bin/"
echo "💡 For Linux/Windows builds: Use GitHub Actions or Docker"