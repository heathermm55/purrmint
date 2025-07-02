#!/bin/bash

# PurrMint Android Run Script
set -e

echo "🚀 Building and running PurrMint Android app..."

# Check if we're in the right directory
if [ ! -f "build.gradle" ]; then
    echo "❌ Error: Please run this script from the purrmint-android directory"
    exit 1
fi

# Build the project
echo "📦 Building project..."
./gradlew assembleDebug

# Check if build was successful
if [ $? -eq 0 ]; then
    echo "✅ Build successful!"
    echo ""
    echo "Next steps:"
    echo "1. Open Android Studio"
    echo "2. Open the purrmint-android folder"
    echo "3. Connect an Android device or start an emulator"
    echo "4. Click 'Run' button or use: ./gradlew installDebug"
    echo ""
    echo "Or run directly on connected device:"
    echo "./gradlew installDebug"
else
    echo "❌ Build failed!"
    exit 1
fi 