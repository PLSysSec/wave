use prusti_contracts::*;

pub const MAX_SBOX_FDS: u32 = 8;
// pub const MAX_SBOX_FDS_I32: i32 = 8;
pub const MAX_HOST_FDS: usize = 1024;
pub const PATH_MAX: u32 = 1024;

pub const PAGE_SIZE: usize = 4096;
pub const LINEAR_MEM_SIZE: usize = 4294965096; //4GB

#[cfg(feature = "verify")]
predicate! {
    fn safe(ctx: &VmCtx) -> bool {
        true
    }
}

//typedef char* hostptr;
// pub type HostPtr = usize;
pub type SboxPtr = u32;

// pub type HostFd = usize;
#[derive(Clone, Copy)]
pub struct HostFd(usize);
impl From<HostFd> for usize {
    fn from(w: HostFd) -> usize {
        w.0
    }
}

impl From<usize> for HostFd {
    fn from(w: usize) -> HostFd {
        HostFd(w)
    }
}

pub type SboxFd = u32;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum RuntimeError {
    Success,
    Ebadf,
    Emfile, // process ran out of file descriptors
    Efault,
    Einval,
    Eoverflow,
    Eio,
    Enospc,
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;

pub struct FdMap {
    pub m: Vec<RuntimeResult<HostFd>>,
    pub reserve: Vec<SboxFd>,
    pub counter: SboxFd,
}

pub struct VmCtx {
    pub mem: Vec<u8>,
    pub memlen: usize,
    pub fdmap: FdMap,
    pub errno: RuntimeError,
}

pub struct SandboxedPath(Vec<u8>);
impl From<SandboxedPath> for Vec<u8> {
    fn from(w: SandboxedPath) -> Vec<u8> {
        w.0
    }
}

impl From<Vec<u8>> for SandboxedPath {
    fn from(w: Vec<u8>) -> SandboxedPath {
        SandboxedPath(w)
    }
}

pub enum Whence {
    Set,
    Cur,
    End,
}
