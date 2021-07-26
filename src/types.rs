use prusti_contracts::*;

pub const MAX_SBOX_FDS: usize = 8;
pub const MAX_SBOX_FDS_I32: i32 = 8;
pub const MAX_HOST_FDS: usize = 1024;
pub const PATH_MAX: usize = 1024;

pub const PAGE_SIZE: usize = 4096;
pub const LINEAR_MEM_SIZE: usize = 4294965096; //4GB
                                               // #define SFI_SAFE(ctx) (true) //This is handled by the builtin memory safety checker

// #define FD_SAFE(ctx) (true) // Unimplemented - I think I want to move to rust for better types to implement this
// #define PATH_SAFE(ctx) (true) // Unimplemented - I think I want to move to rust for better types to implement this
// #define RESOURCE_SAFE(ctx) FD_SAFE(ctx) && PATH_SAFE(ctx)

// #define SAFE(ctx) VALID_CTX(ctx) && SFI_SAFE(ctx) && RESOURCE_SAFE(ctx)

// predicate SFISafe(ctx) =
// not exists. a. a < ctx.membase | a >= ctx.membase + ctx.memlength. access(a)

// predicate FdSafe(ctx) =
// not exists. fd. inRevFdMap(ctx, fd) & os_read_fd(fd)

// WASIRead(ctx): ... write at most v_cnt bytes etc.

// validCtx(ctx), SFISafe(ctx), FdSafe(ctx) = ...
#[cfg(feature = "verify")]
predicate! {
    fn safe(ctx: &VmCtx) -> bool {
        true
    }
}

//typedef char* hostptr;
pub type HostPtr = usize;
pub type SboxPtr = usize;

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

// //TODO: is this right?
// impl From<HostFd> for i32 {
//     fn from(w: HostFd) -> i32 {
//         w.0 as i32
//     }
// }

pub type SboxFd = usize;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum RuntimeError {
    Success,
    Ebadf,
    Emfile, // process ran out of file descriptors
    Efault,
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
