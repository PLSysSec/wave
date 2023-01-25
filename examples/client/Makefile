
TARGET=client

include ../Makefile.inc

.PHONY: run
run: $(TARGET)
	$(WASM2C_BIN_ROOT)/wasm2c-runner ./$< --homedir=. --netlist=TCP:127.0.0.1:36895
