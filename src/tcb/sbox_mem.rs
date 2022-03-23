use crate::do_effect;
#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::types::*;
use crate::{effect, four_effects, no_effect, one_effect, three_effects, two_effects};
use prusti_contracts::*;
use std::ptr::{copy, copy_nonoverlapping};
use wave_macros::{external_calls, external_methods, with_ghost_var};

// use libc::{mmap, mprotect, munmap};
use libc::{PROT_NONE, PROT_READ, PROT_WRITE};
use libc::{MAP_ANONYMOUS, MAP_PRIVATE, MAP_FAILED};
use libc::c_void;
use std::ptr;
use prusti_contracts::*;
use crate::tcb::misc::bitwise_or;

// 1 << 32 = 4GB
const FOUR_GB: usize = 1 << 32;
// 1 << 33 = 8GB
const EIGHT_GB: usize = 1 << 33;

// Uninterpreted predicate meant to accompany slice_mem_mut
// result is equal to the offset into memory that slice came from, i.e.
// slice.start - mem.start
// if the slice did not come from memory, then the return value will be unconstrained (i.e., any pointer)
#[pure]
#[trusted]
pub fn as_sbox_ptr(slice: &[u8]) -> usize {
    unimplemented!()
}

#[pure]
#[trusted]
pub fn raw_ptr(memptr: &[u8]) -> usize {
    unimplemented!()
}

impl VmCtx {
    /// Function for memcpy from sandbox to host
    /// Overwrites contents of vec
    #[with_ghost_var(trace: &mut Trace)]
    #[external_calls(copy_nonoverlapping)]
    #[external_methods(set_len)]
    #[requires(dst.capacity() >= (n as usize) )]
    #[requires(self.fits_in_lin_mem(src, n, trace))]
    #[requires(ctx_safe(self))]
    #[requires(trace_safe(trace, self))]
    // #[ensures(ctx_safe(self))]
    // #[ensures(trace_safe(trace, self))]
    #[ensures(dst.len() == (n as usize) )]
    #[ensures(one_effect!(old(trace), trace, effect!(ReadN, addr, count) if addr == src as usize && count == n as usize))]
    #[trusted]
    pub fn memcpy_from_sandbox(&self, dst: &mut Vec<u8>, src: SboxPtr, n: u32) {
        unsafe {
            copy_nonoverlapping(
                self.mem.as_ptr().offset(src as isize),
                dst.as_mut_ptr(),
                n as usize,
            );
            dst.set_len(n as usize);
        };
        // do_effect!(effect!(ReadN, src, n));
    }

    /// Function for memcpy from sandbox to host
    #[with_ghost_var(trace: &mut Trace)]
    #[external_calls(copy_nonoverlapping)]
    #[requires(src.len() >= (n as usize) )]
    #[requires(self.fits_in_lin_mem(dst, n, trace))]
    #[requires(ctx_safe(self))]
    #[requires(trace_safe(trace, self))]

    #[ensures(ctx_safe(self))]
    #[ensures(trace_safe(trace, self))]
    #[ensures(one_effect!(old(trace), trace, effect!(WriteN, addr, count) if addr == dst as usize && count == n as usize))]
    #[trusted]
    pub fn memcpy_to_sandbox(&mut self, dst: SboxPtr, src: &Vec<u8>, n: u32) {
        unsafe {
            copy_nonoverlapping(
                src.as_ptr(),
                self.mem.as_mut_ptr().offset(dst as isize),
                n as usize,
            )
        };
    }

    // Currently trusted because it causes a fold-unfold error
    #[with_ghost_var(trace: &mut Trace)]
    #[requires(self.fits_in_lin_mem(ptr, len, trace))]
    #[requires(trace_safe(trace, self))]

    // #[ensures(trace_safe(trace, old(self).memlen))]
    #[ensures(result.len() == (len as usize))]
    #[ensures(no_effect!(old(trace), trace))]
    #[ensures(as_sbox_ptr(result) == old(ptr as usize))]
    //#[after_expiry(old(self.netlist) == self.netlist)]
    #[after_expiry(ctx_safe(self) && old(self.netlist) == self.netlist && old(self.homedir_host_fd) == self.homedir_host_fd)]
    #[trusted]
    pub fn slice_mem_mut(&mut self, ptr: SboxPtr, len: u32) -> &mut [u8] {
        let start = ptr as usize;
        let end = ptr as usize + len as usize;
        &mut self.mem[start..end]
    }

    // This needs to be trusted only because I can't seem to convice Prusti
    // that these safe memory writes do not update the linmem ptr
    #[with_ghost_var(trace: &mut Trace)]
    #[requires(self.fits_in_lin_mem_usize(offset, 1, trace))]
    #[requires(ctx_safe(self))]
    #[requires(trace_safe(trace, self))]
    #[ensures(ctx_safe(self))]
    #[ensures(trace_safe(trace, self))]
    #[trusted]
    pub fn write_u8(&mut self, offset: usize, v: u8) {
        self.mem[offset] = v;
    }

}

// Linear memory allocation stuff

#[trusted]
// #[requires(n <= dest.len())]
// #[ensures(forall(|i: usize|  (i < n) ==> dest[i] == c))]
pub fn memset(dest: usize, c: u8, n: usize){
    unsafe{
    libc::memset(dest as *mut c_void, c as i32, n);
    }
}

// // #[requires(addr == 0)]
// // #[requires()]
#[trusted]
//#[ensures((result != MAP_FAILED) ==> mapping(result) == Some(Mmapping(len,prot)) ]
pub fn mmap(
    addr: usize,
    len: usize,
    prot: i32,
    flags: i32,
    fd: i32, // fd to back to
    offset: i64 // offset into file to back to
) -> usize {
    unsafe{
        libc::mmap(addr as *mut c_void, len, prot, flags, fd, offset) as usize
    }
}

// #[ensures((result == 0) ==> mapping(addr) == None)]
#[trusted]
pub fn munmap(addr: usize, len: usize) -> i32 {
    unsafe{
        libc::munmap(addr as *mut libc::c_void, len)
    }
}

// #[ensures((result == 0) ==> )]
#[trusted]
pub fn mprotect(addr: usize, len: usize, prot: i32) -> i32 {
    unsafe{
        libc::mprotect(addr as *mut c_void, len, prot)
    }
}


// bodyless viper function
#[pure]
#[trusted]
pub fn valid_linmem(memptr: usize) -> bool {
    unimplemented!()
} 

#[trusted]
#[ensures(valid_linmem(result))]
fn wave_alloc_linmem() -> usize {
    let linmem_ptr = mmap(
        0,                           // let the kernel place the region anywhere
        EIGHT_GB,                    // Linmem + guard page = 8GB
        bitwise_or(PROT_READ, PROT_WRITE),      // its read/write
        bitwise_or(MAP_PRIVATE, MAP_ANONYMOUS), // should not be shared or backed-up to a file
        -1,                          // no file descrptor since we aren't backing to a file
        0,                           // this arg doesn't matter since we aren't backing to a file
    ); 
    // let x: [u8; 4] = [0,1,2,3];
    // assert!( cool_ptr((&x).as_ptr()) );
    mprotect(linmem_ptr + FOUR_GB, FOUR_GB, PROT_NONE); // Make second 4GB of linear memory a guard page
    memset(linmem_ptr, 0, FOUR_GB); // memzero
    linmem_ptr
    
}

