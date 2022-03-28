RLBOX_ROOT = ../../rlbox_wasm2c_sandbox
WASM2C_SRC_ROOT = $(RLBOX_ROOT)/build/_deps/mod_wasm2c-src/wasm2c
WASM2C_BIN_ROOT = $(RLBOX_ROOT)/build/_deps/mod_wasm2c-src/bin

WASICLANG_ROOT = $(RLBOX_ROOT)/build/_deps/wasiclang-src/build/install/opt/wasi-sdk
WASILIBC_ROOT =  $(RLBOX_ROOT)/build/_deps/wasiclang-src/build/install/opt/wasi-sdk/share
CFLAGS = --sysroot  $(WASILIBC_ROOT)/wasi-sysroot/

# Added htis cause build directories were different on my mac
# This way both mac and linux should "Just work"
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S), Darwin)
DYLIB_EXT = dylib
endif
ifeq ($(UNAME_S), Linux)
DYLIB_EXT = so
endif

CC = $(WASICLANG_ROOT)/bin/clang
CXX = $(WASICLANG_ROOT)/bin/clang++
LD = $(WASICLANG_ROOT)/bin/wasm-ld
LDFLAGS = -Wl,--export-all -Wl,--growable-table

$(call check_defined, TARGET)

.PHONY: build
build: $(TARGET)

.PHONY: clean
clean:
	$(RM) $(TARGET).wasm
	$(RM) $(TARGET).wasm.c
	$(RM) $(TARGET).wasm.h
	$(RM) $(TARGET) 

%.wasm: %.c
	$(CC) $(CFLAGS) $< -o $@ $(LDFLAGS)

%.wasm.c: %.wasm
	$(WASM2C_BIN_ROOT)/wasm2c -o $@ $<

$(TARGET): $(TARGET).wasm.c
	gcc -shared -fPIC -O3 -o $@ $< -I$(WASM2C_SRC_ROOT) $(WASM2C_SRC_ROOT)/wasm-rt-impl.c $(WASM2C_SRC_ROOT)/wasm-rt-os-unix.c $(WASM2C_SRC_ROOT)/wasm-rt-os-win.c $(WASM2C_SRC_ROOT)/wasm-rt-wasi.c ../../target/release/libwave.$(DYLIB_EXT) -I../../bindings

.PHONY: run
run: $(TARGET)
	$(WASM2C_BIN_ROOT)/wasm2c-runner ./$< --homedir=.
