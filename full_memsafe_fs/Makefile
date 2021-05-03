.PHONY: check_C check_CPP check_Rust

Z3 := /root/smack-deps/z3/bin
BOOGIE := /root/smack-deps/boogie
CORRAL := /root/smack-deps/corral
COMPLEX_FS_ENTRIES := safe_open safe_close safe_read safe_write safe_fstat safe_lseek safe_dup2 safe_unlink safe_symlink safe_symlink safe_readlink safe_getcwd safe_chdir safe_mkdir safe_rmdir
CPP_FLAGS := --integer-encoding=bit-vector --pointer-encoding=bit-vector
# Entry points doesnt work unless we use boogie 
VERIFIER := boogie


export PATH := $(Z3):$(PATH)
export PATH := $(BOOGIE):$(PATH)
export PATH := $(CORRAL):$(PATH)


check_C:
	smack C/wrappers.c --verifier $(VERIFIER) $(CPP_FLAGS) --entry-points $(COMPLEX_FS_ENTRIES) 
