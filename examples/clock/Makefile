
TARGET=clock

include ../Makefile.inc

%.wasm: %.c
	$(CC) $(CFLAGS) -D_WASI_EMULATED_PROCESS_CLOCKS -lwasi-emulated-process-clocks $< -o $@ $(LDFLAGS)
