#!/bin/bash

# PurrMint Android Build Script
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${GREEN}PurrMint Android Build Script${NC}"
echo "=================================="

# Check if cargo-ndk is installed
if ! command -v cargo-ndk &> /dev/null; then
    echo -e "${YELLOW}Installing cargo-ndk...${NC}"
    cargo install cargo-ndk
fi

# Check if Android NDK is available
if [ -z "$ANDROID_NDK" ]; then
    # Try to find NDK in common locations
    NDK_PATHS=(
        "$HOME/Library/Android/sdk/ndk/25.2.9519653"
        "$HOME/Android/Sdk/ndk/25.2.9519653"
        "/opt/android-ndk"
    )
    
    for path in "${NDK_PATHS[@]}"; do
        if [ -d "$path" ]; then
            export ANDROID_NDK="$path"
            echo -e "${GREEN}Found Android NDK at: $path${NC}"
            break
        fi
    done
    
    if [ -z "$ANDROID_NDK" ]; then
        echo -e "${RED}Error: ANDROID_NDK not set and not found in common locations${NC}"
        echo "Please set ANDROID_NDK environment variable or install Android NDK"
        exit 1
    fi
fi

# Add NDK to PATH
export PATH="$ANDROID_NDK/toolchains/llvm/prebuilt/darwin-x86_64/bin:$PATH"

# Target architectures
TARGETS=(
    "aarch64-linux-android"
    "armv7-linux-androideabi"
    "x86_64-linux-android"
    "i686-linux-android"
)

echo -e "${GREEN}Building for Android targets...${NC}"

# Generate C header file
echo -e "${YELLOW}Generating C header file...${NC}"
cargo cbindgen --config cbindgen.toml

# Build for each target
for target in "${TARGETS[@]}"; do
    echo -e "${YELLOW}Building for $target...${NC}"
    cargo ndk --target $target --release build
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ Built successfully for $target${NC}"
    else
        echo -e "${RED}✗ Build failed for $target${NC}"
        exit 1
    fi
done

echo -e "${GREEN}All builds completed successfully!${NC}"
echo ""
echo "Generated files:"
for target in "${TARGETS[@]}"; do
    echo "  target/$target/release/libpurrmint.so"
done
echo "  include/purrmint.h"

echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "1. Copy .so files to your Android project's jniLibs directory"
echo "2. Copy include/purrmint.h to your Android project's cpp directory"
echo "3. Run tests: cd test && make test" 