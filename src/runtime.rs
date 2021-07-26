#[cfg(feature = "verify")]
use crate::external_specs::option::*;
use crate::types::*;
use prusti_contracts::*;
use std::ptr::{copy, copy_nonoverlapping};
use RuntimeError::*;

// TODO: any other ctx well-formedness checks?
// predicate! {
//     fn fd_safe(ctx: &FdMap) -> bool {
//         forall(|s_fd: SboxFd|
//             (s_fd < MAX_SBOX_FDS ==> ctx.lookup(s_fd) >= 0))
//     }
// }

// TODO: exportable predicates
// Placeholder for ctx well-formedness checks
#[cfg(feature = "verify")]
predicate! {
    fn safe(ctx: &VmCtx) -> bool {
        true
    }
}

//TODO: instantiate stdin,stdout,stderr?
#[ensures(safe(&result))]
pub fn fresh_ctx() -> VmCtx {
    let memlen = LINEAR_MEM_SIZE;
    let mem = vec![0; memlen];
    let fdmap = FdMap::new();
    let ctx = VmCtx {
        mem,
        memlen,
        fdmap,
        errno: Success,
    };
    return ctx;
}

impl VmCtx {
    /// Check whether sandbox pointer is actually inside the sandbox
    #[pure]
    #[ensures((result == true) ==> ptr < self.memlen)]
    pub fn in_lin_mem(&self, ptr: SboxPtr) -> bool {
        return ptr < self.memlen;
    }

    /// Check whether buffer is entirely within sandbox
    #[pure]
    #[ensures(result == true ==> buf < self.memlen && buf + cnt < self.memlen)]
    pub fn fits_in_lin_mem(&self, buf: SboxPtr, cnt: usize) -> bool {
        return self.in_lin_mem(buf) && self.in_lin_mem(buf + cnt);
    }

    /// Copy buffer from sandbox to host
    #[requires(src < self.memlen)]
    #[requires(src + n < self.memlen)]
    #[requires(safe(self))]
    #[ensures(safe(self))]
    #[ensures(result.len() == n)]
    pub fn copy_buf_from_sandbox(&self, src: SboxPtr, n: usize) -> Vec<u8> {
        let mut host_buffer: Vec<u8> = Vec::new();
        host_buffer.reserve_exact(n);
        self.memcpy_from_sandbox(&mut host_buffer, src, n);
        return host_buffer;
    }

    /// Copy buffer from from host to sandbox
    #[requires(src.len() == n)]
    #[requires(safe(self))]
    #[ensures(safe(self))]
    pub fn copy_buf_to_sandbox(&mut self, dst: SboxPtr, src: &Vec<u8>, n: usize) -> Option<()> {
        if !self.fits_in_lin_mem(dst, n) {
            return None;
        }
        self.memcpy_to_sandbox(dst, &src, n);
        Some(())
    }

    /// Function for memcpy from sandbox to host
    /// Overwrites contents of vec
    /// One of 2 unsafe functions (besides syscalls), so needs to be obviously correct
    //TODO: verify that regions do not overlap so that we can use copy_non_overlapping
    #[trusted]
    #[requires(dst.capacity() >= n)]
    #[requires(src < self.memlen)]
    #[requires(src + n < self.memlen)]
    #[ensures(dst.len() == n)]
    pub fn memcpy_from_sandbox(&self, dst: &mut Vec<u8>, src: SboxPtr, n: usize) {
        unsafe {
            copy(self.mem.as_ptr().offset(src as isize), dst.as_mut_ptr(), n);
            dst.set_len(n);
        };
    }

    /// Function for memcpy from sandbox to host
    /// One of 2 unsafe functions (besides syscalls), so needs to be obviously correct
    //TODO: verify that regions do not overlap so that we can use copy_non_overlapping
    #[trusted]
    #[requires(src.len() == n)]
    #[requires(dst < self.memlen)]
    #[requires(dst + n < self.memlen)]
    pub fn memcpy_to_sandbox(&mut self, dst: SboxPtr, src: &Vec<u8>, n: usize) {
        unsafe { copy(src.as_ptr(), self.mem.as_mut_ptr().offset(dst as isize), n) };
    }

    // // pre: {}
    // // post:  { PathSandboxed(out_path) }
    pub fn resolve_path(&self, in_path: Vec<u8>) -> Vec<u8> {
        //TODO: finish
        //memcpy(out_path, in_path, PATH_MAX);
        return in_path;
    }
}
