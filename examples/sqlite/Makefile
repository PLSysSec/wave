
SQLITE_ROOT    ?= ../sqlite
#WASI_SDK_ROOT  ?= /opt/wasi-sdk

RLBOX_ROOT      = ../../rlbox_wasm2c_sandbox
WASM2C_SRC_ROOT = $(RLBOX_ROOT)/build/_deps/mod_wasm2c-src/wasm2c
WASM2C_BIN_ROOT = $(RLBOX_ROOT)/build/_deps/mod_wasm2c-src/bin

SQLITE_BUILD = sqlite_build_wasi

WASI_SDK_ROOT = ../$(RLBOX_ROOT)/build/_deps/wasiclang-src/build/install/opt/wasi-sdk

# WASICLANG_ROOT = $(RLBOX_ROOT)/build/_deps/wasiclang-src/build/install/opt/wasi-sdk
# WASILIBC_ROOT =  $(RLBOX_ROOT)/build/_deps/wasiclang-src/src/wasi-libc

# CC = $(WASICLANG_ROOT)/bin/clang                        
# CXX = $(WASICLANG_ROOT)/bin/clang++                   
# CFLAGS = --sysroot  $(WASILIBC_ROOT)/sysroot/ 

# LD = $(WASICLANG_ROOT)/bin/wasm-ld
# LDFLAGS = -Wl,--export-all -Wl,--growable-table




build: speedtest1

.PHONY: clean
clean:
	$(RM) speedtest1.wasm
	$(RM) speedtest1.wasm.c
	$(RM) speedtest1.wasm.h
	$(RM) speedtest1
	$(RM) -r $(SQLITE_BUILD) 

speedtest1.wasm:
	mkdir -p $(SQLITE_BUILD) 
	cd $(SQLITE_BUILD) && ../compile_speedtest1.sh $(WASI_SDK_ROOT) $(SQLITE_ROOT) $(CURDIR)/lib
	cp $(SQLITE_BUILD)/speedtest1 ./speedtest1.wasm

speedtest1.wasm.c: speedtest1.wasm
	$(WASM2C_BIN_ROOT)/wasm2c -o speedtest1.wasm.c speedtest1.wasm

speedtest1_wasm2c: speedtest1.wasm.c
	gcc -shared -fPIC -O3 -o speedtest1_wasm2c speedtest1.wasm.c -I$(WASM2C_SRC_ROOT) $(WASM2C_SRC_ROOT)/wasm-rt-impl.c $(WASM2C_SRC_ROOT)/wasm-rt-os-unix.c $(WASM2C_SRC_ROOT)/wasm-rt-os-win.c $(WASM2C_SRC_ROOT)/wasm-rt-wasi.c ../../target/release/libwave.so -I../../bindings

speedtest1_wasmtime: speedtest1.wasm
	wasmtime/target/release/wasmtime compile speedtest1.wasm -o speedtest1_wasmtime

run_wasmtime: speedtest1.wasm
	wasmtime/target/release/wasmtime --dir=. speedtest1_wasmtime --allow-precompiled 2>/dev/null

run_wasm2c: speedtest1_wasm2c 
	$(WASM2C_BIN_ROOT)/wasm2c-runner ./speedtest1_wasm2c --homedir=.
