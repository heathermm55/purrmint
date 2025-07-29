#!/bin/bash

# Android NDK 路径
ANDROID_NDK_ROOT="/Users/zzz/Library/Android/sdk/ndk/25.1.8937393"
LLVM_ARCH_PATH="darwin-x86_64"

# 设置环境变量
export ANDROID_NDK_ROOT
export PATH="$ANDROID_NDK_ROOT/toolchains/llvm/prebuilt/$LLVM_ARCH_PATH/bin:$PATH"

# 添加 Android 目标
rustup target add aarch64-linux-android

# 设置编译器环境变量
export CFLAGS="-D__ANDROID_MIN_SDK_VERSION__=21"
export AR="llvm-ar"
export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER="aarch64-linux-android21-clang"
export CC="aarch64-linux-android21-clang"

# 清理之前的构建
cargo clean

# 构建
echo "Building for aarch64-linux-android..."
cargo build --profile release --target aarch64-linux-android --features jni-support,tor

echo "Build completed!" 