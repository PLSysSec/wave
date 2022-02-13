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
build: setfl

.PHONY: clean
clean:
	$(RM) setfl.wasm
	$(RM) setfl.wasm.c
	$(RM) setfl.wasm.h
	$(RM) setfl

setfl.wasm: setfl.c
	$(CC) $(CFLAGS) setfl.c -o setfl.wasm $(LDFLAGS)

setfl.wasm.c: setfl.wasm
	$(WASM2C_BIN_ROOT)/wasm2c -o setfl.wasm.c setfl.wasm

setfl: setfl.wasm.c
	gcc -shared -fPIC -O3 -o setfl setfl.wasm.c -I$(WASM2C_SRC_ROOT) $(WASM2C_SRC_ROOT)/wasm-rt-impl.c $(WASM2C_SRC_ROOT)/wasm-rt-os-unix.c $(WASM2C_SRC_ROOT)/wasm-rt-os-win.c $(WASM2C_SRC_ROOT)/wasm-rt-wasi.c ../../target/release/libwave.so -I../../bindings

run: setfl 
	RUST_LOG=trace $(WASM2C_BIN_ROOT)/wasm2c-runner ./setfl --homedir=.
