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
    #[with_ghost_var(trace: &Trace)]
    #[pure]
    #[ensures((result == true) ==> (ptr as usize) < self.memlen)]
    pub fn in_lin_mem(&self, ptr: SboxPtr) -> bool {
        (ptr as usize) < self.memlen
    }

    #[with_ghost_var(trace: &Trace)]
    #[pure]
    #[ensures((result == true) ==> ptr < self.memlen)]
    pub fn in_lin_mem_usize(&self, ptr: usize) -> bool {
        ptr < self.memlen
    }

    /// Check whether buffer is entirely within sandbox
    // Can I eliminate this in favor of fits_in_lin_mem_usize
    #[pure]
    #[with_ghost_var(trace: &Trace)]
    #[ensures(result == true ==> (buf as usize) < self.memlen && (buf <= buf + cnt) && (cnt as usize) < self.memlen)]
    pub fn fits_in_lin_mem(&self, buf: SboxPtr, cnt: u32) -> bool {
        let total_size = (buf as usize) + (cnt as usize);
        if total_size > self.memlen || total_size > LINEAR_MEM_SIZE {
            return false;
        }
        self.in_lin_mem(buf) && self.in_lin_mem(cnt) && buf <= buf + cnt
    }

    #[pure]
    #[with_ghost_var(trace: &Trace)]
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
    #[requires(ctx_safe(self))]
    #[requires(trace_safe(trace, self.memlen))]
    #[ensures(ctx_safe(self))]
    #[ensures(trace_safe(trace, self.memlen))]
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
    #[requires(ctx_safe(self))]
    #[requires(trace_safe(trace, self.memlen))]
    #[ensures(ctx_safe(self))]
    #[ensures(trace_safe(trace, self.memlen))]
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
    #[requires(ctx_safe(self))]
    #[requires(trace_safe(trace, self.memlen))]
    #[ensures(ctx_safe(self))]
    #[ensures(trace_safe(trace, self.memlen))]
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
    #[requires(ctx_safe(self))]
    #[requires(trace_safe(trace, self.memlen))]
    #[ensures(ctx_safe(self))]
    #[ensures(trace_safe(trace, self.memlen))]
    pub fn copy_environ_buffer_to_sandbox(&mut self, dst: SboxPtr, n: u32) -> Option<()> {
        if !self.fits_in_lin_mem(dst, n) {
            return None;
        }
        // let env_buffer = self.env_buffer.clone();
        let env_buffer = clone_vec_u8(&self.env_buffer);
        self.memcpy_to_sandbox(dst, &env_buffer, n);
        Some(())
    }

    #[with_ghost_var(trace: &mut Trace)]
    #[external_methods(resolve_path)]
    #[requires(ctx_safe(self))]
    #[requires(trace_safe(trace, self.memlen))]
    #[ensures(trace_safe(trace, self.memlen))]
    #[ensures(ctx_safe(self))]
    pub fn translate_path(&self, path: SboxPtr, path_len: u32) -> RuntimeResult<SandboxedPath> {
        if !self.fits_in_lin_mem(path, path_len) {
            return Err(Eoverflow);
        }
        let host_buffer = self.copy_buf_from_sandbox(path, path_len);
        self.resolve_path(host_buffer)
    }

    pub fn get_homedir(&self) -> Vec<u8> {
        string_to_vec_u8(&self.homedir)
        // self.homedir.as_bytes().to_vec()
    }

    #[pure]
    pub fn in_netlist(&self, proto: WasiProto, addr: u32, port: u32) -> bool {
        if self.matches_netlist_entry(proto, addr, port, 0) {
            return true;
        }
        if self.matches_netlist_entry(proto, addr, port, 1) {
            return true;
        }
        if self.matches_netlist_entry(proto, addr, port, 2) {
            return true;
        }
        if self.matches_netlist_entry(proto, addr, port, 3) {
            return true;
        }

        false
    }

    #[pure]
    pub fn addr_in_netlist(&self, addr: u32, port: u32) -> bool {
        if self.addr_matches_netlist_entry(addr, port, 0) {
            return true;
        }
        if self.addr_matches_netlist_entry(addr, port, 1) {
            return true;
        }
        if self.addr_matches_netlist_entry(addr, port, 2) {
            return true;
        }
        if self.addr_matches_netlist_entry(addr, port, 3) {
            return true;
        }

        false
    }

    /// read u16 from wasm linear memory
    // Not thrilled about this implementation, but it works
    #[with_ghost_var(trace: &mut Trace)]
    #[external_calls(from_le_bytes)]
    #[requires(self.fits_in_lin_mem_usize(start, 2, trace))]
    #[requires(ctx_safe(self))]
    #[requires(trace_safe(trace, self.memlen))]
    #[ensures(ctx_safe(self))]
    #[ensures(trace_safe(trace, self.memlen))]
    // #[ensures(one_effect!(old(trace), trace, effect!(ReadN, addr, 2) if addr == start as usize))]
    // #[trusted]
    pub fn read_u16(&self, start: usize) -> u16 {
        let bytes: [u8; 2] = [self.mem[start], self.mem[start + 1]];
        u16::from_le_bytes(bytes)
    }

    /// read u32 from wasm linear memory
    // Not thrilled about this implementation, but it works
    #[with_ghost_var(trace: &mut Trace)]
    #[external_calls(from_le_bytes)]
    #[requires(self.fits_in_lin_mem_usize(start, 4, trace))]
    #[requires(ctx_safe(self))]
    #[requires(trace_safe(trace, self.memlen))]
    #[ensures(ctx_safe(self))]
    #[ensures(trace_safe(trace, self.memlen))]
    // #[ensures(one_effect!(old(trace), trace, effect!(ReadN, addr, 4) if addr == start as usize))]
    // #[trusted]
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
    #[requires(ctx_safe(self))]
    #[requires(trace_safe(trace, self.memlen))]
    #[ensures(ctx_safe(self))]
    #[ensures(trace_safe(trace, self.memlen))]
    // #[ensures(one_effect!(old(trace), trace, effect!(ReadN, addr, 8) if addr == start as usize))]
    // #[trusted]
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
    #[requires(ctx_safe(self))]
    #[requires(trace_safe(trace, self.memlen))]
    #[ensures(ctx_safe(self))]
    #[ensures(trace_safe(trace, self.memlen))]
    // #[ensures(one_effect!(old(trace), trace, effect!(WriteN, addr, 2) if addr == start as usize))]
    // #[trusted]
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
    #[requires(ctx_safe(self))]
    #[requires(trace_safe(trace, self.memlen))]
    #[ensures(ctx_safe(self))]
    #[ensures(trace_safe(trace, self.memlen))]
    // #[ensures(one_effect!(old(trace), trace, effect!(WriteN, addr, 4) if addr == start as usize))]
    // #[trusted]
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
    #[requires(ctx_safe(self))]
    #[requires(trace_safe(trace, self.memlen))]
    #[ensures(ctx_safe(self))]
    #[ensures(trace_safe(trace, self.memlen))]
    // #[ensures(one_effect!(old(trace), trace, effect!(WriteN, addr, 8) if addr == start as usize))]
    // #[trusted]
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
}
