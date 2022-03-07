.PHONY: bootstrap build verify prusti bindings wasm2c

# Prusti setup and build instructions: 
# (this makefile should handle anything, but you can look at these if you run into problems)
# https://viperproject.github.io/prusti-dev/dev-guide/development/setup.html
# https://viperproject.github.io/prusti-dev/dev-guide/development/build.html

# Used by Prusti
JAVA_HOME ?= /usr/lib/jvm/ 

SPEC_PATH := ./wave-specbenchmark

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
	PRUSTI_LOG=trace prusti-dev/target/debug/cargo-prusti --features verify

# Generate C/Cpp bindings for wave
# wasm2c expects to pass a void pointer instead of a VmCtx pointer 
# (which cbindgen generates), so I just use a sed command to replace it :)
bindings:
	mkdir -p bindings
	cbindgen --config cbindgen.toml --crate wave --lang c --output bindings/wave.h
	sed -i 's/struct[[:space:]]VmCtx[[:space:]][*]const/void/g' bindings/wave.h

wasm2c:
	cd rlbox_wasm2c_sandbox && cmake -S . -B ./build
	cd rlbox_wasm2c_sandbox && cmake --build ./build --target all

fuzz_trusted:
	bash scan_for_trusted.sh
	RUST_LOG=quickcheck cargo test -- --nocapture



NATIVE_BUILD=linux32-i386-clang linux32-i386-clangzerocost
NACL_BUILDS=linux32-i386-nacl
SPEC_BUILDS=$(NACL_BUILDS) $(NATIVE_BUILDS)


$(SPEC_PATH): # libnsl/build/lib/libnsl.so.1
	git clone git@github.com:PLSysSec/wave-specbenchmark.git
	cd $(SPEC_PATH) && SPEC_INSTALL_NOCHECK=1 SPEC_FORCE_INSTALL=1 sh install.sh -f

# TODO: use parallel compilation? remove unnecessary options?
build_spec: $(SPEC_PATH)
	cd $(SPEC_PATH) && source ./shrc && \
	cd config && \
	runspec --config=linux64-amd64-clang.cfg --action=build --define cores=1 --iterations=1 --noreportable --size=ref wasm_compatible
	runspec --config=wasmtime.cfg --action=build --define cores=1 --iterations=1 --noreportable --size=ref wasm_compatible
	runspec --config=wasm2c_wave.cfg --action=build --define cores=1 --iterations=1 --noreportable --size=ref wasm_compatible	

# echo "Cleaning dirs" && \
# for spec_build in $(SPEC_BUILDS); do \
# 	runspec --config=$$spec_build.cfg --action=clobber all_c_cpp 2&>1 > /dev/null; \
# done && \
#  2>&1 | grep -i "building"

# TODO: change size of spec runs back to size=ref
# TODO: finalize
run_spec:
	cd $(SPEC_PATH) && source ./shrc && cd config && \
	runspec --config=wasm2c_wave.cfg --wasm2c_wave --action=run --define cores=1 --iterations=1 --noreportable --size=test wasm_compatible
	#for spec_build in $(NATIVE_BUILDS); do \
	#	runspec --config=$$spec_build.cfg --action=run --define cores=1 --iterations=1 --noreportable --size=ref all_c_cpp; \
	#done && \
	#for spec_build in $(NACL_BUILDS); do \
	#	runspec --config=$$spec_build.cfg --action=run --define cores=1 --iterations=1 --noreportable --size=ref --nacl all_c_cpp; \
	#done
	#python3 spec_stats.py -i $(SPEC_PATH)/result --filter  \
	#	"$(SPEC_PATH)/result/spec_results=Stock:Stock,NaCl:NaCl,SegmentZero:SegmentZero" -n 3 --usePercent
	#mv $(SPEC_PATH)/result/ benchmarks/spec_$(shell date --iso=seconds)

