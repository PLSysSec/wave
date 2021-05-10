.PHONY: check_C check_CPP check_Rust

Z3 := /root/smack-deps/z3/bin
BOOGIE := /root/smack-deps/boogie
CORRAL := /root/smack-deps/corral
SIMPLE_FS_ENTRIES := safe_open safe_close safe_read safe_write
CPP_FLAGS := --integer-encoding=bit-vector --pointer-encoding=bit-vector
VERIFIER := boogie


export PATH := $(Z3):$(PATH)
export PATH := $(BOOGIE):$(PATH)
export PATH := $(CORRAL):$(PATH)


check_C:
	smack C/wrappers.c --verifier $(VERIFIER) $(CPP_FLAGS) --entry-points $(SIMPLE_FS_ENTRIES) 

check_C_model:
	smack C/wrappers_for_model.c --verifier $(VERIFIER) $(CPP_FLAGS) --entry-points $(SIMPLE_FS_ENTRIES) 
	smack C/wrappers_for_model.c --verifier $(VERIFIER) $(CPP_FLAGS) --entry-points $(SIMPLE_FS_ENTRIES) --check memory-safety

check_Cpp:
	smack Cpp/wrappers.cpp --verifier $(VERIFIER) $(CPP_FLAGS) --entry-points $(SIMPLE_FS_ENTRIES) 