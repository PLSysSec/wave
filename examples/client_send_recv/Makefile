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

build: client_send_recv

.PHONY: clean
clean:
	$(RM) client_send_recv.wasm
	$(RM) client_send_recv.wasm.c
	$(RM) client_send_recv.wasm.h
	$(RM) client_send_recv

client_send_recv.wasm: client_send_recv.c
	$(CC) $(CFLAGS) client_send_recv.c -o client_send_recv.wasm $(LDFLAGS)

client_send_recv.wasm.c: client_send_recv.wasm
	$(WASM2C_BIN_ROOT)/wasm2c -o client_send_recv.wasm.c client_send_recv.wasm

client_send_recv: client_send_recv.wasm.c
	gcc -shared -fPIC -O3 -o client_send_recv client_send_recv.wasm.c -I$(WASM2C_SRC_ROOT) $(WASM2C_SRC_ROOT)/wasm-rt-impl.c $(WASM2C_SRC_ROOT)/wasm-rt-os-unix.c $(WASM2C_SRC_ROOT)/wasm-rt-os-win.c $(WASM2C_SRC_ROOT)/wasm-rt-wasi.c ../../target/release/libveriwasi.so -I../../bindings

run: client_send_recv 
	RUST_LOG=trace $(WASM2C_BIN_ROOT)/wasm2c-runner ./client_send_recv --homedir=.
