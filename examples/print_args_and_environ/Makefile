
TARGET=print_args_and_environ

include ../Makefile.inc

run: print_args_and_environ 
	RUST_LOG=trace $(WASM2C_BIN_ROOT)/wasm2c-runner ./print_args_and_environ --args="a b c" --env="CC=clang CXX=clang++" --homedir=.
