#!/bin/bash

# PurrMint Android 16KB Page Size Build Script
# Build only 16KB page size .so for all major Android devices

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== PurrMint Android 16KB Page Size Build Script ===${NC}"

echo -e "${YELLOW}Only building 16KB page size .so for all major Android devices${NC}"
echo

# Check if Android NDK is set
if [ -z "$ANDROID_NDK_HOME" ]; then
    echo -e "${RED}ANDROID_NDK_HOME is not set. Please set it to your Android NDK path.${NC}"
    echo -e "${YELLOW}Example: export ANDROID_NDK_HOME=/path/to/android-ndk${NC}"
    exit 1
fi

echo -e "${GREEN}Using Android NDK: $ANDROID_NDK_HOME${NC}"

# Configuration
RUST_TARGETS=("aarch64-linux-android" "armv7-linux-androideabi" "i686-linux-android" "x86_64-linux-android")
ANDROID_ABIS=("arm64-v8a" "armeabi-v7a" "x86" "x86_64")
ANDROID_PROJECT_DIR="purrmint-android"

# Clean previous jniLibs directories
echo -e "${BLUE}Cleaning previous jniLibs...${NC}"
rm -rf $ANDROID_PROJECT_DIR/app/jniLibs
rm -rf $ANDROID_PROJECT_DIR/app/src/main/jniLibs

# Build 16KB page size version (only)
echo -e "${BLUE}Building 16KB page size version...${NC}"
export RUSTFLAGS="-C link-arg=-z -C link-arg=max-page-size=16384"

for i in "${!RUST_TARGETS[@]}"; do
    target=${RUST_TARGETS[$i]}
    abi=${ANDROID_ABIS[$i]}
    echo -e "${YELLOW}Building 16KB for $target ($abi)...${NC}"
    cargo ndk --target $target --platform 21 build --release --features jni-support
    mkdir -p $ANDROID_PROJECT_DIR/app/src/main/jniLibs/$abi
    cp target/$target/release/libpurrmint.so $ANDROID_PROJECT_DIR/app/src/main/jniLibs/$abi/
    echo -e "${GREEN}✓ Built 16KB for $abi${NC}"
done

unset RUSTFLAGS

echo -e "${GREEN}✓ All 16KB libraries built and copied successfully${NC}"

echo -e "${BLUE}=== Build Summary ===${NC}"
echo -e "${GREEN}✓ 16KB page size libraries: libpurrmint.so${NC}"
echo -e "${GREEN}✓ Libraries copied to: $ANDROID_PROJECT_DIR/app/src/main/jniLibs/${NC}"
echo -e "${BLUE}Next steps:${NC}"
echo -e "1. Open $ANDROID_PROJECT_DIR in Android Studio"
echo -e "2. Sync project with Gradle files"
echo -e "3. Build and run the Android app"
echo -e ""
echo -e "${YELLOW}Note: 16KB page size .so covers all ABIs and is compatible with all major Android devices${NC}"
echo -e "${GREEN}=== 16KB build completed successfully! ===${NC}" 