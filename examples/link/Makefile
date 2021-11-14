RLBOX_ROOT = ../../rlbox_wasm2c_sandbox
WASM2C_SRC_ROOT = $(RLBOX_ROOT)/build/_deps/mod_wasm2c-src/wasm2c
WASM2C_BIN_ROOT = $(RLBOX_ROOT)/build/_deps/mod_wasm2c-src/bin
CC = $(RLBOX_ROOT)/build/_deps/wasiclang-src/bin/clang                        
CXX = $(RLBOX_ROOT)/build/_deps/wasiclang-src/bin/clang++                   
CFLAGS = --sysroot  $(RLBOX_ROOT)/build/_deps/wasiclang-src/share/wasi-sysroot/ 
LD = $(RLBOX_ROOT)/build/_deps/wasiclang-src/bin/wasm-ld
LDFLAGS = -Wl,--export-all -Wl,--growable-table

build: link

.PHONY: clean
clean:
	$(RM) link.wasm
	$(RM) link.wasm.c
	$(RM) link.wasm.h
	$(RM) link

link.wasm: link.c
	$(CC) $(CFLAGS) link.c -o link.wasm $(LDFLAGS)

link.wasm.c: link.wasm
	$(WASM2C_BIN_ROOT)/wasm2c -o link.wasm.c link.wasm

link: link.wasm.c
	gcc -shared -fPIC -O3 -o link link.wasm.c -I$(WASM2C_SRC_ROOT) $(WASM2C_SRC_ROOT)/wasm-rt-impl.c $(WASM2C_SRC_ROOT)/wasm-rt-os-unix.c $(WASM2C_SRC_ROOT)/wasm-rt-os-win.c $(WASM2C_SRC_ROOT)/wasm-rt-wasi.c ../../target/release/libveriwasi.so -I../../bindings

run: link 
	$(WASM2C_BIN_ROOT)/wasm2c-runner ./link
