use crate::do_effect;
#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::types::*;
use crate::{effect, four_effects, no_effect, one_effect, three_effects, two_effects};
use extra_args::{external_calls, external_methods, with_ghost_var};
use prusti_contracts::*;
use std::ptr::{copy, copy_nonoverlapping};

// Uninterpreted predicate meant to accompany slice_mem_mut
// result is equal to the offset into memory that slice came from, i.e.
// slice.start - mem.start
// if the slice did not come from memory, then the return value will be unconstrained (i.e., any pointer)
#[pure]
#[trusted]
pub fn as_sbox_ptr(slice: &[u8]) -> usize {
    unimplemented!()
}

impl VmCtx {
    /// read u16 from wasm linear memory
    // Not thrilled about this implementation, but it works
    #[with_ghost_var(trace: &mut Trace)]
    #[external_calls(from_le_bytes)]
    #[requires(self.fits_in_lin_mem_usize(start, 2, trace))]
    #[requires(trace_safe(trace, self.memlen) && ctx_safe(self))]
    #[ensures(trace_safe(trace, self.memlen) && ctx_safe(self))]
    #[ensures(one_effect!(old(trace), trace, effect!(ReadN, addr, 2) if addr == start as usize))]
    #[trusted]
    pub fn read_u16(&self, start: usize) -> u16 {
        let bytes: [u8; 2] = [self.mem[start], self.mem[start + 1]];
        u16::from_le_bytes(bytes)
    }

    /// read u32 from wasm linear memory
    // Not thrilled about this implementation, but it works
    #[with_ghost_var(trace: &mut Trace)]
    #[external_calls(from_le_bytes)]
    #[requires(self.fits_in_lin_mem_usize(start, 4, trace))]
    #[requires(trace_safe(trace, self.memlen) && ctx_safe(self))]
    #[ensures(trace_safe(trace, self.memlen) && ctx_safe(self))]
    #[ensures(one_effect!(old(trace), trace, effect!(ReadN, addr, 4) if addr == start as usize))]
    #[trusted]
    pub fn read_u32(&self, start: usize) -> u32 {
        let bytes: [u8; 4] = [
            self.mem[start],
            self.mem[start + 1],
            self.mem[start + 2],
            self.mem[start + 3],
        ];
        u32::from_le_bytes(bytes)
    }

    /// read u64 from wasm linear memory
    // Not thrilled about this implementation, but it works
    // TODO: need to test different implementatiosn for this function
    #[with_ghost_var(trace: &mut Trace)]
    #[external_calls(from_le_bytes)]
    #[requires(self.fits_in_lin_mem_usize(start, 8, trace))]
    #[requires(trace_safe(trace, self.memlen) && ctx_safe(self))]
    #[ensures(trace_safe(trace, self.memlen) && ctx_safe(self))]
    #[ensures(one_effect!(old(trace), trace, effect!(ReadN, addr, 8) if addr == start as usize))]
    #[trusted]
    pub fn read_u64(&self, start: usize) -> u64 {
        let bytes: [u8; 8] = [
            self.mem[start],
            self.mem[start + 1],
            self.mem[start + 2],
            self.mem[start + 3],
            self.mem[start + 4],
            self.mem[start + 5],
            self.mem[start + 6],
            self.mem[start + 7],
        ];
        u64::from_le_bytes(bytes)
    }

    /// write u16 to wasm linear memory
    // Not thrilled about this implementation, but it works
    #[with_ghost_var(trace: &mut Trace)]
    #[external_methods(to_le_bytes)]
    #[requires(self.fits_in_lin_mem_usize(start, 2, trace))]
    #[requires(trace_safe(trace, self.memlen) && ctx_safe(self))]
    #[ensures(trace_safe(trace, self.memlen) && ctx_safe(self))]
    #[ensures(one_effect!(old(trace), trace, effect!(WriteN, addr, 2) if addr == start as usize))]
    #[trusted]
    pub fn write_u16(&mut self, start: usize, v: u16) {
        let bytes: [u8; 2] = v.to_le_bytes();
        self.mem[start] = bytes[0];
        self.mem[start + 1] = bytes[1];
    }

    /// write u32 to wasm linear memory
    // Not thrilled about this implementation, but it works
    #[with_ghost_var(trace: &mut Trace)]
    #[external_methods(to_le_bytes)]
    #[requires(self.fits_in_lin_mem_usize(start, 4, trace))]
    #[requires(trace_safe(trace, self.memlen) && ctx_safe(self))]
    #[ensures(trace_safe(trace, self.memlen) && ctx_safe(self))]
    #[ensures(one_effect!(old(trace), trace, effect!(WriteN, addr, 4) if addr == start as usize))]
    #[trusted]
    pub fn write_u32(&mut self, start: usize, v: u32) {
        let bytes: [u8; 4] = v.to_le_bytes();
        self.mem[start] = bytes[0];
        self.mem[start + 1] = bytes[1];
        self.mem[start + 2] = bytes[2];
        self.mem[start + 3] = bytes[3];
    }

    #[with_ghost_var(trace: &mut Trace)]
    #[external_methods(to_le_bytes)]
    #[requires(self.fits_in_lin_mem_usize(start, 8, trace))]
    #[requires(trace_safe(trace, self.memlen) && ctx_safe(self))]
    #[ensures(trace_safe(trace, self.memlen) && ctx_safe(self))]
    #[ensures(one_effect!(old(trace), trace, effect!(WriteN, addr, 8) if addr == start as usize))]
    #[trusted]
    pub fn write_u64(&mut self, start: usize, v: u64) {
        let bytes: [u8; 8] = v.to_le_bytes();
        self.mem[start] = bytes[0];
        self.mem[start + 1] = bytes[1];
        self.mem[start + 2] = bytes[2];
        self.mem[start + 3] = bytes[3];
        self.mem[start + 4] = bytes[4];
        self.mem[start + 5] = bytes[5];
        self.mem[start + 6] = bytes[6];
        self.mem[start + 7] = bytes[7];
    }

    /// Function for memcpy from sandbox to host
    /// Overwrites contents of vec
    #[with_ghost_var(trace: &mut Trace)]
    #[external_calls(copy_nonoverlapping)]
    #[external_methods(set_len)]
    #[trusted]
    #[requires(dst.capacity() >= (n as usize) )]
    #[requires(self.fits_in_lin_mem(src, n, trace))]
    #[requires(trace_safe(trace, self.memlen) && ctx_safe(self))]
    #[ensures(trace_safe(trace, self.memlen) && ctx_safe(self))]
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
    #[trusted]
    #[requires(src.len() >= (n as usize) )]
    #[requires(self.fits_in_lin_mem(dst, n, trace))]
    #[requires(trace_safe(trace, self.memlen) && ctx_safe(self))]
    #[ensures(trace_safe(trace, self.memlen) && ctx_safe(self))]
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
    #[requires(trace_safe(trace, self.memlen))]
    #[ensures(trace_safe(trace, old(self).memlen))]
    #[ensures(result.len() == (len as usize))]
    #[ensures(no_effect!(old(trace), trace))]
    #[ensures(as_sbox_ptr(result) == old(ptr as usize))]
    #[after_expiry(ctx_safe(self))]
    #[trusted]
    pub fn slice_mem_mut(&mut self, ptr: SboxPtr, len: u32) -> &mut [u8] {
        let start = ptr as usize;
        let end = ptr as usize + len as usize;
        &mut self.mem[start..end]
    }
}
