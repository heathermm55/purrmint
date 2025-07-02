#!/bin/bash

# PurrMint Android App Build Script
# This script builds the Rust library and Android app

set -e

echo "🚀 Starting PurrMint Android App build..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "build.gradle" ]; then
    print_error "Please run this script from the purrmint-android directory"
    exit 1
fi

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    print_error "Rust is not installed. Please install Rust first: https://rustup.rs/"
    exit 1
fi

# Check if Android NDK is available
if [ -z "$ANDROID_NDK_HOME" ]; then
    print_warning "ANDROID_NDK_HOME is not set. Please set it to your Android NDK path."
    print_warning "You can set it by adding this to your ~/.bashrc or ~/.zshrc:"
    print_warning "export ANDROID_NDK_HOME=/path/to/your/android-ndk"
    exit 1
fi

print_status "Building Rust library for Android..."

# Build Rust library for Android with JNI support
cd ../
cargo ndk -t arm64-v8a -o purrmint-android/app/src/main/jniLibs build --release --features jni-support

# print_status "Copying Rust library to Android project..."
# cp ../target/aarch64-linux-android/release/libpurrmint_jni.so app/src/main/jniLibs/arm64-v8a/

# Build Android app
print_status "Building Android app..."

# Check if gradlew exists
if [ ! -f "gradlew" ]; then
    print_error "gradlew not found. Please run 'gradle wrapper' first or use Android Studio."
    exit 1
fi

# Make gradlew executable
chmod +x gradlew

# Build the app
./gradlew assembleDebug

if [ $? -eq 0 ]; then
    print_status "✅ Android app built successfully!"
    print_status "APK location: app/build/outputs/apk/debug/app-debug.apk"
    print_status "You can now install it on your device or run it in Android Studio."
else
    print_error "Failed to build Android app"
    exit 1
fi

print_status "🎉 Build completed successfully!"
print_status "To install on device: adb install app/build/outputs/apk/debug/app-debug.apk" 