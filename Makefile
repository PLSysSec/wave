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
	cargo build --release
	make bindings # this is dirty, fix this
	cd tools/wasm2c_sandbox_compiler && make
	cd tools/wasi-sdk && make 


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
# TODO: reenable overflow checks
verify:
	prusti-dev/target/release/cargo-prusti --features verify

verify-debug:
	PRUSTI_LOG=trace prusti-dev/target/debug/cargo-prusti --features verify

# Generate C/Cpp bindings for wave
# wasm2c expects to pass a void pointer instead of a VmCtx pointer 
# (which cbindgen generates), so I just use a sed command to replace it :)
bindings:
	mkdir -p bindings
	# Generates a temporary file with sed because Mac doesn't support -i flag
	cbindgen --config cbindgen.toml --crate wave --lang c --output bindings/wave_tmp.h
	sed 's/struct[[:space:]]VmCtx[[:space:]][*]const/void/g' bindings/wave_tmp.h > bindings/wave.h
	rm bindings/wave_tmp.h

wasm2c:
	cd rlbox_wasm2c_sandbox && cmake -S . -B ./build
	cd rlbox_wasm2c_sandbox && cmake --build ./build --target all

fuzz_trusted:
	mkdir -p fuzz-dir
	bash scan_for_trusted.sh
	RUST_LOG=quickcheck cargo test -- --nocapture
	rm -r fuzz-dir


