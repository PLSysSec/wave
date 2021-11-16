RLBOX_ROOT = ../../rlbox_wasm2c_sandbox
WASM2C_SRC_ROOT = $(RLBOX_ROOT)/build/_deps/mod_wasm2c-src/wasm2c
WASM2C_BIN_ROOT = $(RLBOX_ROOT)/build/_deps/mod_wasm2c-src/bin

WASICLANG_ROOT = $(RLBOX_ROOT)/build/_deps/wasiclang-src/build/install/opt/wasi-sdk
CC = $(WASICLANG_ROOT)/bin/clang                        
CXX = $(WASICLANG_ROOT)/bin/clang++                   
CFLAGS = --sysroot  $(WASICLANG_ROOT)/share/wasi-sysroot/ 
LD = $(WASICLANG_ROOT)/bin/wasm-ld
LDFLAGS = -Wl,--export-all -Wl,--growable-table

build: sync

.PHONY: clean
clean:
	$(RM) sync.wasm
	$(RM) sync.wasm.c
	$(RM) sync.wasm.h
	$(RM) sync

sync.wasm: sync.c
	$(CC) $(CFLAGS) sync.c -o sync.wasm $(LDFLAGS)

sync.wasm.c: sync.wasm
	$(WASM2C_BIN_ROOT)/wasm2c -o sync.wasm.c sync.wasm

sync: sync.wasm.c
	gcc -shared -fPIC -O3 -o sync sync.wasm.c -I$(WASM2C_SRC_ROOT) $(WASM2C_SRC_ROOT)/wasm-rt-impl.c $(WASM2C_SRC_ROOT)/wasm-rt-os-unix.c $(WASM2C_SRC_ROOT)/wasm-rt-os-win.c $(WASM2C_SRC_ROOT)/wasm-rt-wasi.c ../../target/release/libveriwasi.so -I../../bindings

run: sync 
	$(WASM2C_BIN_ROOT)/wasm2c-runner ./sync
