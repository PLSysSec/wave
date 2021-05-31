.PHONY: check_C check_CPP check_Rust

Z3 := /root/smack-deps/z3/bin
BOOGIE := /root/smack-deps/boogie
CORRAL := /root/smack-deps/corral
# COMPLEX_FS_ENTRIES := safe_open safe_close 
CPP_FLAGS := --integer-encoding=bit-vector --pointer-encoding=bit-vector
# Entry points doesnt work unless we use boogie 
VERIFIER := boogie


export PATH := $(Z3):$(PATH)
export PATH := $(BOOGIE):$(PATH)
export PATH := $(CORRAL):$(PATH)


check_C:
	smack C/harness.c --verifier $(VERIFIER) $(CPP_FLAGS) --verbose

check_mem_safety:
	smack C/harness.c --verifier $(VERIFIER) --check memory-safety --verbose

check_C_dump_file:
	smack C/harness.c --verifier $(VERIFIER) --modular -t -bc out.bc -ll out.ll -bpl out.bpl

check_mem_safety_dump_file:
	smack C/wrappers.c --verifier $(VERIFIER) $(CPP_FLAGS) --check memory-safety -t -bc out.bc -ll out.ll -bpl out.bpl
