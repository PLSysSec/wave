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

build: sleep

.PHONY: clean
clean:
	$(RM) sleep.wasm
	$(RM) sleep.wasm.c
	$(RM) sleep.wasm.h
	$(RM) sleep

sleep.wasm: sleep.c
	$(CC) $(CFLAGS) sleep.c -o sleep.wasm $(LDFLAGS)

sleep.wasm.c: sleep.wasm
	$(WASM2C_BIN_ROOT)/wasm2c -o sleep.wasm.c sleep.wasm

sleep: sleep.wasm.c
	gcc -shared -fPIC -O3 -o sleep sleep.wasm.c -I$(WASM2C_SRC_ROOT) $(WASM2C_SRC_ROOT)/wasm-rt-impl.c $(WASM2C_SRC_ROOT)/wasm-rt-os-unix.c $(WASM2C_SRC_ROOT)/wasm-rt-os-win.c $(WASM2C_SRC_ROOT)/wasm-rt-wasi.c ../../target/release/libwave.so -I../../bindings

run: sleep 
	RUST_LOG=trace $(WASM2C_BIN_ROOT)/wasm2c-runner ./sleep --homedir=.
