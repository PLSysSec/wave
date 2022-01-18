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
	cd rlbox_wasm2c_sandbox && cmake -S . -B ./build
	#cd rlbox_wasm2c_sandbox && cmake --build ./build --target all
	cd rlbox_wasm2c_sandbox/build/_deps/wasiclang-src && make # TODO: upstream sockets-wasiclang or amke rlbox/cmakelists do this
	cd rlbox_wasm2c_sandbox/build/_deps/wasiclang-src/src/wasi-libc && make # TODO: upstream sockets-wasi-libc or make rlbox/cmakelists do this
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
# TODO: reenable overflow checks
verify:
	prusti-dev/target/release/cargo-prusti --features verify

verify-debug:
	PRUSTI_DUMP_VIPER_PROGRAM=true PRUSTI_DUMP_DEBUG_INFO_DURING_FOLD=true prusti-dev/target/debug/cargo-prusti --features verify

# Generate C/Cpp bindings for veriwasi
# wasm2c expects to pass a void pointer instead of a VmCtx pointer 
# (which cbindgen generates), so I just use a sed command to replace it :)
# Need to do temporary file with sed because Mac doesn't support -i flag
bindings:
	mkdir -p bindings
	cbindgen --config cbindgen.toml --crate veriwasi --lang c --output bindings/veriwasi_tmp.h
	sed 's/struct[[:space:]]VmCtx[[:space:]][*]const/void/g' bindings/veriwasi_tmp.h > bindings/veriwasi.h
	rm bindings/veriwasi_tmp.h

wasm2c:
	cd rlbox_wasm2c_sandbox && cmake -S . -B ./build
	cd rlbox_wasm2c_sandbox && cmake --build ./build --target all

fuzz_trusted:
	bash scan_for_trusted.sh
	RUST_LOG=quickcheck cargo test -- --nocapture
