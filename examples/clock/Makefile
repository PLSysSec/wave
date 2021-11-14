RLBOX_ROOT = ../../rlbox_wasm2c_sandbox
WASM2C_SRC_ROOT = $(RLBOX_ROOT)/build/_deps/mod_wasm2c-src/wasm2c
WASM2C_BIN_ROOT = $(RLBOX_ROOT)/build/_deps/mod_wasm2c-src/bin
CC = $(RLBOX_ROOT)/build/_deps/wasiclang-src/bin/clang                        
CXX = $(RLBOX_ROOT)/build/_deps/wasiclang-src/bin/clang++                   
CFLAGS = --sysroot  $(RLBOX_ROOT)/build/_deps/wasiclang-src/share/wasi-sysroot/ 
LD = $(RLBOX_ROOT)/build/_deps/wasiclang-src/bin/wasm-ld
LDFLAGS = -Wl,--export-all -Wl,--growable-table

build: clock

.PHONY: clean
clean:
	$(RM) clock.wasm
	$(RM) clock.wasm.c
	$(RM) clock.wasm.h
	$(RM) clock

clock.wasm: clock.c
	$(CC) $(CFLAGS) clock.c -o clock.wasm $(LDFLAGS)

clock.wasm.c: clock.wasm
	$(WASM2C_BIN_ROOT)/wasm2c -o clock.wasm.c clock.wasm

clock: clock.wasm.c
	gcc -shared -fPIC -O0 -o clock clock.wasm.c -I$(WASM2C_SRC_ROOT) $(WASM2C_SRC_ROOT)/wasm-rt-impl.c $(WASM2C_SRC_ROOT)/wasm-rt-os-unix.c $(WASM2C_SRC_ROOT)/wasm-rt-os-win.c $(WASM2C_SRC_ROOT)/wasm-rt-wasi.c ../../target/release/libveriwasi.so -I../../bindings

run: clock 
	$(WASM2C_BIN_ROOT)/wasm2c-runner ./clock
