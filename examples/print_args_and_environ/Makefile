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

build: print_args_and_environ

.PHONY: clean
clean:
	$(RM) print_args_and_environ.wasm
	$(RM) print_args_and_environ.wasm.c
	$(RM) print_args_and_environ.wasm.h
	$(RM) print_args_and_environ

print_args_and_environ.wasm: print_args_and_environ.c
	$(CC) $(CFLAGS) print_args_and_environ.c -o print_args_and_environ.wasm $(LDFLAGS)

print_args_and_environ.wasm.c: print_args_and_environ.wasm
	$(WASM2C_BIN_ROOT)/wasm2c -o print_args_and_environ.wasm.c print_args_and_environ.wasm

print_args_and_environ: print_args_and_environ.wasm.c
	gcc -shared -fPIC -O3 -o print_args_and_environ print_args_and_environ.wasm.c -I$(WASM2C_SRC_ROOT) $(WASM2C_SRC_ROOT)/wasm-rt-impl.c $(WASM2C_SRC_ROOT)/wasm-rt-os-unix.c $(WASM2C_SRC_ROOT)/wasm-rt-os-win.c $(WASM2C_SRC_ROOT)/wasm-rt-wasi.c ../../target/release/libwave.so -I../../bindings

run: print_args_and_environ 
	RUST_LOG=trace $(WASM2C_BIN_ROOT)/wasm2c-runner ./print_args_and_environ --args="a b c" --env="CC=clang CXX=clang++" --homedir=.
