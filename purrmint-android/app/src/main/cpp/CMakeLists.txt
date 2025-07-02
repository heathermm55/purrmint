cmake_minimum_required(VERSION 3.22.1)

project("purrmint-android")

# Set C++ standard
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Build JNI wrapper
add_library(purrmint-jni SHARED
    purrmint_jni.cpp)

# Include directories
target_include_directories(purrmint-jni PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR})

# Link libraries - specify library path for linking
target_link_libraries(purrmint-jni
    "${CMAKE_SOURCE_DIR}/../jniLibs/${ANDROID_ABI}/libpurrmint.so"
    android
    log) 