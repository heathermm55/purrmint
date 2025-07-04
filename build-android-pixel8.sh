#!/bin/bash

# Quick Android Build Script for Pixel 8 Emulator (arm64-v8a only)
# Fast build for testing and verification

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== PurrMint Quick Build for Pixel 8 Emulator ===${NC}"
echo -e "${YELLOW}Target: arm64-v8a (aarch64-linux-android)${NC}"
echo

# Check if Android NDK is set
if [ -z "$ANDROID_NDK_HOME" ]; then
    echo -e "${RED}ANDROID_NDK_HOME is not set. Please set it to your Android NDK path.${NC}"
    echo -e "${YELLOW}Example: export ANDROID_NDK_HOME=/path/to/android-ndk${NC}"
    exit 1
fi

echo -e "${GREEN}Using Android NDK: $ANDROID_NDK_HOME${NC}"

# Configuration for Pixel 8 emulator only
RUST_TARGET="aarch64-linux-android"
ANDROID_ABI="arm64-v8a"
ANDROID_PROJECT_DIR="purrmint-android"

# Clean previous builds for this target only
# echo -e "${BLUE}Cleaning previous builds for $RUST_TARGET...${NC}"
# cargo clean --target $RUST_TARGET

# Clean previous jniLibs directories
echo -e "${BLUE}Cleaning previous jniLibs...${NC}"
rm -rf $ANDROID_PROJECT_DIR/app/jniLibs
rm -rf $ANDROID_PROJECT_DIR/app/src/main/jniLibs

# Build standard version (4KB page size)
echo -e "${BLUE}Building standard version (4KB page size) for $RUST_TARGET...${NC}"
cargo ndk --target $RUST_TARGET --platform 21 build --release --features jni-support

# Create output directory
mkdir -p $ANDROID_PROJECT_DIR/app/src/main/jniLibs/$ANDROID_ABI

# Copy the built library
cp target/$RUST_TARGET/release/libpurrmint.so $ANDROID_PROJECT_DIR/app/src/main/jniLibs/$ANDROID_ABI/

echo -e "${GREEN}✓ Built standard for $ANDROID_ABI${NC}"

# Build large page version (16KB page size)
echo -e "${BLUE}Building large page version (16KB page size) for $RUST_TARGET...${NC}"
export RUSTFLAGS="-C link-arg=-z -C link-arg=max-page-size=16384"

cargo ndk --target $RUST_TARGET --platform 21 build --release --features jni-support

# Copy with different name
cp target/$RUST_TARGET/release/libpurrmint.so $ANDROID_PROJECT_DIR/app/src/main/jniLibs/$ANDROID_ABI/libpurrmint_16k.so

# Reset RUSTFLAGS
unset RUSTFLAGS

echo -e "${GREEN}✓ Built large page for $ANDROID_ABI${NC}"

echo -e "${BLUE}=== Build Summary ===${NC}"
echo -e "${GREEN}✓ Standard library (4KB): libpurrmint.so${NC}"
echo -e "${GREEN}✓ Large page library (16KB): libpurrmint_16k.so${NC}"
echo -e "${GREEN}✓ Libraries copied to: $ANDROID_PROJECT_DIR/app/src/main/jniLibs/$ANDROID_ABI/${NC}"
echo -e "${BLUE}Next steps:${NC}"
echo -e "1. Open $ANDROID_PROJECT_DIR in Android Studio"
echo -e "2. Sync project with Gradle files"
echo -e "3. Build and run the Android app on Pixel 8 emulator"
echo -e ""
echo -e "${YELLOW}Note: This build is optimized for Pixel 8 emulator testing${NC}"

echo -e "${GREEN}=== Quick build completed successfully! ===${NC}" 