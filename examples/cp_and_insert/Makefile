RLBOX_ROOT = ../../rlbox_wasm2c_sandbox
WASM2C_SRC_ROOT = $(RLBOX_ROOT)/build/_deps/mod_wasm2c-src/wasm2c
WASM2C_BIN_ROOT = $(RLBOX_ROOT)/build/_deps/mod_wasm2c-src/bin
CC = $(RLBOX_ROOT)/build/_deps/wasiclang-src/bin/clang                        
CXX = $(RLBOX_ROOT)/build/_deps/wasiclang-src/bin/clang++                   
CFLAGS = --sysroot  $(RLBOX_ROOT)/build/_deps/wasiclang-src/share/wasi-sysroot/ 
LD = $(RLBOX_ROOT)/build/_deps/wasiclang-src/bin/wasm-ld
LDFLAGS = -Wl,--export-all -Wl,--growable-table

build: cp_and_insert

.PHONY: clean
clean:
	$(RM) cp_and_insert.wasm
	$(RM) cp_and_insert.wasm.c
	$(RM) cp_and_insert.wasm.h
	$(RM) cp_and_insert

cp_and_insert.wasm: cp_and_insert.c
	$(CC) $(CFLAGS) cp_and_insert.c -o cp_and_insert.wasm $(LDFLAGS)

cp_and_insert.wasm.c: cp_and_insert.wasm
	$(WASM2C_BIN_ROOT)/wasm2c -o cp_and_insert.wasm.c cp_and_insert.wasm

cp_and_insert: cp_and_insert.wasm.c
	gcc -shared -fPIC -O3 -o cp_and_insert cp_and_insert.wasm.c -I$(WASM2C_SRC_ROOT) $(WASM2C_SRC_ROOT)/wasm-rt-impl.c $(WASM2C_SRC_ROOT)/wasm-rt-os-unix.c $(WASM2C_SRC_ROOT)/wasm-rt-os-win.c $(WASM2C_SRC_ROOT)/wasm-rt-wasi.c ../../target/release/libveriwasi.so -I../../bindings

run: cp_and_insert 
	$(WASM2C_BIN_ROOT)/wasm2c-runner ./cp_and_insert
