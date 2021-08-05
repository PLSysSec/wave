#[cfg(feature = "verify")]
use crate::external_specs::option::*;
use crate::types::*;
use prusti_contracts::*;
use std::ffi::OsString;
use std::os::unix::ffi::OsStringExt;
use std::path::{Component, Path, PathBuf};
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
pub fn fresh_ctx(homedir: String) -> VmCtx {
    let memlen = LINEAR_MEM_SIZE;
    let mem = vec![0; memlen];
    let fdmap = FdMap::new();
    VmCtx {
        mem,
        memlen,
        fdmap,
        homedir,
        errno: Success,
    }
}

impl VmCtx {
    /// Check whether sandbox pointer is actually inside the sandbox
    #[pure]
    #[ensures((result == true) ==> (ptr as usize) < self.memlen)]
    pub fn in_lin_mem(&self, ptr: SboxPtr) -> bool {
        (ptr as usize) < self.memlen
    }

    /// Check whether buffer is entirely within sandbox
    #[pure]
    #[ensures(result == true ==> (buf as usize) < self.memlen && ((buf + cnt) as usize) < self.memlen)]
    pub fn fits_in_lin_mem(&self, buf: SboxPtr, cnt: u32) -> bool {
        self.in_lin_mem(buf) && self.in_lin_mem(buf + cnt)
    }

    /// Copy buffer from sandbox to host
    #[requires(self.fits_in_lin_mem(src, n))]
    #[requires(safe(self))]
    #[ensures(safe(self))]
    #[ensures(result.len() == (n as usize) )]
    pub fn copy_buf_from_sandbox(&self, src: SboxPtr, n: u32) -> Vec<u8> {
        let mut host_buffer: Vec<u8> = Vec::new();
        host_buffer.reserve_exact(n as usize);
        self.memcpy_from_sandbox(&mut host_buffer, src, n);
        host_buffer
    }

    /// Copy buffer from from host to sandbox
    #[requires(src.len() == (n as usize) )]
    #[requires(safe(self))]
    #[ensures(safe(self))]
    pub fn copy_buf_to_sandbox(&mut self, dst: SboxPtr, src: &Vec<u8>, n: u32) -> Option<()> {
        if !self.fits_in_lin_mem(dst, n) {
            return None;
        }
        self.memcpy_to_sandbox(dst, src, n);
        Some(())
    }

    /// Function for memcpy from sandbox to host
    /// Overwrites contents of vec
    /// One of 2 unsafe functions (besides syscalls), so needs to be obviously correct
    //TODO: verify that regions do not overlap so that we can use copy_non_overlapping
    #[trusted]
    #[requires(dst.capacity() >= (n as usize) )]
    #[requires(self.fits_in_lin_mem(src, n))]
    #[ensures(dst.len() == (n as usize) )]
    pub fn memcpy_from_sandbox(&self, dst: &mut Vec<u8>, src: SboxPtr, n: u32) {
        unsafe {
            copy(
                self.mem.as_ptr().offset(src as isize),
                dst.as_mut_ptr(),
                n as usize,
            );
            dst.set_len(n as usize);
        };
    }

    /// Function for memcpy from sandbox to host
    /// One of 2 unsafe functions (besides syscalls), so needs to be obviously correct
    //TODO: verify that regions do not overlap so that we can use copy_non_overlapping
    #[trusted]
    #[requires(src.len() == (n as usize) )]
    #[requires(self.fits_in_lin_mem(dst, n))]
    // #[requires(dst < (self.memlen as u32) )]
    // #[requires(dst + n < (self.memlen as u32) )]
    pub fn memcpy_to_sandbox(&mut self, dst: SboxPtr, src: &Vec<u8>, n: u32) {
        unsafe {
            copy(
                src.as_ptr(),
                self.mem.as_mut_ptr().offset(dst as isize),
                n as usize,
            )
        };
    }

    /// Check whether a path is in the home directory.
    /// If it is, return it as an absolute path, if it isn't, return error
    // TODO: verify and make untrusted
    #[trusted]
    pub fn resolve_path(&self, in_path: Vec<u8>) -> RuntimeResult<SandboxedPath> {
        let path = PathBuf::from(OsString::from_vec(in_path));
        let safe_path = normalize_path(&path);
        let path_str = safe_path.into_os_string();
        if let Ok(s) = path_str.into_string() {
            if self.homedir.starts_with(&s) {
                return Ok(SandboxedPath::from(s.into_bytes()));
            }
        }
        Err(Eacces)
    }

    /// read u32 from wasm linear memory
    // Not thrilled about this implementation, but it works
    pub fn read_u32(&self, start: usize) -> u32 {
        let bytes: [u8; 4] = [
            self.mem[start],
            self.mem[start + 1],
            self.mem[start + 2],
            self.mem[start + 3],
        ];
        u32::from_le_bytes(bytes)
    }

    /// write u32 to wasm linear memory
    // Not thrilled about this implementation, but it works
    pub fn write_u32(&mut self, start: usize, v: u32) {
        let bytes: [u8; 4] = v.to_le_bytes();
        self.mem[start] = bytes[0];
        self.mem[start + 1] = bytes[1];
        self.mem[start + 2] = bytes[2];
        self.mem[start + 3] = bytes[3];
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
