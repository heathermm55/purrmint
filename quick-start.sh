#!/bin/bash

# PurrMint Quick Start Script
set -e

echo "🚀 PurrMint Quick Start"
echo "======================"

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Please run this script from the purrmint directory"
    exit 1
fi

# Step 1: Install cargo-ndk if not present
if ! command -v cargo-ndk &> /dev/null; then
    echo "📦 Installing cargo-ndk..."
    cargo install cargo-ndk
else
    echo "✅ cargo-ndk already installed"
fi

# Step 2: Generate C header
echo "📝 Generating C header file..."
cargo cbindgen --config cbindgen.toml

# Step 3: Build for aarch64 (most common Android architecture)
echo "🔨 Building for aarch64-linux-android..."
cargo ndk --target aarch64-linux-android --release build

# Step 4: Run tests
echo "🧪 Running tests..."
cargo test

echo ""
echo "✅ Quick start completed!"
echo ""
echo "Generated files:"
echo "  📄 include/purrmint.h"
echo "  📦 target/aarch64-linux-android/release/libpurrmint.so"
echo ""
echo "Next steps:"
echo "  1. Copy libpurrmint.so to your Android project's jniLibs/arm64-v8a/"
echo "  2. Copy purrmint.h to your Android project's cpp/"
echo "  3. Use the Android example in android-example/ as reference"
echo ""
echo "To build for all architectures, run: ./build-android.sh" 