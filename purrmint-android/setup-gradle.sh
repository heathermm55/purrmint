#!/bin/bash

# Setup Gradle Wrapper
set -e

echo "Setting up Gradle Wrapper..."

# Create gradle wrapper directory
mkdir -p gradle/wrapper

# Download gradle-wrapper.jar
echo "Downloading gradle-wrapper.jar..."
curl -L -o gradle/wrapper/gradle-wrapper.jar \
  "https://github.com/gradle/gradle/raw/v8.2.0/gradle/wrapper/gradle-wrapper.jar"

# Create gradle-wrapper.properties
echo "Creating gradle-wrapper.properties..."
cat > gradle/wrapper/gradle-wrapper.properties << EOF
distributionBase=GRADLE_USER_HOME
distributionPath=wrapper/dists
distributionUrl=https\://services.gradle.org/distributions/gradle-8.2-bin.zip
networkTimeout=10000
validateDistributionUrl=true
zipStoreBase=GRADLE_USER_HOME
zipStorePath=wrapper/dists
EOF

echo "Gradle Wrapper setup complete!"
echo "You can now run: ./gradlew assembleDebug" 