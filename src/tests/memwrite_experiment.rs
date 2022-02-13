use crate::os;
use crate::runtime::fresh_ctx;
use crate::types::{SboxPtr, VmCtx};
use std::time::{Duration, Instant};
use std::ptr::{copy, copy_nonoverlapping};
use quickcheck::{QuickCheck, TestResult};
use quickcheck_macros;

/*
The goal of these experiments are to compare the correctness and performance of 7 different means 
by which we could write data to mem.
I will use this information to select an implementation for how wave will read/write to memory.
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
1 small write
repeated small writes
big writes
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

fn write_method_1_u32(ctx: &mut VmCtx, v: u32, offset:usize){
    let bytes: [u8; 4] = v.to_le_bytes();
    self.mem[offset] = bytes[0];
    self.mem[offset + 1] = bytes[1];
    self.mem[offset + 2] = bytes[2];
    self.mem[offset + 3] = bytes[3];
}

fn write_method_1_u64(ctx: &mut VmCtx, v: u64, offset:usize){
    let bytes: [u8; 8] = v.to_le_bytes();
    self.mem[offset] = bytes[0];
    self.mem[offset + 1] = bytes[1];
    self.mem[offset + 2] = bytes[2];
    self.mem[offset + 3] = bytes[3];
    self.mem[offset + 4] = bytes[4];
    self.mem[offset + 5] = bytes[5];
    self.mem[offset + 6] = bytes[6];
    self.mem[offset + 7] = bytes[7];
}

// Method 2: vector write
fn write_method_2(ctx: &mut VmCtx, src: &[u8], offset:usize){
    (ctx.mem.get(offset)).write(src);
    ctx.mem.flush();
}

// Method 4: direct pointer manipulation
fn write_method_4_u32(ctx: &mut VmCtx, v: u32, offset:usize){
    unsafe{ ptr::write(ctx.mem.as_mut_ptr().offset(offset), v); }
}

fn write_method_4_u64(ctx: &mut VmCtx, v: u64, offset:usize){
    unsafe{ ptr::write(ctx.mem.as_mut_ptr().offset(offset), v); }
}

// Method 5: vector::splice
fn write_method_5(ctx: &mut VmCtx, src: &[u8], offset:usize){
    ctx.mem.splice(offset..offset+src.len(), src);

}

// Method 6: copy_non_overlapping (memcpy)
fn write_method_6(ctx: &mut VmCtx, src: &[u8], offset:usize){
    unsafe {
        copy_nonoverlapping(
            src.as_ptr(),
            self.mem.as_mut_ptr().offset(offset as isize),
            src.len(),
        )
    };
}

// Method 7: copy (memmove)
fn write_method_7(ctx: &mut VmCtx, src: &[u8], offset:usize){
    unsafe {
        copy(
            src.as_ptr(),
            self.mem.as_mut_ptr().offset(offset as isize),
            src.len(),
        )
    };
}

// Compute lag on using Instant::now
fn trial_0(){
    //let ctx = fresh_ctx();
    let total_time = 0;
    // Run 1000 trials
    for idx in 0..1000{
        let now = Instant::now();
        let time = now.elapsed().as_nanos();
        total_time += time;
    }
    println!("Average delay: {:?}", total_time / 1000);
}

// Testing 32 bit writes using safe method vs unsafe method
fn trial_1_safe(){
    let ctx = fresh_ctx();
    let total_time = 0;
    // Run 1000 trials
    for idx in 0..1000{
        let addr = idx * 8;
        let now = Instant::now();
        write_method_1_u32(ctx, 1337, addr);
        let time = now.elapsed().as_nanos();
        total_time += time;
    }
    println!("Average safe 32-bit write delay: {:?}", total_time / 1000);
}

fn trial_1_unsafe(){
    let ctx = fresh_ctx();
    let total_time = 0;
    // Run 1000 trials
    for idx in 0..1000{
        let addr = idx * 8;
        let now = Instant::now();
        write_method_4_u32(ctx, 1337, addr);
        let time = now.elapsed().as_nanos();
        total_time += time;
    }
    println!("Average unsafe 32-bit write delay: {:?}", total_time / 1000);
}

