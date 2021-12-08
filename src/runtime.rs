#[cfg(feature = "verify")]
use crate::tcb::verifier::external_specs::option::*;
#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::types::*;
use crate::{effect, four_effects, no_effect, one_effect, three_effects, two_effects};
use extra_args::{external_calls, external_methods, with_ghost_var};
use prusti_contracts::*;
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::io::AsRawFd;
use std::path::{Component, Path, PathBuf};

use RuntimeError::*;

// Exit codes for wasi-libc: https://github.com/WebAssembly/wasi-libc/blob/659ff414560721b1660a19685110e484a081c3d4/libc-top-half/musl/include/sysexits.h

//#[ensures(safe(&result))]
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(init_std_fds, unwrap, as_raw_fd, create, to_owned)]
#[external_calls(open, forget)]
#[trusted]
pub fn fresh_ctx(homedir: String) -> VmCtx {
    let memlen = LINEAR_MEM_SIZE;
    let mem = vec![0; memlen];
    let mut fdmap = FdMap::new();
    fdmap.init_std_fds();
    let homedir_file = std::fs::File::open(&homedir).unwrap();
    let homedir_fd = homedir_file.as_raw_fd();
    if homedir_fd > 0 {
        fdmap.create((homedir_fd as usize).into());
    }
    // Need to forget file to make sure it does not get auto-closed
    // when it gets out of scope
    std::mem::forget(homedir_file);
    let log_path = "".to_owned();

    let arg_buffer = Vec::new();
    let argc = 0;
    let env_buffer = Vec::new();
    let envc = 0;
    VmCtx {
        mem,
        memlen,
        fdmap,
        homedir,
        errno: Success,
        arg_buffer,
        argc,
        env_buffer,
        envc,
        log_path,
    }
}

impl VmCtx {
    /// Check whether sandbox pointer is actually inside the sandbox
    // TODO: can I eliminate this in favor os in_lin_mem_usize?
    #[with_ghost_var(trace: &mut Trace)]
    #[pure]
    #[ensures((result == true) ==> (ptr as usize) < self.memlen)]
    pub fn in_lin_mem(&self, ptr: SboxPtr) -> bool {
        (ptr as usize) < self.memlen
    }

    #[with_ghost_var(trace: &mut Trace)]
    #[pure]
    #[ensures((result == true) ==> ptr < self.memlen)]
    pub fn in_lin_mem_usize(&self, ptr: usize) -> bool {
        ptr < self.memlen
    }

    /// Check whether buffer is entirely within sandbox
    // Can I eliminate this in favor of fits_in_lin_mem_usize
    #[pure]
    #[with_ghost_var(trace: &mut Trace)]
    #[ensures(result == true ==> (buf as usize) < self.memlen && (buf <= buf + cnt) && (cnt as usize) < self.memlen)]
    pub fn fits_in_lin_mem(&self, buf: SboxPtr, cnt: u32) -> bool {
        let total_size = (buf as usize) + (cnt as usize);
        if total_size > self.memlen || total_size > LINEAR_MEM_SIZE {
            return false;
        }
        self.in_lin_mem(buf) && self.in_lin_mem(cnt) && buf <= buf + cnt
    }

    #[pure]
    #[with_ghost_var(trace: &mut Trace)]
    #[ensures(result == true ==> buf < self.memlen && (buf <= buf + cnt) && cnt < self.memlen)]
    pub fn fits_in_lin_mem_usize(&self, buf: usize, cnt: usize) -> bool {
        let total_size = buf + cnt;
        if total_size > self.memlen || total_size > LINEAR_MEM_SIZE {
            return false;
        }
        self.in_lin_mem_usize(buf) && self.in_lin_mem_usize(cnt) && buf <= buf + cnt
    }

    /// Copy buffer from sandbox to host
    #[with_ghost_var(trace: &mut Trace)]
    #[external_methods(reserve_exact)]
    #[requires(self.fits_in_lin_mem(src, n, trace))]
    #[requires(trace_safe(trace, self.memlen) && ctx_safe(self))]
    #[ensures(trace_safe(trace, self.memlen) && ctx_safe(self))]
    #[ensures(result.len() == (n as usize) )]
    pub fn copy_buf_from_sandbox(&self, src: SboxPtr, n: u32) -> Vec<u8> {
        let mut host_buffer: Vec<u8> = Vec::new();
        host_buffer.reserve_exact(n as usize);
        self.memcpy_from_sandbox(&mut host_buffer, src, n);
        host_buffer
    }

    /// Copy buffer from from host to sandbox
    #[with_ghost_var(trace: &mut Trace)]
    #[external_calls(Some)]
    #[requires(trace_safe(trace, self.memlen) && ctx_safe(self))]
    #[ensures(trace_safe(trace, self.memlen) && ctx_safe(self))]
    #[ensures(self.memlen == old(self.memlen))]
    pub fn copy_buf_to_sandbox(&mut self, dst: SboxPtr, src: &Vec<u8>, n: u32) -> Option<()> {
        if src.len() < n as usize || !self.fits_in_lin_mem(dst, n) {
            return None;
        }
        self.memcpy_to_sandbox(dst, src, n);
        Some(())
    }

    /// Copy arg buffer from from host to sandbox
    /// TODO: make this not trusted
    /// (its only trusted because clone breaks viper for some reason)
    #[with_ghost_var(trace: &mut Trace)]
    #[external_calls(Some)]
    #[trusted]
    #[requires(self.arg_buffer.len() == (n as usize) )]
    #[requires(trace_safe(trace, self.memlen) && ctx_safe(self))]
    #[ensures(trace_safe(trace, self.memlen) && ctx_safe(self))]
    pub fn copy_arg_buffer_to_sandbox(&mut self, dst: SboxPtr, n: u32) -> Option<()> {
        if !self.fits_in_lin_mem(dst, n) {
            return None;
        }
        self.memcpy_to_sandbox(dst, &self.arg_buffer.clone(), n);
        Some(())
    }

    /// Copy arg buffer from from host to sandbox
    /// TODO: make this not trusted
    /// (its only trusted because clone breaks viper for some reason)
    #[with_ghost_var(trace: &mut Trace)]
    #[external_calls(Some)]
    #[trusted]
    #[requires(self.env_buffer.len() == (n as usize) )]
    #[requires(trace_safe(trace, self.memlen) && ctx_safe(self))]
    #[ensures(trace_safe(trace, self.memlen) && ctx_safe(self))]
    pub fn copy_environ_buffer_to_sandbox(&mut self, dst: SboxPtr, n: u32) -> Option<()> {
        if !self.fits_in_lin_mem(dst, n) {
            return None;
        }
        self.memcpy_to_sandbox(dst, &self.env_buffer.clone(), n);
        Some(())
    }

    #[trusted]
    pub fn get_homedir(&self) -> Vec<u8> {
        self.homedir.as_bytes().to_vec()
    }
}
