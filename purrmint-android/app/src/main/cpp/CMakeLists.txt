cmake_minimum_required(VERSION 3.22.1)

project("purrmint-android")

# Set C++ standard
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# Add the prebuilt library
add_library(purrmint SHARED IMPORTED)
set_target_properties(purrmint PROPERTIES IMPORTED_LOCATION
    ${CMAKE_CURRENT_SOURCE_DIR}/../../../jniLibs/${ANDROID_ABI}/libpurrmint.so)

# Create the JNI library
add_library(purrmint-jni SHARED
    purrmint_jni.cpp)

# Include directories
target_include_directories(purrmint-jni PRIVATE
    ${CMAKE_CURRENT_SOURCE_DIR})

# Link libraries
target_link_libraries(purrmint-jni
    purrmint
    android
    log) 