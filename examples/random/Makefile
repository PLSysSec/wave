RLBOX_ROOT = ../../rlbox_wasm2c_sandbox
WASM2C_SRC_ROOT = $(RLBOX_ROOT)/build/_deps/mod_wasm2c-src/wasm2c
WASM2C_BIN_ROOT = $(RLBOX_ROOT)/build/_deps/mod_wasm2c-src/bin

WASICLANG_ROOT = $(RLBOX_ROOT)/build/_deps/wasiclang-src/build/install/opt/wasi-sdk
CC = $(WASICLANG_ROOT)/bin/clang                        
CXX = $(WASICLANG_ROOT)/bin/clang++                   
CFLAGS = --sysroot  $(WASICLANG_ROOT)/share/wasi-sysroot/ 
LD = $(WASICLANG_ROOT)/bin/wasm-ld
LDFLAGS = -Wl,--export-all -Wl,--growable-table

build: random

.PHONY: clean
clean:
	$(RM) random.wasm
	$(RM) random.wasm.c
	$(RM) random.wasm.h
	$(RM) random

random.wasm: random.c
	$(CC) $(CFLAGS) random.c -o random.wasm $(LDFLAGS)

random.wasm.c: random.wasm
	$(WASM2C_BIN_ROOT)/wasm2c -o random.wasm.c random.wasm

random: random.wasm.c
	gcc -shared -fPIC -O3 -o random random.wasm.c -I$(WASM2C_SRC_ROOT) $(WASM2C_SRC_ROOT)/wasm-rt-impl.c $(WASM2C_SRC_ROOT)/wasm-rt-os-unix.c $(WASM2C_SRC_ROOT)/wasm-rt-os-win.c $(WASM2C_SRC_ROOT)/wasm-rt-wasi.c ../../target/release/libveriwasi.so -I../../bindings

run: random 
	$(WASM2C_BIN_ROOT)/wasm2c-runner ./random
