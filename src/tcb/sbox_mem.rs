#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::types::*;
use extra_args::{external_calls, external_methods, with_ghost_var};
use prusti_contracts::*;
use std::ptr::{copy, copy_nonoverlapping};
use crate::{effect, four_effects, no_effect, one_effect, three_effects, two_effects};

//TODO: effects annotations

impl VmCtx {
    /// read u16 from wasm linear memory
    // Not thrilled about this implementation, but it works
    #[with_ghost_var(trace: &mut Trace)]
    #[external_calls(from_le_bytes)]
    #[requires(self.fits_in_lin_mem_usize(start, 2, trace))]
    #[requires(trace_safe(trace, self.memlen) && ctx_safe(self))]
    #[ensures(trace_safe(trace, self.memlen) && ctx_safe(self))]
    pub fn read_u16(&self, start: usize) -> u16 {
        let bytes: [u8; 2] = [self.mem[start], self.mem[start + 1]];
        // effect!(trace, Effect::ReadN(2));
        u16::from_le_bytes(bytes)
    }

    /// read u32 from wasm linear memory
    // Not thrilled about this implementation, but it works
    #[with_ghost_var(trace: &mut Trace)]
    #[external_calls(from_le_bytes)]
    #[requires(self.fits_in_lin_mem_usize(start, 4, trace))]
    #[requires(trace_safe(trace, self.memlen) && ctx_safe(self))]
    #[ensures(trace_safe(trace, self.memlen) && ctx_safe(self))]
    pub fn read_u32(&self, start: usize) -> u32 {
        let bytes: [u8; 4] = [
            self.mem[start],
            self.mem[start + 1],
            self.mem[start + 2],
            self.mem[start + 3],
        ];
        // effect!(trace, Effect::ReadN(4));
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
        // effect!(trace, Effect::ReadN(8));
        u64::from_le_bytes(bytes)
    }

    /// write u16 to wasm linear memory
    // Not thrilled about this implementation, but it works
    #[with_ghost_var(trace: &mut Trace)]
    #[external_methods(to_le_bytes)]
    #[requires(self.fits_in_lin_mem_usize(start, 2, trace))]
    #[requires(trace_safe(trace, self.memlen) && ctx_safe(self))]
    #[ensures(trace_safe(trace, self.memlen) && ctx_safe(self))]
    pub fn write_u16(&mut self, start: usize, v: u16) {
        let bytes: [u8; 2] = v.to_le_bytes();
        self.mem[start] = bytes[0];
        self.mem[start + 1] = bytes[1];
        // effect!(trace, Effect::WriteN(2));
    }

    /// write u32 to wasm linear memory
    // Not thrilled about this implementation, but it works
    #[with_ghost_var(trace: &mut Trace)]
    #[external_methods(to_le_bytes)]
    #[requires(self.fits_in_lin_mem_usize(start, 4, trace))]
    #[requires(trace_safe(trace, self.memlen) && ctx_safe(self))]
    #[ensures(trace_safe(trace, self.memlen) && ctx_safe(self))]
    pub fn write_u32(&mut self, start: usize, v: u32) {
        let bytes: [u8; 4] = v.to_le_bytes();
        self.mem[start] = bytes[0];
        self.mem[start + 1] = bytes[1];
        self.mem[start + 2] = bytes[2];
        self.mem[start + 3] = bytes[3];
        // effect!(trace, Effect::WriteN(4));
    }

    #[with_ghost_var(trace: &mut Trace)]
    #[external_methods(to_le_bytes)]
    #[requires(self.fits_in_lin_mem_usize(start, 8, trace))]
    #[requires(trace_safe(trace, self.memlen) && ctx_safe(self))]
    #[ensures(trace_safe(trace, self.memlen) && ctx_safe(self))]
    // #[ensures(one_effect!(old(trace), trace, Effect::WriteN(8)))]
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
        // effect!(trace, Effect::WriteN(8));
    }

    /// Function for memcpy from sandbox to host
    /// Overwrites contents of vec
    //TODO: Add effects annotation
    #[with_ghost_var(trace: &mut Trace)]
    #[external_calls(copy_nonoverlapping)]
    #[external_methods(set_len)]
    #[trusted]
    #[requires(dst.capacity() >= (n as usize) )]
    #[requires(self.fits_in_lin_mem(src, n, trace))]
    #[requires(trace_safe(trace, self.memlen) && ctx_safe(self))]
    #[ensures(trace_safe(trace, self.memlen) && ctx_safe(self))]
    #[ensures(dst.len() == (n as usize) )]
    pub fn memcpy_from_sandbox(&self, dst: &mut Vec<u8>, src: SboxPtr, n: u32) {
        unsafe {
            copy_nonoverlapping(
                self.mem.as_ptr().offset(src as isize),
                dst.as_mut_ptr(),
                n as usize,
            );
            dst.set_len(n as usize);
        };
    }

    /// Function for memcpy from sandbox to host
    /// One of 2 unsafe functions (besides syscalls), so needs to be obviously correct
    //TODO: add effects annotation
    #[with_ghost_var(trace: &mut Trace)]
    #[external_calls(copy_nonoverlapping)]
    #[trusted]
    #[requires(src.len() >= (n as usize) )]
    #[requires(self.fits_in_lin_mem(dst, n, trace))]
    #[requires(trace_safe(trace, self.memlen) && ctx_safe(self))]
    #[ensures(trace_safe(trace, self.memlen) && ctx_safe(self))]
    pub fn memcpy_to_sandbox(&mut self, dst: SboxPtr, src: &Vec<u8>, n: u32) {
        unsafe {
            copy_nonoverlapping(
                src.as_ptr(),
                self.mem.as_mut_ptr().offset(dst as isize),
                n as usize,
            )
        };
    }
}
