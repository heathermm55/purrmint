#!/bin/bash

# PurrMint Android Build Script
# This script builds the Rust library for Android and prepares the Android project

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
RUST_TARGETS=("aarch64-linux-android" "armv7-linux-androideabi" "i686-linux-android" "x86_64-linux-android")
ANDROID_ABIS=("arm64-v8a" "armeabi-v7a" "x86" "x86_64")
ANDROID_PROJECT_DIR="purrmint-android"
RUST_LIB_DIR="target"

echo -e "${BLUE}=== PurrMint Android Build Script ===${NC}"

# Check if cargo-ndk is installed
if ! command -v cargo-ndk &> /dev/null; then
    echo -e "${YELLOW}cargo-ndk not found. Installing...${NC}"
    cargo install cargo-ndk
fi

# Check if Android NDK is set
if [ -z "$ANDROID_NDK_HOME" ]; then
    echo -e "${RED}ANDROID_NDK_HOME is not set. Please set it to your Android NDK path.${NC}"
    echo -e "${YELLOW}Example: export ANDROID_NDK_HOME=/path/to/android-ndk${NC}"
    exit 1
fi

echo -e "${GREEN}Using Android NDK: $ANDROID_NDK_HOME${NC}"

# Clean previous builds
echo -e "${BLUE}Cleaning previous builds...${NC}"
cargo clean

# Build Rust library for each target
echo -e "${BLUE}Building Rust library for Android targets...${NC}"

for i in "${!RUST_TARGETS[@]}"; do
    target=${RUST_TARGETS[$i]}
    abi=${ANDROID_ABIS[$i]}
    
    echo -e "${YELLOW}Building for $target ($abi)...${NC}"
    
    # Build with cargo-ndk
    cargo ndk --target $target --platform 21 build --release
    
    # Create output directory
    mkdir -p $ANDROID_PROJECT_DIR/app/src/main/jniLibs/$abi
    
    # Copy the built library
    cp $RUST_LIB_DIR/$target/release/libpurrmint.so $ANDROID_PROJECT_DIR/app/src/main/jniLibs/$abi/
    
    echo -e "${GREEN}✓ Built for $abi${NC}"
done

echo -e "${GREEN}✓ All Rust libraries built successfully${NC}"

# Check if Android project exists
if [ ! -d "$ANDROID_PROJECT_DIR" ]; then
    echo -e "${RED}Android project directory not found: $ANDROID_PROJECT_DIR${NC}"
    exit 1
fi

# Copy CMakeLists.txt if it doesn't exist
if [ ! -f "$ANDROID_PROJECT_DIR/app/src/main/cpp/CMakeLists.txt" ]; then
    echo -e "${BLUE}Creating CMakeLists.txt...${NC}"
    mkdir -p $ANDROID_PROJECT_DIR/app/src/main/cpp
    
    cat > $ANDROID_PROJECT_DIR/app/src/main/cpp/CMakeLists.txt << 'EOF'
cmake_minimum_required(VERSION 3.18.1)

project("purrmint")

# Add Rust library
add_library(purrmint SHARED IMPORTED)
set_target_properties(purrmint PROPERTIES IMPORTED_LOCATION
    ${CMAKE_SOURCE_DIR}/../jniLibs/${ANDROID_ABI}/libpurrmint.so)

# Link library
target_link_libraries(purrmint)
EOF
    
    echo -e "${GREEN}✓ CMakeLists.txt created${NC}"
fi

# Check if gradle wrapper exists
if [ ! -f "$ANDROID_PROJECT_DIR/gradlew" ]; then
    echo -e "${YELLOW}Gradle wrapper not found. You need to run 'gradle wrapper' in the Android project directory.${NC}"
    echo -e "${BLUE}Or use Android Studio to generate the project files.${NC}"
fi

echo -e "${BLUE}=== Build Summary ===${NC}"
echo -e "${GREEN}✓ Rust libraries built for all Android ABIs${NC}"
echo -e "${GREEN}✓ Libraries copied to: $ANDROID_PROJECT_DIR/app/src/main/jniLibs/${NC}"
echo -e "${GREEN}✓ CMakeLists.txt configured${NC}"
echo -e "${BLUE}Next steps:${NC}"
echo -e "1. Open $ANDROID_PROJECT_DIR in Android Studio"
echo -e "2. Sync project with Gradle files"
echo -e "3. Build and run the Android app"
echo -e ""
echo -e "${YELLOW}Note: Make sure you have the following in your app's build.gradle:${NC}"
echo -e "android {"
echo -e "    defaultConfig {"
echo -e "        ndk {"
echo -e "            abiFilters 'arm64-v8a', 'armeabi-v7a', 'x86', 'x86_64'"
echo -e "        }"
echo -e "    }"
echo -e "    externalNativeBuild {"
echo -e "        cmake {"
echo -e "            path \"src/main/cpp/CMakeLists.txt\""
echo -e "        }"
echo -e "    }"
echo -e "}"

echo -e "${GREEN}=== Build completed successfully! ===${NC}" 