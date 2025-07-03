#!/bin/bash

# Universal Android Build Script
# Builds both standard (4KB) and large page (16KB) versions

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}=== PurrMint Universal Android Build Script ===${NC}"
echo -e "${BLUE}Device Whitelist Strategy Implementation${NC}"
echo -e "${YELLOW}- Standard library (4KB pages): Most devices${NC}"
echo -e "${YELLOW}- 16KB library: Whitelisted devices (Pixel, Samsung, OnePlus, etc.)${NC}"
echo -e "${YELLOW}- Automatic fallback if 16KB library fails${NC}"
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

# Clean previous builds
echo -e "${BLUE}Cleaning previous builds...${NC}"
cargo clean

# Clean previous jniLibs directories
echo -e "${BLUE}Cleaning previous jniLibs...${NC}"
rm -rf $ANDROID_PROJECT_DIR/app/jniLibs
rm -rf $ANDROID_PROJECT_DIR/app/src/main/jniLibs

# Build standard version (4KB page size)
echo -e "${BLUE}Building standard version (4KB page size)...${NC}"
for i in "${!RUST_TARGETS[@]}"; do
    target=${RUST_TARGETS[$i]}
    abi=${ANDROID_ABIS[$i]}
    
    echo -e "${YELLOW}Building standard for $target ($abi)...${NC}"
    cargo ndk --target $target --platform 21 build --release --features jni-support
    
    # Create output directory
    mkdir -p $ANDROID_PROJECT_DIR/app/src/main/jniLibs/$abi
    
    # Copy the built library
    cp target/$target/release/libpurrmint.so $ANDROID_PROJECT_DIR/app/src/main/jniLibs/$abi/
    
    echo -e "${GREEN}✓ Built standard for $abi${NC}"
done

# Build large page version (16KB page size)
echo -e "${BLUE}Building large page version (16KB page size)...${NC}"
export RUSTFLAGS="-C link-arg=-z -C link-arg=max-page-size=16384"

for i in "${!RUST_TARGETS[@]}"; do
    target=${RUST_TARGETS[$i]}
    abi=${ANDROID_ABIS[$i]}
    
    echo -e "${YELLOW}Building large page for $target ($abi)...${NC}"
    cargo ndk --target $target --platform 21 build --release --features jni-support
    
    # Copy with different name
    cp target/$target/release/libpurrmint.so $ANDROID_PROJECT_DIR/app/src/main/jniLibs/$abi/libpurrmint_16k.so
    
    echo -e "${GREEN}✓ Built large page for $abi${NC}"
done

# Reset RUSTFLAGS
unset RUSTFLAGS

echo -e "${GREEN}✓ All libraries built successfully${NC}"

# Create CMakeLists.txt if it doesn't exist
if [ ! -f "$ANDROID_PROJECT_DIR/app/src/main/cpp/CMakeLists.txt" ]; then
    echo -e "${BLUE}Creating CMakeLists.txt...${NC}"
    mkdir -p $ANDROID_PROJECT_DIR/app/src/main/cpp
    
    cat > $ANDROID_PROJECT_DIR/app/src/main/cpp/CMakeLists.txt << 'EOF'
cmake_minimum_required(VERSION 3.18.1)

project("purrmint")

# Add standard Rust library
add_library(purrmint SHARED IMPORTED)
set_target_properties(purrmint PROPERTIES IMPORTED_LOCATION
    ${CMAKE_SOURCE_DIR}/../jniLibs/${ANDROID_ABI}/libpurrmint.so)

# Add large page Rust library
add_library(purrmint_16k SHARED IMPORTED)
set_target_properties(purrmint_16k PROPERTIES IMPORTED_LOCATION
    ${CMAKE_SOURCE_DIR}/../jniLibs/${ANDROID_ABI}/libpurrmint_16k.so)

# Link library (Android will choose appropriate one)
target_link_libraries(purrmint)
EOF
    
    echo -e "${GREEN}✓ CMakeLists.txt created${NC}"
fi

echo -e "${BLUE}=== Build Summary ===${NC}"
echo -e "${GREEN}✓ Standard libraries (4KB): libpurrmint.so${NC}"
echo -e "${GREEN}✓ Large page libraries (16KB): libpurrmint_16k.so${NC}"
echo -e "${GREEN}✓ Libraries copied to: $ANDROID_PROJECT_DIR/app/src/main/jniLibs/${NC}"
echo -e "${GREEN}✓ CMakeLists.txt configured${NC}"
echo -e "${BLUE}Next steps:${NC}"
echo -e "1. Open $ANDROID_PROJECT_DIR in Android Studio"
echo -e "2. Sync project with Gradle files"
echo -e "3. Build and run the Android app"
echo -e ""
echo -e "${YELLOW}Note: The app uses device whitelist to automatically choose the appropriate library${NC}"
echo -e "${YELLOW}Whitelisted devices: Google Pixel, Samsung Galaxy S/Note, OnePlus, Xiaomi, OPPO, Vivo${NC}"

echo -e "${GREEN}=== Universal build completed successfully! ===${NC}" 