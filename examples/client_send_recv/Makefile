
TARGET=client_send_recv

include ../Makefile.inc

.PHONY: run
run: $(TARGET)
	$(WASM2C_BIN_ROOT)/wasm2c-runner ./$< --homedir=. --netlist=UDP:127.0.0.1:8080
