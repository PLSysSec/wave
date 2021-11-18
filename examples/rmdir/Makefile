RLBOX_ROOT = ../../rlbox_wasm2c_sandbox
WASM2C_SRC_ROOT = $(RLBOX_ROOT)/build/_deps/mod_wasm2c-src/wasm2c
WASM2C_BIN_ROOT = $(RLBOX_ROOT)/build/_deps/mod_wasm2c-src/bin

WASICLANG_ROOT = $(RLBOX_ROOT)/build/_deps/wasiclang-src/build/install/opt/wasi-sdk
WASILIBC_ROOT =  $(RLBOX_ROOT)/build/_deps/wasiclang-src/src/wasi-libc
CC = $(WASICLANG_ROOT)/bin/clang                        
CXX = $(WASICLANG_ROOT)/bin/clang++                   
CFLAGS = --sysroot  $(WASILIBC_ROOT)/sysroot/ 
LD = $(WASICLANG_ROOT)/bin/wasm-ld
LDFLAGS = -Wl,--export-all -Wl,--growable-table

build: rmdir

.PHONY: clean
clean:
	$(RM) rmdir.wasm
	$(RM) rmdir.wasm.c
	$(RM) rmdir.wasm.h
	$(RM) rmdir

rmdir.wasm: rmdir.c
	$(CC) $(CFLAGS) rmdir.c -o rmdir.wasm $(LDFLAGS)

rmdir.wasm.c: rmdir.wasm
	$(WASM2C_BIN_ROOT)/wasm2c -o rmdir.wasm.c rmdir.wasm

rmdir: rmdir.wasm.c
	gcc -shared -fPIC -O3 -o rmdir rmdir.wasm.c -I$(WASM2C_SRC_ROOT) $(WASM2C_SRC_ROOT)/wasm-rt-impl.c $(WASM2C_SRC_ROOT)/wasm-rt-os-unix.c $(WASM2C_SRC_ROOT)/wasm-rt-os-win.c $(WASM2C_SRC_ROOT)/wasm-rt-wasi.c ../../target/release/libveriwasi.so -I../../bindings

run: rmdir 
	$(WASM2C_BIN_ROOT)/wasm2c-runner ./rmdir
