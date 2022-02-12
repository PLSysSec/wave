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
    #[after_expiry(ctx_safe(self) && old(self.netlist) == self.netlist)]
    #[trusted]
    pub fn slice_mem_mut(&mut self, ptr: SboxPtr, len: u32) -> &mut [u8] {
        let start = ptr as usize;
        let end = ptr as usize + len as usize;
        &mut self.mem[start..end]
    }
}
