#!/bin/bash

# Android 16KB page size Rust so build script
set -e

export RUSTFLAGS="-C link-arg=-z -C link-arg=max-page-size=16384"

# Only build for arm64-v8a
cargo ndk -t aarch64-linux-android -o ./target build --release

# Copy so to Android project
cp target/aarch64-linux-android/release/libpurrmint.so purrmint-android/app/jniLibs/arm64-v8a/

echo "✅ Rebuilt and copied so file with 16KB page size!"
echo "For multi-arch support, extend this script as needed." 