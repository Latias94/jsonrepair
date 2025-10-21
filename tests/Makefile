# Makefile for C API tests

# Detect OS
ifeq ($(OS),Windows_NT)
    DETECTED_OS := Windows
    LIB_EXT := dll
    EXE_EXT := .exe
    LIB_PATH_VAR := PATH
else
    UNAME_S := $(shell uname -s)
    ifeq ($(UNAME_S),Linux)
        DETECTED_OS := Linux
        LIB_EXT := so
        EXE_EXT :=
        LIB_PATH_VAR := LD_LIBRARY_PATH
        # Embed rpath so the tester finds ../target/release at runtime (no env needed)
        RPATH_FLAG := -Wl,-rpath,'$$ORIGIN/../target/release'
    endif
    ifeq ($(UNAME_S),Darwin)
        DETECTED_OS := macOS
        LIB_EXT := dylib
        EXE_EXT :=
        LIB_PATH_VAR := DYLD_LIBRARY_PATH
        # Embed rpath on macOS as well for robustness
        RPATH_FLAG := -Wl,-rpath,@loader_path/../target/release
    endif
endif

# Paths
LIB_DIR := ../target/release
INCLUDE_DIR := ../include
LIB_NAME := jsonrepair

# Compiler flags
CC := gcc
CFLAGS := -Wall -Wextra -I$(INCLUDE_DIR) -std=c99
LDFLAGS := -L$(LIB_DIR) -l$(LIB_NAME)

# Targets
TEST_BINARY := c_api_test$(EXE_EXT)

.PHONY: all clean build-lib test run

all: build-lib $(TEST_BINARY)

build-lib:
	@echo "Building Rust library with c-api feature..."
	cd .. && cargo build --release --features c-api

$(TEST_BINARY): c_api_test.c
	@echo "Compiling C API tests..."
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS) $(RPATH_FLAG)

test: all
	@echo "Running C API tests..."
ifeq ($(DETECTED_OS),Windows)
	set PATH=$(LIB_DIR);%PATH% && $(TEST_BINARY)
else
	$(LIB_PATH_VAR)=$(LIB_DIR) ./$(TEST_BINARY)
endif

run: test

clean:
	rm -f $(TEST_BINARY)
	rm -f *.o

help:
	@echo "Available targets:"
	@echo "  all        - Build library and compile tests"
	@echo "  build-lib  - Build Rust library with c-api feature"
	@echo "  test       - Build and run tests"
	@echo "  clean      - Remove compiled tests"
	@echo ""
	@echo "Detected OS: $(DETECTED_OS)"
