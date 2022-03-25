WASI_SDK_ROOT=../../tools/wasi-sdk
WASI_SDK_INSTALL=$(WASI_SDK_ROOT)/build/install/opt/wasi-sdk/
WASM2C_ROOT=../../tools/wasm2c_sandbox_compiler

WASM2C_SRC_ROOT = $(WASM2C_ROOT)/wasm2c
WASM2C_BIN_ROOT = $(WASM2C_ROOT)/bin

# Added this cause build directories were different on my mac
# This way both mac and linux should "Just work"
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S), Darwin)
WASILIBC_ROOT =  $(WASI_SDK_INSTALL)/share
DYLIB_EXT = dylib
endif
ifeq ($(UNAME_S), Linux)
DYLIB_EXT = so
endif

CFLAGS = --sysroot $(WASI_SDK_INSTALL)/share/wasi-sysroot/

CC = $(WASI_SDK_INSTALL)/bin/clang
CXX = $(WASI_SDK_INSTALL)/bin/clang++
LD = $(WASI_SDK_INSTALL)/bin/wasm-ld
LDFLAGS = -Wl,--export-all -Wl,--growable-table

$(call check_defined, TARGET)

.PHONY: build
build: $(TARGET)

.PHONY: clean
clean:
	$(RM) $(TARGET).wasm
	$(RM) $(TARGET).wasm.c
	$(RM) $(TARGET).wasm.h
	$(RM) $(TARGET) 

%.wasm: %.c
	$(CC) $(CFLAGS) $< -o $@ $(LDFLAGS)

%.wasm.c: %.wasm
	$(WASM2C_BIN_ROOT)/wasm2c -o $@ $<

$(TARGET): $(TARGET).wasm.c
	gcc -shared -fPIC -O3 -o $@ $< -I$(WASM2C_SRC_ROOT) $(WASM2C_SRC_ROOT)/wasm-rt-impl.c $(WASM2C_SRC_ROOT)/wasm-rt-os-unix.c $(WASM2C_SRC_ROOT)/wasm-rt-os-win.c $(WASM2C_SRC_ROOT)/wasm-rt-wasi.c ../../target/release/libwave.$(DYLIB_EXT) -I../../bindings

.PHONY: run
run: $(TARGET)
	$(WASM2C_BIN_ROOT)/wasm2c-runner ./$< --homedir=.
