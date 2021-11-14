RLBOX_ROOT = ../../rlbox_wasm2c_sandbox
WASM2C_SRC_ROOT = $(RLBOX_ROOT)/build/_deps/mod_wasm2c-src/wasm2c
WASM2C_BIN_ROOT = $(RLBOX_ROOT)/build/_deps/mod_wasm2c-src/bin
CC = $(RLBOX_ROOT)/build/_deps/wasiclang-src/bin/clang                        
CXX = $(RLBOX_ROOT)/build/_deps/wasiclang-src/bin/clang++                   
CFLAGS = --sysroot  $(RLBOX_ROOT)/build/_deps/wasiclang-src/share/wasi-sysroot/ 
LD = $(RLBOX_ROOT)/build/_deps/wasiclang-src/bin/wasm-ld
LDFLAGS = -Wl,--export-all -Wl,--growable-table

build: rename

.PHONY: clean
clean:
	$(RM) rename.wasm
	$(RM) rename.wasm.c
	$(RM) rename.wasm.h
	$(RM) rename

rename.wasm: rename.c
	$(CC) $(CFLAGS) rename.c -o rename.wasm $(LDFLAGS)

rename.wasm.c: rename.wasm
	$(WASM2C_BIN_ROOT)/wasm2c -o rename.wasm.c rename.wasm

rename: rename.wasm.c
	gcc -shared -fPIC -O3 -o rename rename.wasm.c -I$(WASM2C_SRC_ROOT) $(WASM2C_SRC_ROOT)/wasm-rt-impl.c $(WASM2C_SRC_ROOT)/wasm-rt-os-unix.c $(WASM2C_SRC_ROOT)/wasm-rt-os-win.c $(WASM2C_SRC_ROOT)/wasm-rt-wasi.c ../../target/release/libveriwasi.so -I../../bindings

run: rename 
	$(WASM2C_BIN_ROOT)/wasm2c-runner ./rename
