# Makefile for C examples

# Detect OS
ifeq ($(OS),Windows_NT)
    DETECTED_OS := Windows
    LIB_EXT := dll
    EXE_EXT := .exe
else
    UNAME_S := $(shell uname -s)
    ifeq ($(UNAME_S),Linux)
        DETECTED_OS := Linux
        LIB_EXT := so
        EXE_EXT :=
    endif
    ifeq ($(UNAME_S),Darwin)
        DETECTED_OS := macOS
        LIB_EXT := dylib
        EXE_EXT :=
    endif
endif

# Paths
LIB_DIR := ../../target/release
INCLUDE_DIR := ../../include
LIB_NAME := jsonrepair

# Compiler flags
CC := gcc
CFLAGS := -Wall -Wextra -I$(INCLUDE_DIR)
LDFLAGS := -L$(LIB_DIR) -l$(LIB_NAME)

# Targets
TARGETS := basic$(EXE_EXT) advanced$(EXE_EXT)

.PHONY: all clean build-lib run run-basic run-advanced

all: build-lib $(TARGETS)

build-lib:
	@echo "Building Rust library with c-api feature..."
	cd ../.. && cargo build --release --features c-api

basic$(EXE_EXT): basic.c
	@echo "Compiling $@..."
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS)

advanced$(EXE_EXT): advanced.c
	@echo "Compiling $@..."
	$(CC) $(CFLAGS) -o $@ $< $(LDFLAGS)

run: run-basic run-advanced

run-basic: basic$(EXE_EXT)
	@echo "Running basic example..."
ifeq ($(DETECTED_OS),Windows)
	set PATH=$(LIB_DIR);%PATH% && basic$(EXE_EXT)
else
	LD_LIBRARY_PATH=$(LIB_DIR) ./basic$(EXE_EXT)
endif

run-advanced: advanced$(EXE_EXT)
	@echo ""
	@echo "Running advanced example..."
ifeq ($(DETECTED_OS),Windows)
	set PATH=$(LIB_DIR);%PATH% && advanced$(EXE_EXT)
else
	LD_LIBRARY_PATH=$(LIB_DIR) ./advanced$(EXE_EXT)
endif

clean:
	rm -f $(TARGETS)
	rm -f *.o

help:
	@echo "Available targets:"
	@echo "  all          - Build library and compile all examples"
	@echo "  build-lib    - Build Rust library with c-api feature"
	@echo "  run          - Build and run all examples"
	@echo "  run-basic    - Build and run basic example"
	@echo "  run-advanced - Build and run advanced example"
	@echo "  clean        - Remove compiled examples"
	@echo ""
	@echo "Detected OS: $(DETECTED_OS)"

