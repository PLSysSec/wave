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
    // TODO: Currently trusted because it causes a fold-unfold error
    /*#[with_ghost_var(trace: &mut Trace)]
    #[requires(self.fits_in_lin_mem(ptr, len, trace))]
    #[requires(trace_safe(trace, self.memlen) && ctx_safe(self))]
    #[ensures(trace_safe(trace, self.memlen) && ctx_safe(self))]
    #[ensures(result.len() == old(len as usize))]
    #[trusted]
    pub fn slice_mem(&self, ptr: SboxPtr, len: u32) -> &[u8] {
        let start = ptr as usize;
        let end = ptr as usize + len as usize;
        &self.mem[start..end]
    }*/

    // TODO: Currently trusted because it causes a fold-unfold error
    #[with_ghost_var(trace: &mut Trace)]
    #[requires(self.fits_in_lin_mem(ptr, len, trace))]
    #[requires(trace_safe(trace, self.memlen))]
    #[ensures(trace_safe(trace, old(self).memlen))]
    #[ensures(result.len() == (len as usize))]
    #[ensures(no_effect!(old(trace), trace))]
    #[after_expiry(ctx_safe(self))]
    #[trusted]
    pub fn slice_mem_mut(&mut self, ptr: SboxPtr, len: u32) -> &mut [u8] {
        let start = ptr as usize;
        let end = ptr as usize + len as usize;
        &mut self.mem[start..end]
    }

    /// Check whether buffer is entirely within sandbox
    // Can I eliminate this in favor of fits_in_lin_mem_usize
    #[pure]
    #[with_ghost_var(trace: &mut Trace)]
    #[ensures(result == true ==> (buf as usize) < self.memlen && ((buf + cnt) as usize) < self.memlen && (cnt as usize) < self.memlen)]
    pub fn fits_in_lin_mem(&self, buf: SboxPtr, cnt: u32) -> bool {
        let total_size = (buf as usize) + (cnt as usize);
        if total_size > self.memlen || total_size > LINEAR_MEM_SIZE {
            return false;
        }
        self.in_lin_mem(buf) && self.in_lin_mem(cnt) && self.in_lin_mem(buf + cnt)
    }

    #[pure]
    #[with_ghost_var(trace: &mut Trace)]
    #[ensures(result == true ==> buf < self.memlen && (buf + cnt) < self.memlen && cnt < self.memlen)]
    pub fn fits_in_lin_mem_usize(&self, buf: usize, cnt: usize) -> bool {
        let total_size = buf + cnt;
        if total_size > self.memlen || total_size > LINEAR_MEM_SIZE {
            return false;
        }
        self.in_lin_mem_usize(buf) && self.in_lin_mem_usize(cnt) && self.in_lin_mem_usize(buf + cnt)
    }

    ///// Copy buffer from sandbox to host
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

    /// Check whether a path is in the home directory.
    /// If it is, return it as an absolute path, if it isn't, return error
    // TODO: verify and make untrusted
    // #[with_ghost_var(trace: &mut Trace)]
    #[trusted]
    // #[requires(trace_safe(trace, self.memlen) && ctx_safe(self))]
    // #[ensures(trace_safe(trace, self.memlen) && ctx_safe(self))]
    pub fn resolve_path(&self, in_path: Vec<u8>) -> RuntimeResult<SandboxedPath> {
        let path = PathBuf::from(OsString::from_vec(in_path));
        println!("resolve_path: path = {:?}", path);
        let safe_path = PathBuf::from(self.homedir.clone()).join(normalize_path(&path));
        println!("safe_path: safe_path = {:?}", safe_path);
        let path_str = safe_path.into_os_string();
        //println!("path_str = {:?}, into_string = ", path_str, path_str.into_string());
        if let Ok(s) = path_str.into_string() {
            println!("Checking prefix of s = {:?}", s);
            if s.starts_with(&self.homedir) {
                return Ok(SandboxedPath::from(s.into_bytes()));
            }
        }
        Err(Eacces)
    }

    #[trusted]
    pub fn get_homedir(&self) -> Vec<u8> {
        self.homedir.as_bytes().to_vec()
    }
}

/// Convert relative path to absolute path
/// Used to check that that paths are sandboxed
// TODO: verify this
// Prusti does not like this function at all

#[trusted]
pub fn normalize_path(path: &PathBuf) -> PathBuf {
    let mut components = path.components().peekable();
    let mut ret = if let Some(c @ Component::Prefix(..)) = components.peek().cloned() {
        components.next();
        PathBuf::from(c.as_os_str())
    } else {
        PathBuf::new()
    };

    for component in components {
        match component {
            Component::Prefix(..) => unreachable!(),
            Component::RootDir => {
                ret.push(component.as_os_str());
            }
            Component::CurDir => {}
            Component::ParentDir => {
                ret.pop();
            }
            Component::Normal(c) => {
                ret.push(c);
            }
        }
    }
    ret
}
