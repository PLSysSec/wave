use crate::tcb::misc::{clone_vec_u8, empty_netlist, get_homedir_fd, string_to_vec_u8};
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
#[external_methods(init_std_fds, unwrap, as_raw_fd, create, to_owned, clone)]
#[external_calls(open, forget, get_homedir_fd)]
pub fn fresh_ctx(homedir: String) -> VmCtx {
    let memlen = LINEAR_MEM_SIZE;
    let mem = vec![0; memlen];
    let mut fdmap = FdMap::new();
    fdmap.init_std_fds();
    let homedir_fd = get_homedir_fd(&homedir);
    // let homedir_file = std::fs::File::open(&homedir).unwrap();
    // let homedir_fd = homedir_file.as_raw_fd();
    if homedir_fd > 0 {
        fdmap.create((homedir_fd as usize).into());
    }
    // Need to forget file to make sure it does not get auto-closed
    // when it gets out of scope
    // std::mem::forget(homedir_file);
    // let log_path = "".to_owned();
    let log_path = String::new();

    let arg_buffer = Vec::new();
    let argc = 0;
    let env_buffer = Vec::new();
    let envc = 0;
    let empty = NetEndpoint {
        protocol: 0,
        addr: 0,
        port: 0,
    };
    // let netlist = [
    //     NetEndpoint {
    //         protocol: 0,
    //         addr: 0,
    //         port: 0,
    //     },
    //     NetEndpoint {
    //         protocol: 0,
    //         addr: 0,
    //         port: 0,
    //     },
    //     NetEndpoint {
    //         protocol: 0,
    //         addr: 0,
    //         port: 0,
    //     },
    //     NetEndpoint {
    //         protocol: 0,
    //         addr: 0,
    //         port: 0,
    //     },
    // ];
    let netlist = empty_netlist();
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
        netlist,
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
    #[with_ghost_var(trace: &mut Trace)]
    #[external_calls(Some, clone_vec_u8)]
    #[requires(self.arg_buffer.len() == (n as usize) )]
    #[requires(trace_safe(trace, self.memlen) && ctx_safe(self))]
    #[ensures(trace_safe(trace, self.memlen) && ctx_safe(self))]
    pub fn copy_arg_buffer_to_sandbox(&mut self, dst: SboxPtr, n: u32) -> Option<()> {
        if !self.fits_in_lin_mem(dst, n) {
            return None;
        }
        // let arg_buffer = self.arg_buffer.clone();
        let arg_buffer = clone_vec_u8(&self.arg_buffer);
        self.memcpy_to_sandbox(dst, &arg_buffer, n);
        Some(())
    }

    /// Copy arg buffer from from host to sandbox
    #[with_ghost_var(trace: &mut Trace)]
    #[external_calls(Some, clone_vec_u8)]
    #[requires(self.env_buffer.len() == (n as usize) )]
    #[requires(trace_safe(trace, self.memlen) && ctx_safe(self))]
    #[ensures(trace_safe(trace, self.memlen) && ctx_safe(self))]
    pub fn copy_environ_buffer_to_sandbox(&mut self, dst: SboxPtr, n: u32) -> Option<()> {
        if !self.fits_in_lin_mem(dst, n) {
            return None;
        }
        // let env_buffer = self.env_buffer.clone();
        let env_buffer = clone_vec_u8(&self.env_buffer);
        self.memcpy_to_sandbox(dst, &env_buffer, n);
        Some(())
    }

    pub fn get_homedir(&self) -> Vec<u8> {
        string_to_vec_u8(&self.homedir)
        // self.homedir.as_bytes().to_vec()
    }

    pub fn in_netlist(&self, domain: u32, ty: u32, proto: u32, addr: u32, port: u32) -> bool {
        let protocol = if domain as i32 == libc::AF_INET && ty as i32 == libc::SOCK_STREAM {
            1
        } else if domain as i32 == libc::AF_INET && ty as i32 == libc::SOCK_DGRAM {
            2
        } else {
            return false;
        };

        let target = NetEndpoint {
            protocol,
            addr,
            port,
        };

        if self.matches_netlist_entry(&target, 0) {
            return true;
        }
        if self.matches_netlist_entry(&target, 1) {
            return true;
        }
        if self.matches_netlist_entry(&target, 2) {
            return true;
        }
        if self.matches_netlist_entry(&target, 3) {
            return true;
        }

        return false;
    }
}
