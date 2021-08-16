RLBOX_ROOT = ../../rlbox_wasm2c_sandbox
WASM2C_SRC_ROOT = $(RLBOX_ROOT)/build/_deps/mod_wasm2c-src/wasm2c
WASM2C_BIN_ROOT = $(RLBOX_ROOT)/build/_deps/mod_wasm2c-src/bin
CC = $(RLBOX_ROOT)/build/_deps/wasiclang-src/bin/clang                        
CXX = $(RLBOX_ROOT)/build/_deps/wasiclang-src/bin/clang++                   
CFLAGS = --sysroot  $(RLBOX_ROOT)/build/_deps/wasiclang-src/share/wasi-sysroot/ 
LD = $(RLBOX_ROOT)/build/_deps/wasiclang-src/bin/wasm-ld
LDFLAGS = -Wl,--export-all -Wl,--growable-table

build: cat

.PHONY: clean
clean:
	$(RM) cat.wasm
	$(RM) cat.wasm.c
	$(RM) cat.wasm.h
	$(RM) cat

cat.wasm: cat.c
	$(CC) $(CFLAGS) cat.c -o cat.wasm $(LDFLAGS)

cat.wasm.c: cat.wasm
	$(WASM2C_BIN_ROOT)/wasm2c -o cat.wasm.c cat.wasm

cat: cat.wasm.c
	gcc -shared -fPIC -O3 -o cat cat.wasm.c -I$(WASM2C_SRC_ROOT) $(WASM2C_SRC_ROOT)/wasm-rt-impl.c $(WASM2C_SRC_ROOT)/wasm-rt-os-unix.c $(WASM2C_SRC_ROOT)/wasm-rt-os-win.c $(WASM2C_SRC_ROOT)/wasm-rt-wasi.c ../../target/release/libveriwasi.so -I../../bindings

run: cat 
	$(WASM2C_BIN_ROOT)/wasm2c-runner ./cat
