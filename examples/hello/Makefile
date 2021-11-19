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

build: hello

.PHONY: clean
clean:
	$(RM) hello.wasm
	$(RM) hello.wasm.c
	$(RM) hello.wasm.h
	$(RM) hello

hello.wasm: hello.c
	$(CC) $(CFLAGS) hello.c -o hello.wasm $(LDFLAGS)

hello.wasm.c: hello.wasm
	$(WASM2C_BIN_ROOT)/wasm2c -o hello.wasm.c hello.wasm

hello: hello.wasm.c
	gcc -shared -fPIC -O3 -o hello hello.wasm.c -I$(WASM2C_SRC_ROOT) $(WASM2C_SRC_ROOT)/wasm-rt-impl.c $(WASM2C_SRC_ROOT)/wasm-rt-os-unix.c $(WASM2C_SRC_ROOT)/wasm-rt-os-win.c $(WASM2C_SRC_ROOT)/wasm-rt-wasi.c ../../target/release/libveriwasi.so -I../../bindings

run: hello 
	RUST_LOG=trace $(WASM2C_BIN_ROOT)/wasm2c-runner ./hello
