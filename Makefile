.PHONY: bootstrap build verify prusti bindings wasm2c

# Prusti setup and build instructions: 
# (this makefile should handle anything, but you can look at these if you run into problems)
# https://viperproject.github.io/prusti-dev/dev-guide/development/setup.html
# https://viperproject.github.io/prusti-dev/dev-guide/development/build.html

# Used by Prusti
JAVA_HOME ?= /usr/lib/jvm/ 

bootstrap:
	git submodule update --init --recursive
	cd prusti-dev && ./x.py setup
	cd prusti-dev && JAVA_HOME=$(JAVA_HOME) ./x.py build --release
	cd rlbox_wasm2c_sandbox && cmake -S . -B ./build
	cd rlbox_wasm2c_sandbox && cmake --build ./build --target all


prusti:
	cd prusti-dev && JAVA_HOME=$(JAVA_HOME) ./x.py build --release

build:
	cargo build --release

build_hello_example:
	cd examples/hello && make clean
	cd examples/hello && make build

run_hello_example:
	cd examples/hello && make run

# If this command is giving you trouble, try deleting the ./target directory and retrying
# overflow checks temporarily disabled
verify:
	PRUSTI_CHECK_OVERFLOWS=false prusti-dev/target/release/cargo-prusti --features verify

# Generate C/Cpp bindings for veriwasi
bindings:
	mkdir -p bindings
	cbindgen --config cbindgen.toml --crate veriwasi --lang c --output bindings/veriwasi.h

wasm2c:
	cd rlbox_wasm2c_sandbox && cmake -S . -B ./build
	cd rlbox_wasm2c_sandbox && cmake --build ./build --target all


