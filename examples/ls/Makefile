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

build: ls

.PHONY: clean
clean:
	$(RM) ls.wasm
	$(RM) ls.wasm.c
	$(RM) ls.wasm.h
	$(RM) ls

ls.wasm: ls.c
	$(CC) $(CFLAGS) ls.c -o ls.wasm $(LDFLAGS)

ls.wasm.c: ls.wasm
	$(WASM2C_BIN_ROOT)/wasm2c -o ls.wasm.c ls.wasm

ls: ls.wasm.c
	gcc -shared -fPIC -O3 -o ls ls.wasm.c -I$(WASM2C_SRC_ROOT) $(WASM2C_SRC_ROOT)/wasm-rt-impl.c $(WASM2C_SRC_ROOT)/wasm-rt-os-unix.c $(WASM2C_SRC_ROOT)/wasm-rt-os-win.c $(WASM2C_SRC_ROOT)/wasm-rt-wasi.c ../../target/release/libveriwasi.so -I../../bindings

run: ls 
	RUST_LOG=trace $(WASM2C_BIN_ROOT)/wasm2c-runner ./ls --homedir=.
