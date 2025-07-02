# Makefile for testing PurrMint FFI
# Set your Android NDK path here
ANDROID_NDK ?= $(HOME)/Library/Android/sdk/ndk/25.2.9519653

# Target architecture
TARGET_ARCH ?= aarch64-linux-android
API_LEVEL ?= 21

# Toolchain paths
TOOLCHAIN = $(ANDROID_NDK)/toolchains/llvm/prebuilt/darwin-x86_64
CC = $(TOOLCHAIN)/bin/clang
AR = $(TOOLCHAIN)/bin/llvm-ar

# Compiler flags
CFLAGS = --target=$(TARGET_ARCH)$(API_LEVEL) \
         -fPIC \
         -Wall \
         -Wextra \
         -std=c99

# Linker flags
LDFLAGS = -shared \
          -Wl,--no-undefined \
          -Wl,--version-script=version.script

# Directories
LIB_DIR = ../target/$(TARGET_ARCH)/release
INCLUDE_DIR = ../include
TEST_DIR = .

# Files
LIB_NAME = purrmint
TEST_BIN = test_ffi
TEST_SRC = $(TEST_DIR)/test.c

.PHONY: all clean test

all: $(TEST_BIN)

# Generate header file first
$(INCLUDE_DIR)/purrmint.h:
	cd .. && cargo cbindgen --config cbindgen.toml

# Build the test binary
$(TEST_BIN): $(TEST_SRC) $(INCLUDE_DIR)/purrmint.h
	$(CC) $(CFLAGS) -o $@ $< \
		-I$(INCLUDE_DIR) \
		-L$(LIB_DIR) -l$(LIB_NAME) \
		-Wl,-rpath,$(LIB_DIR)

test: $(TEST_BIN)
	@echo "Running FFI tests..."
	@if [ -f $(LIB_DIR)/lib$(LIB_NAME).so ]; then \
		LD_LIBRARY_PATH=$(LIB_DIR) ./$(TEST_BIN); \
	else \
		echo "Error: lib$(LIB_NAME).so not found. Please build it first with:"; \
		echo "  cd .. && cargo ndk --target $(TARGET_ARCH) --release build"; \
		exit 1; \
	fi

clean:
	rm -f $(TEST_BIN)

help:
	@echo "Available targets:"
	@echo "  all     - Build test binary"
	@echo "  test    - Run FFI tests"
	@echo "  clean   - Clean build artifacts"
	@echo ""
	@echo "Before running tests, make sure to:"
	@echo "  1. Set ANDROID_NDK environment variable"
	@echo "  2. Build the library: cd .. && cargo ndk --target $(TARGET_ARCH) --release build" 