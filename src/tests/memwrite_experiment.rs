use crate::os;
use crate::runtime::fresh_ctx;
use crate::types::{SboxPtr, VmCtx};
use quickcheck::{QuickCheck, TestResult};
use quickcheck_macros;

/*
The goal of these experiments are to compare the correctness and performance of 7 different means 
by which we could write data to mem.
I will use this information to select an implementation for how veriwasi will read/write to memory.
The methods are:
    1. repeated 1-byte writes
    2. vector writer
    3. vector writer-vectored
    4. direct pointer manipulation
    5. vector::splice
    6. copy_nonoverlapping (memcpy)
    7. copy (memmove)


    All of these will be tested both with small and large writes, specifically:
    1. 4 byte write
    2. 8 byte write
    3. 8-8 byte write (random)
    4. 64 byte write (any way)
    5. 4096 byte write (any way)
*/









