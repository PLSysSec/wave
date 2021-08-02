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
#[cfg_attr(test, derive(Debug))] // needed for assert_eq!
pub enum RuntimeError {
    Success,
    Ebadf,
    Emfile, // process ran out of file descriptors
    Efault,
    Einval,
    Eoverflow,
    Eio,
    Enospc,
    Eacces,
}

impl RuntimeError {
    /// Returns Some(RuntimeError) if the passed in `ret` value from a syscall corresponds to
    /// some Errno value. None otherwise.
    pub fn from_syscall_ret(ret: usize) -> Option<RuntimeError> {
        // syscall returns between -1 and -4095 are errors, source:
        // https://code.woboq.org/userspace/glibc/sysdeps/unix/sysv/linux/x86_64/sysdep.h.html#369
        if ret < -4095isize as usize {
            return None;
        }

        let ret = -(ret as i32);
        let errno = match ret {
            libc::EBADF => Self::Ebadf,
            libc::EMFILE => Self::Emfile,
            libc::EFAULT => Self::Efault,
            libc::EINVAL => Self::Einval,
            libc::EOVERFLOW => Self::Eoverflow,
            libc::EIO => Self::Eio,
            libc::ENOSPC => Self::Enospc,
            libc::EACCES => Self::Eacces,
            _ => Self::Einval, // TODO: what to put here? can't panic cause validator
        };

        Some(errno)
    }
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
    pub homedir: String,
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

impl From<Whence> for i32 {
    fn from(whence: Whence) -> Self {
        match whence {
            Whence::Set => libc::SEEK_SET,
            Whence::Cur => libc::SEEK_CUR,
            Whence::End => libc::SEEK_END,
        }
    }
}

pub enum ClockId {
    Realtime,
    Monotonic,
    ProcessCpuTimeId,
    ThreadCpuTime,
}

impl From<ClockId> for i32 {
    fn from(id: ClockId) -> Self {
        match id {
            ClockId::Realtime => libc::CLOCK_REALTIME,
            ClockId::Monotonic => libc::CLOCK_MONOTONIC,
            ClockId::ProcessCpuTimeId => libc::CLOCK_PROCESS_CPUTIME_ID,
            ClockId::ThreadCpuTime => libc::CLOCK_THREAD_CPUTIME_ID,
        }
    }
}

/// Wasi timestamp in nanoseconds
pub type Timestamp = u64;

pub enum Advice {
    Normal,
    Sequential,
    Random,
    WillNeed,
    DontNeed,
    NoReuse,
}

impl From<Advice> for i32 {
    fn from(advice: Advice) -> Self {
        match advice {
            Advice::Normal => libc::POSIX_FADV_NORMAL,
            Advice::Sequential => libc::POSIX_FADV_SEQUENTIAL,
            Advice::Random => libc::POSIX_FADV_RANDOM,
            Advice::WillNeed => libc::POSIX_FADV_WILLNEED,
            Advice::DontNeed => libc::POSIX_FADV_DONTNEED,
            Advice::NoReuse => libc::POSIX_FADV_NOREUSE,
        }
    }
}
