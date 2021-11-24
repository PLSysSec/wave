use crate::os;
use crate::runtime::fresh_ctx;
use crate::types::{SboxPtr, VmCtx};
use std::time::{Duration, Instant};
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

/* 
let now = Instant::now();
now.elapsed().as_nanos()
*/

// Method 1: repeated writes
fn write_method_1(ctx: &mut VmCtx, src: &[u8], offset:usize){
    for idx in 0..src.len() {
        ctx[offset + idx] = src[idx];
    }
}

// Method 2: vector write
fn write_method_2(ctx: &mut VmCtx, src: &[u8], offset:usize){
    (ctx.mem.get(offset)).write(src);
    ctx.mem.flush();
}


// Method 5: vector::splice
fn write_method_5(ctx: &mut VmCtx, src: &[u8], offset:usize){
    ctx.mem.splice(offset..offset+src.len(), src);

}

// // Method 6: copy
// fn write_method_5(ctx: &mut VmCtx, src: &[u8], offset:usize){
//     ctx.mem.splice(offset..offset+src.len(), src);

// }




