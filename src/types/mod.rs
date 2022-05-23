use crate::effects;
use crate::tcb::misc::*;
use crate::tcb::path::addr_matches_netlist_entry;
#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use prusti_contracts::*;
use std::convert::TryFrom;
use std::ops::Sub;
use wave_macros::{external_calls, external_methods, with_ghost_var};

// manual implementation of the `?` operator because it is currently
// broken in prusti
#[macro_export]
macro_rules! unwrap_result {
    ($p:ident) => {
        let $p = match $p {
            Ok(oc) => oc,
            Err(e) => {
                return Err(e);
            }
        };
    };
}

// include platform specific implementations
#[cfg_attr(
    all(target_os = "macos", target_arch = "aarch64"),
    path = "platform/macos-aarch64.rs"
)]
#[cfg_attr(
    all(target_os = "linux", target_arch = "x86_64"),
    path = "platform/linux-x86_64.rs"
)]
mod platform;
pub use platform::*;

pub const MAX_SBOX_FDS: u32 = 8; // up to 16 or 32?
pub const MAX_HOST_FDS: usize = 1024;
pub const PATH_MAX: usize = 4096;

pub const PAGE_SIZE: usize = 4096;
pub const LINEAR_MEM_SIZE: usize = 4294965096; //4GB

pub const HOMEDIR_FD: SboxFd = 3; //4GB

// Note: prusti does not like derive(Debug)

pub type SboxPtr = u32;
pub type HostPtr = usize;
pub type HostPath = [u8; PATH_MAX];

// pub type HostFd = usize;
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(feature = "verify"), derive(Debug))]
pub struct HostFd(usize);

// Not using impl From, since Prusti has a hard time understanding
// that those conversions are pure
impl HostFd {
    #[pure]
    pub(crate) fn to_raw(&self) -> usize {
        self.0
    }

    //#[pure]
    pub(crate) fn from_raw(w: usize) -> HostFd {
        HostFd(w)
    }
}

// impl From<HostFd> for usize {
//     //#[pure]
//     fn from(w: HostFd) -> usize {
//         w.0
//     }
// }

// impl From<usize> for HostFd {
//     //#[pure]
//     fn from(w: usize) -> HostFd {
//         HostFd(w)
//     }
// }

pub type SboxFd = u32;

#[derive(PartialEq, Eq, Clone, Copy)]
#[cfg_attr(not(feature = "verify"), derive(Debug))]
#[repr(u16)]
pub enum RuntimeError {
    Success = 0,
    Etoobig,
    Eacces,
    Eaddrinuse,
    Eaddrnotavail,
    Eafnosupport,
    Eagain,
    Ealready,
    Ebadf,
    Ebadmsg,
    Ebusy,
    Ecanceled,
    Echild,
    Econnaborted,
    Econnrefused,
    Econnreset,
    Edeadlk,
    Edestaddrreq,
    Edomain,
    Edquot,
    Eexist,
    Efault,
    Efbig,
    Ehostunreach,
    Eidrm,
    Eilseq,
    Einprogress,
    Eintr,
    Einval,
    Eio,
    Eisconn,
    Eisdir,
    Eloop,
    Emfile,
    Emlink,
    Emsgsize,
    Emultihop,
    Enametoolong,
    Enetdown,
    Enetreset,
    Enetunreach,
    Enfile,
    Enobufs,
    Enodev,
    Enoent,
    Enoexec,
    Enolck,
    Enolink,
    Enomem,
    Enomsg,
    Enoprotoopt,
    Enospc,
    Enosys,
    Enotconn,
    Enotdir,
    Enotempty,
    Enotrecoverable,
    Enotsock,
    Enotsup,
    Enotty,
    Enxio,
    Eoverflow,
    Eownerdead,
    Eperm,
    Epipe,
    Eproto,
    Eprotonosupport,
    Eprototype,
    Erange,
    Erofs,
    Espipe,
    Esrch,
    Estale,
    Etimedout,
    Etxtbsy,
    Exdev,
    Enotcapable,
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;

// Wasi errors are not actually the same numbers as posix errors
// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#errno
// WASI constants: https://github.com/WebAssembly/wasi-libc/blob/659ff414560721b1660a19685110e484a081c3d4/libc-bottom-half/headers/public/wasi/api.h#L117-L497
impl From<RuntimeError> for u32 {
    fn from(item: RuntimeError) -> Self {
        as_u32(item)
    }
}

impl From<RuntimeError> for u16 {
    fn from(item: RuntimeError) -> Self {
        as_u16(item)
    }
}

impl RuntimeError {
    /// Returns Ok(()) if the syscall return doesn't correspond to an Errno value.
    /// Returns Err(RuntimeError) if it does.
    #[with_ghost_var(trace: &mut Trace)]
    #[ensures(effects!(old(trace), trace))]
    #[ensures(old(ret >= 0) ==> (match result {
        Ok(r) => r == ret as usize,
        _ => false,
    }))]
    pub fn from_syscall_ret(ret: isize) -> RuntimeResult<usize> {
        // syscall returns between -1 and -4095 are errors, source:
        // https://code.woboq.org/userspace/glibc/sysdeps/unix/sysv/linux/x86_64/sysdep.h.html#369
        // I am treating all negative values on error - we don't support any hostcalls that return negative values on success
        // (e.g., mmap returning a sufficiently large pointer)
        if ret >= 0 {
            return Ok(ret as usize);
        }

        // We support no syscalls that return negative values, so something has gone wronge
        if ret <= -4096 {
            return Err(Self::Einval);
        }

        let ret = -ret;
        let errno = match ret as i32 {
            libc::E2BIG => Self::Etoobig,
            libc::EACCES => Self::Eacces,
            libc::EADDRINUSE => Self::Eaddrinuse,
            libc::EADDRNOTAVAIL => Self::Eaddrnotavail,
            libc::EAFNOSUPPORT => Self::Eafnosupport,
            libc::EAGAIN => Self::Eagain,
            libc::EALREADY => Self::Ealready,
            libc::EBADF => Self::Ebadf,
            libc::EBADMSG => Self::Ebadmsg,
            libc::EBUSY => Self::Ebusy,
            libc::ECANCELED => Self::Ecanceled,
            libc::ECHILD => Self::Echild,
            libc::ECONNABORTED => Self::Econnaborted,
            libc::ECONNREFUSED => Self::Econnrefused,
            libc::ECONNRESET => Self::Econnreset,
            libc::EDEADLK => Self::Edeadlk,
            libc::EDESTADDRREQ => Self::Edestaddrreq,
            // libc::EDOMAIN => Self::Edomain,
            libc::EDQUOT => Self::Edquot,
            libc::EEXIST => Self::Eexist,
            libc::EFAULT => Self::Efault,
            libc::EFBIG => Self::Efbig,
            libc::EHOSTUNREACH => Self::Ehostunreach,
            libc::EIDRM => Self::Eidrm,
            libc::EILSEQ => Self::Eilseq,
            libc::EINPROGRESS => Self::Einprogress,
            libc::EINTR => Self::Eintr,
            libc::EINVAL => Self::Einval,
            libc::EIO => Self::Eio,
            libc::EISCONN => Self::Eisconn,
            libc::EISDIR => Self::Eisdir,
            libc::ELOOP => Self::Eloop,
            libc::EMFILE => Self::Emfile,
            libc::EMLINK => Self::Emlink,
            libc::EMSGSIZE => Self::Emsgsize,
            libc::EMULTIHOP => Self::Emultihop,
            libc::ENAMETOOLONG => Self::Enametoolong,
            libc::ENETDOWN => Self::Enetdown,
            libc::ENETRESET => Self::Enetreset,
            libc::ENETUNREACH => Self::Enetunreach,
            libc::ENFILE => Self::Enfile,
            libc::ENOBUFS => Self::Enobufs,
            libc::ENODEV => Self::Enodev,
            libc::ENOENT => Self::Enoent,
            libc::ENOEXEC => Self::Enoexec,
            libc::ENOLCK => Self::Enolck,
            libc::ENOLINK => Self::Enolink,
            libc::ENOMEM => Self::Enomem,
            libc::ENOMSG => Self::Enomsg,
            libc::ENOPROTOOPT => Self::Enoprotoopt,
            libc::ENOSPC => Self::Enospc,
            libc::ENOSYS => Self::Enosys,
            libc::ENOTCONN => Self::Enotconn,
            libc::ENOTDIR => Self::Enotdir,
            libc::ENOTEMPTY => Self::Enotempty,
            libc::ENOTRECOVERABLE => Self::Enotrecoverable,
            libc::ENOTSOCK => Self::Enotsock,
            libc::ENOTSUP => Self::Enotsup,
            libc::ENOTTY => Self::Enotty,
            libc::ENXIO => Self::Enxio,
            libc::EOVERFLOW => Self::Eoverflow,
            libc::EOWNERDEAD => Self::Eownerdead,
            libc::EPERM => Self::Eperm,
            libc::EPIPE => Self::Epipe,
            libc::EPROTO => Self::Eproto,
            libc::EPROTONOSUPPORT => Self::Eprotonosupport,
            libc::EPROTOTYPE => Self::Eprototype,
            libc::ERANGE => Self::Erange,
            libc::EROFS => Self::Erofs,
            libc::ESPIPE => Self::Espipe,
            libc::ESRCH => Self::Esrch,
            libc::ESTALE => Self::Estale,
            libc::ETIMEDOUT => Self::Etimedout,
            libc::ETXTBSY => Self::Etxtbsy,
            libc::EXDEV => Self::Exdev,
            // libc::ENOTCAPABLE => Self::Enotcapable,
            _ => Self::Einval,
        };

        Err(errno)
    }

    pub fn from_poll_revents(revents: i16) -> RuntimeError {
        if bitwise_and_i16(revents, libc::POLLNVAL) != 0 {
            RuntimeError::Ebadf
        } else if bitwise_and_i16(revents, libc::POLLERR) != 0 {
            RuntimeError::Eio
        } else {
            RuntimeError::Success
        }
    }
}

#[repr(transparent)]
pub struct SyscallRet(usize);

#[derive(PartialEq, Eq)]
#[cfg_attr(not(feature = "verify"), derive(Debug))]
pub struct FdMap {
    pub m: Vec<RuntimeResult<HostFd>>,
    pub sockinfo: Vec<RuntimeResult<WasiProto>>,
    pub reserve: Vec<SboxFd>,
    pub counter: SboxFd,
}

#[derive(PartialEq, Eq)]
pub struct VmCtx {
    pub mem: Vec<u8>,
    pub memlen: usize,
    pub fdmap: FdMap,
    pub homedir: String,
    pub homedir_host_fd: HostFd,
    // pub errno: RuntimeError,
    pub arg_buffer: Vec<u8>,
    pub argc: usize,
    pub env_buffer: Vec<u8>,
    pub envc: usize,
    // pub log_path: String,
    pub netlist: Netlist,
}

// #[cfg_attr(not(feature = "verify"), derive(Debug))]
// pub struct SandboxedPath(Vec<u8>);
// impl From<SandboxedPath> for Vec<u8> {
//     fn from(w: SandboxedPath) -> Vec<u8> {
//         w.0
//     }
// }

// impl From<Vec<u8>> for SandboxedPath {
//     fn from(w: Vec<u8>) -> SandboxedPath {
//         SandboxedPath(w)
//     }
// }

// pub struct RelativePath(Vec<u8>);
// impl From<RelativePath> for Vec<u8> {
//     fn from(w: RelativePath) -> Vec<u8> {
//         w.0
//     }
// }

// impl From<Vec<u8>> for RelativePath {
//     fn from(w: Vec<u8>) -> RelativePath {
//         RelativePath(w)
//     }
// }

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

impl Whence {
    pub fn from_u32(num: u32) -> Option<Self> {
        match num {
            0 => Some(Whence::Set),
            1 => Some(Whence::Cur),
            2 => Some(Whence::End),
            _ => None,
        }
    }
}

pub enum ClockId {
    Realtime,
    Monotonic,
    ProcessCpuTimeId,
    ThreadCpuTime,
}

impl From<ClockId> for libc::clockid_t {
    fn from(id: ClockId) -> Self {
        match id {
            ClockId::Realtime => libc::CLOCK_REALTIME,
            ClockId::Monotonic => libc::CLOCK_MONOTONIC,
            ClockId::ProcessCpuTimeId => libc::CLOCK_PROCESS_CPUTIME_ID,
            ClockId::ThreadCpuTime => libc::CLOCK_THREAD_CPUTIME_ID,
        }
    }
}

impl TryFrom<u32> for ClockId {
    type Error = RuntimeError;

    fn try_from(id: u32) -> RuntimeResult<Self> {
        match id {
            0 => Ok(ClockId::Realtime),
            1 => Ok(ClockId::Monotonic),
            2 => Ok(ClockId::ProcessCpuTimeId),
            3 => Ok(ClockId::ThreadCpuTime),
            _ => Err(RuntimeError::Einval),
        }
    }
}

#[with_ghost_var(trace: &Trace)]
pub fn fresh_libc_timespec() -> libc::timespec {
    libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    }
}

/// Wasi timestamp in nanoseconds
#[repr(transparent)]
#[derive(Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
#[cfg_attr(not(feature = "verify"), derive(Debug))]
pub struct Timestamp(u64);

impl Timestamp {
    pub fn new(nsec: u64) -> Timestamp {
        Timestamp(nsec)
    }

    pub fn max() -> Timestamp {
        Timestamp(u64::MAX)
    }

    pub fn from_sec_nsec(sec: u64, nsec: u64) -> Timestamp {
        let nanos = (sec * 1_000_000_000 + nsec) as u64;
        Timestamp(nanos)
    }

    pub fn to_millis(&self) -> u64 {
        self.0 / 1_000_000
    }

    /// This function converts a Wasi timestamp to a posix ns-timestamp
    /// Specifically it encodes the logic around the UTIME_NOW and UTIME_OMIT
    /// flag as described in https://man7.org/linux/man-pages/man2/utimensat.2.html
    #[with_ghost_var(trace: &Trace)]
    #[external_calls(from)]
    pub fn ts_to_native(self, use_ts: bool, use_now: bool) -> libc::timespec {
        if use_ts {
            libc::timespec::from(self)
        } else {
            let nsec = if use_now {
                libc::UTIME_NOW
            } else {
                libc::UTIME_OMIT
            };
            // when setting tv_nsec to a flag, tv_sec is ignored (see link above)
            libc::timespec {
                tv_sec: 0,
                tv_nsec: nsec,
            }
        }
    }

    pub fn nsec(&self) -> u64 {
        self.0
    }

    pub fn checked_sub(self, rhs: Self) -> Option<Self> {
        self.0.checked_sub(rhs.0).map(|res| Timestamp(res))
    }
}

impl From<libc::timespec> for Timestamp {
    fn from(spec: libc::timespec) -> Timestamp {
        Timestamp::from_sec_nsec(spec.tv_sec as u64, spec.tv_nsec as u64)
    }
}

impl From<Timestamp> for libc::timespec {
    fn from(timestamp: Timestamp) -> Self {
        // nanos must be in range 0 to 999999999
        // see: https://man7.org/linux/man-pages/man2/nanosleep.2.html
        let sec = timestamp.0 / 1000000000;
        let nsec = timestamp.0 % 1000000000;
        libc::timespec {
            tv_sec: sec as i64,
            tv_nsec: nsec as i64,
        }
    }
}

impl From<Timestamp> for u64 {
    fn from(timestamp: Timestamp) -> u64 {
        timestamp.0
    }
}

impl Sub for Timestamp {
    type Output = Timestamp;

    fn sub(self, rhs: Self) -> Self::Output {
        Timestamp(self.0 - rhs.0)
    }
}

pub enum Advice {
    Normal,
    Sequential,
    Random,
    WillNeed,
    DontNeed,
    NoReuse,
}

#[cfg_attr(not(feature = "verify"), derive(Debug))]
pub enum Filetype {
    Unknown,
    BlockDevice,
    CharacterDevice,
    Directory,
    RegularFile,
    SocketDgram,
    SocketStream,
    SymbolicLink,
}

impl Filetype {
    pub fn to_wasi(&self) -> u8 {
        match self {
            Filetype::Unknown => 0,
            Filetype::BlockDevice => 1,
            Filetype::CharacterDevice => 2,
            Filetype::Directory => 3,
            Filetype::RegularFile => 4,
            Filetype::SocketDgram => 5,
            Filetype::SocketStream => 6,
            Filetype::SymbolicLink => 7,
        }
    }
}

impl From<libc::mode_t> for Filetype {
    fn from(filetype: libc::mode_t) -> Self {
        match bitwise_and_u32(filetype.into(), libc::S_IFMT.into()) as libc::mode_t {
            libc::S_IFBLK => Filetype::BlockDevice,
            libc::S_IFCHR => Filetype::CharacterDevice,
            libc::S_IFDIR => Filetype::Directory,
            libc::S_IFREG => Filetype::RegularFile,
            // TODO: This actually means Unix domain socket. Do WASI socket commands even support
            // this?
            libc::S_IFSOCK => Filetype::Unknown,
            libc::S_IFLNK => Filetype::SymbolicLink,
            _ => Filetype::Unknown,
        }
    }
}

type Rights = u64;

// internal representation is the wasi representation
#[cfg_attr(not(feature = "verify"), derive(Debug))]
#[repr(transparent)]
pub struct FdFlags(u16);

impl FdFlags {
    pub fn empty() -> FdFlags {
        FdFlags(0)
    }
}
// create transparent wrapper around wasi
impl From<libc::c_int> for FdFlags {
    fn from(flags: libc::c_int) -> Self {
        FdFlags(flags as u16)
    }
}

//       See: https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#fdstat
#[cfg_attr(not(feature = "verify"), derive(Debug))]
pub struct FdStat {
    pub fs_filetype: Filetype,
    pub fs_flags: FdFlags,
    pub fs_rights_base: Rights,
    pub fs_rights_inheriting: Rights,
}

#[cfg_attr(not(feature = "verify"), derive(Debug))]
pub struct FileStat {
    pub dev: u64,
    pub ino: u64,
    pub filetype: Filetype,
    pub nlink: u64,
    pub size: u64,
    pub atim: Timestamp,
    pub mtim: Timestamp,
    pub ctim: Timestamp,
}

impl From<libc::stat> for FileStat {
    fn from(stat: libc::stat) -> Self {
        FileStat {
            dev: stat.st_dev as u64,
            ino: stat.st_ino,
            filetype: stat.st_mode.into(),
            nlink: stat.st_nlink as u64,
            size: stat.st_size as u64,
            atim: Timestamp::from_sec_nsec(stat.st_atime as u64, stat.st_atime_nsec as u64),
            mtim: Timestamp::from_sec_nsec(stat.st_mtime as u64, stat.st_mtime_nsec as u64),
            ctim: Timestamp::from_sec_nsec(stat.st_ctime as u64, stat.st_ctime_nsec as u64),
        }
    }
}

impl Default for FileStat {
    fn default() -> FileStat {
        FileStat {
            dev: 0,
            ino: 0,
            filetype: Filetype::Unknown,
            nlink: 0,
            size: 0,
            atim: Timestamp::new(0),
            mtim: Timestamp::new(0),
            ctim: Timestamp::new(0),
        }
    }
}

pub struct LookupFlags(u32);
impl LookupFlags {
    pub fn new(flags: u32) -> Self {
        LookupFlags(flags)
    }

    // #[with_ghost_var(trace: &mut Trace)]
    // #[external_calls(bitwise_or, nth_bit_set_u32)]
    #[ensures(!nth_bit_set_u32(self.0, 0) ==> result == bitwise_or(0, libc::AT_SYMLINK_NOFOLLOW))]
    #[ensures(nth_bit_set_u32(self.0, 0) ==> result == 0)]
    pub fn to_stat_posix(&self) -> i32 {
        let mut flags = 0;
        if !nth_bit_set_u32(self.0, 0) {
            flags = bitwise_or(flags, libc::AT_SYMLINK_NOFOLLOW);
        }
        flags
    }

    // annoyingly, these flags are different between the two syscalls
    // #[with_ghost_var(trace: &mut Trace)]
    // #[external_calls(bitwise_or, nth_bit_set_u32)]
    #[ensures(nth_bit_set_u32(self.0, 0) ==> result == bitwise_or(0, libc::AT_SYMLINK_FOLLOW))]
    #[ensures(!nth_bit_set_u32(self.0, 0) ==> result == 0)]
    pub fn to_linkat_posix(&self) -> i32 {
        let mut flags = 0;
        if nth_bit_set_u32(self.0, 0) {
            flags = bitwise_or(flags, libc::AT_SYMLINK_FOLLOW);
        }
        flags
    }

    // #[with_ghost_var(trace: &mut Trace)]
    // #[external_calls(bitwise_or, nth_bit_set_u32)]
    #[ensures(!nth_bit_set_u32(self.0, 0) ==> result == bitwise_or(0, libc::O_NOFOLLOW))]
    #[ensures(nth_bit_set_u32(self.0, 0) ==> result == 0)]
    pub fn to_openat_posix(&self) -> i32 {
        let mut flags = 0;
        if !nth_bit_set_u32(self.0, 0) {
            flags = bitwise_or(flags, libc::O_NOFOLLOW);
        }
        flags
    }

    #[pure]
    #[ensures(result == nth_bit_set_u32(self.0, 0))]
    pub fn should_follow(&self) -> bool {
        nth_bit_set_u32(self.0, 0)
    }
}

pub struct OFlags(u32);
impl OFlags {
    pub fn new(flags: u32) -> Self {
        OFlags(flags)
    }

    pub fn to_posix(&self) -> i32 {
        let mut flags = 0;
        if nth_bit_set_u32(self.0, 0) {
            flags = bitwise_or(flags, libc::O_CREAT)
        }
        if nth_bit_set_u32(self.0, 1) {
            flags = bitwise_or(flags, libc::O_DIRECTORY)
        }
        if nth_bit_set_u32(self.0, 2) {
            flags = bitwise_or(flags, libc::O_EXCL)
        }
        if nth_bit_set_u32(self.0, 3) {
            flags = bitwise_or(flags, libc::O_TRUNC)
        }
        // musl definitions of these flags
        // #define O_RDONLY  00
        // #define O_WRONLY  01
        // #define O_RDWR    02
        if nth_bit_set_u32(self.0, 4) {
            flags = bitwise_or(flags, libc::O_WRONLY)
        }
        if nth_bit_set_u32(self.0, 5) {
            flags = bitwise_or(flags, libc::O_RDWR)
        }
        flags
    }
}

pub struct FstFlags(u16);

impl FstFlags {
    // must impl flag checking as trusted due to bitwise ops not being supported by prusti
    pub fn atim(&self) -> bool {
        nth_bit_set(self.0, 0)
    }

    pub fn atim_now(&self) -> bool {
        nth_bit_set(self.0, 1)
    }

    pub fn mtim(&self) -> bool {
        nth_bit_set(self.0, 2)
    }

    pub fn mtim_now(&self) -> bool {
        nth_bit_set(self.0, 3)
    }
}

impl TryFrom<u16> for FstFlags {
    type Error = RuntimeError;

    fn try_from(flags: u16) -> RuntimeResult<FstFlags> {
        let fst_flags = FstFlags(flags);
        if fst_flags.atim() && fst_flags.atim_now() || fst_flags.mtim() && fst_flags.mtim_now() {
            return Err(RuntimeError::Einval);
        }
        Ok(fst_flags)
    }
}

pub struct SdFlags(u8);

impl SdFlags {
    pub fn new(num: u32) -> Self {
        SdFlags(num as u8)
    }

    pub fn rd(&self) -> bool {
        nth_bit_set(self.0.into(), 0)
    }

    pub fn wr(&self) -> bool {
        nth_bit_set(self.0.into(), 1)
    }
}

impl TryFrom<SdFlags> for libc::c_int {
    type Error = RuntimeError;

    fn try_from(flags: SdFlags) -> RuntimeResult<Self> {
        if flags.rd() && flags.wr() {
            Ok(libc::SHUT_RDWR)
        } else if flags.rd() {
            Ok(libc::SHUT_RD)
        } else if flags.wr() {
            Ok(libc::SHUT_WR)
        } else {
            Err(RuntimeError::Einval)
        }
    }
}

pub struct RiFlags(u16);

impl RiFlags {
    fn recv_peek(&self) -> bool {
        nth_bit_set(self.0, 0)
    }

    fn recv_waitall(&self) -> bool {
        nth_bit_set(self.0, 1)
    }

    pub fn to_posix(&self) -> i32 {
        let mut flags = 0;
        if self.recv_peek() {
            flags = bitwise_or(flags, libc::MSG_PEEK)
        }
        if self.recv_waitall() {
            flags = bitwise_or(flags, libc::MSG_WAITALL)
        }
        flags
    }
}

impl TryFrom<u32> for RiFlags {
    type Error = RuntimeError;

    fn try_from(flags: u32) -> RuntimeResult<RiFlags> {
        // if any bits are set that aren't associated with a wasi flag,
        // return an error
        if bitwise_and_u32(flags, u32::MAX - 0b11) != 0 {
            Err(RuntimeError::Einval)
        } else {
            Ok(RiFlags(flags as u16))
        }
    }
}

pub struct Subscription {
    pub userdata: u64,
    pub subscription_u: SubscriptionInner,
}

impl Subscription {
    pub const WASI_SIZE: u32 = 48;

    pub const CLOCK_TAG: u64 = 0;
    pub const FD_READ_TAG: u64 = 1;
    pub const FD_WRITE_TAG: u64 = 2;

    #[with_ghost_var(trace: &mut Trace)]
    #[external_calls(try_from, is_aligned)]
    #[requires(ctx_safe(ctx))]
    #[requires(trace_safe(trace, ctx))]
    #[ensures(ctx_safe(ctx))]
    #[ensures(trace_safe(trace, ctx))]
    pub fn read(ctx: &VmCtx, ptr: u32) -> RuntimeResult<Subscription> {
        if !ctx.fits_in_lin_mem_usize(ptr as usize, Self::WASI_SIZE as usize) {
            return Err(RuntimeError::Eoverflow);
        }

        if !is_aligned(Alignment::Eight, ptr) {
            return Err(RuntimeError::Einval);
        }

        // read the subscription struct fields
        let userdata = ctx.read_u64(ptr as usize);
        let tag = ctx.read_u64((ptr + 8) as usize);

        match tag {
            Self::CLOCK_TAG => {
                let v_clock_id = ctx.read_u32((ptr + 16) as usize);
                let v_timeout = ctx.read_u64((ptr + 24) as usize);
                let v_precision = ctx.read_u64((ptr + 32) as usize);
                let v_flags = ctx.read_u64((ptr + 40) as usize);

                let precision = Timestamp::new(v_precision);
                let flags = SubClockFlags::try_from(v_flags)?;

                Ok(Subscription {
                    userdata,
                    subscription_u: SubscriptionInner::Clock(SubscriptionClock {
                        id: v_clock_id,
                        timeout: Timestamp::new(v_timeout),
                        precision,
                        flags,
                    }),
                })
            }
            Self::FD_READ_TAG => {
                let v_fd = ctx.read_u32((ptr + 16) as usize);

                Ok(Subscription {
                    userdata,
                    subscription_u: SubscriptionInner::Fd(SubscriptionFdReadWrite {
                        v_fd,
                        typ: SubscriptionFdType::Read,
                    }),
                })
            }
            Self::FD_WRITE_TAG => {
                let v_fd = ctx.read_u32((ptr + 16) as usize);

                Ok(Subscription {
                    userdata,
                    subscription_u: SubscriptionInner::Fd(SubscriptionFdReadWrite {
                        v_fd,
                        typ: SubscriptionFdType::Write,
                    }),
                })
            }
            _ => Err(RuntimeError::Einval),
        }
    }
}

#[repr(C, u8)]
pub enum SubscriptionInner {
    Clock(SubscriptionClock),
    Fd(SubscriptionFdReadWrite),
}

#[derive(Clone)]
#[repr(C)]
pub struct SubscriptionClock {
    pub id: u32,
    pub timeout: Timestamp,
    pub precision: Timestamp,
    pub flags: SubClockFlags,
}

#[repr(C)]
pub struct SubscriptionFdReadWrite {
    pub v_fd: u32,
    pub typ: SubscriptionFdType,
}

#[derive(Copy, Clone)]
pub enum SubscriptionFdType {
    Read,
    Write,
}

impl SubscriptionFdType {
    pub fn to_posix(&self) -> i16 {
        match self {
            Self::Read => libc::POLLIN,
            Self::Write => libc::POLLOUT,
        }
    }

    pub fn to_event_type(&self) -> EventType {
        match self {
            Self::Read => EventType::FdRead,
            Self::Write => EventType::FdWrite,
        }
    }
}

#[derive(Clone)]
#[repr(transparent)]
pub struct SubClockFlags(u16);

impl SubClockFlags {
    pub fn subscription_clock_abstime(&self) -> bool {
        nth_bit_set(self.0, 0)
    }
}

impl TryFrom<u16> for SubClockFlags {
    type Error = RuntimeError;

    fn try_from(flags: u16) -> RuntimeResult<Self> {
        if bitwise_and_u16(flags, u16::MAX - 0x1) != 0 {
            Err(RuntimeError::Einval)
        } else {
            Ok(SubClockFlags(flags))
        }
    }
}

impl TryFrom<u64> for SubClockFlags {
    type Error = RuntimeError;

    fn try_from(flags: u64) -> RuntimeResult<Self> {
        if bitwise_and_u64(flags, u64::MAX - 0x1) != 0 {
            Err(RuntimeError::Einval)
        } else {
            Ok(SubClockFlags(flags as u16))
        }
    }
}

pub struct Event {
    pub userdata: u64,
    pub error: RuntimeError,
    pub typ: EventType,
    pub fd_readwrite: Option<EventFdReadWrite>,
}

impl Event {
    pub const WASI_SIZE: u32 = 32;

    #[with_ghost_var(trace: &mut Trace)]
    #[external_calls(try_from, is_aligned)]
    #[requires(ctx_safe(ctx))]
    #[requires(trace_safe(trace, ctx))]
    #[ensures(ctx_safe(ctx))]
    #[ensures(trace_safe(trace, ctx))]
    pub fn write(&self, ctx: &mut VmCtx, ptr: u32) -> RuntimeResult<()> {
        if !ctx.fits_in_lin_mem_usize(ptr as usize, Self::WASI_SIZE as usize) {
            return Err(RuntimeError::Eoverflow);
        }

        if !is_aligned(Alignment::Eight, ptr) {
            return Err(RuntimeError::Einval);
        }

        // read the subscription struct fields
        ctx.write_u64(ptr as usize, self.userdata);
        ctx.write_u16((ptr + 8) as usize, self.error.into());
        ctx.write_u16((ptr + 10) as usize, self.typ.into());
        if let Some(ref fd_readwrite) = self.fd_readwrite {
            ctx.write_u64((ptr + 16) as usize, fd_readwrite.nbytes);
            ctx.write_u16((ptr + 24) as usize, fd_readwrite.flags.into());
        }

        Ok(())
    }
}

#[derive(Copy, Clone)]
pub enum EventType {
    Clock,
    FdRead,
    FdWrite,
}

impl From<EventType> for u16 {
    fn from(event: EventType) -> Self {
        match event {
            EventType::Clock => 0,
            EventType::FdRead => 1,
            EventType::FdWrite => 2,
        }
    }
}

pub struct EventFdReadWrite {
    pub nbytes: u64,
    pub flags: EventRwFlags,
}

#[derive(Clone, Copy)]
pub struct EventRwFlags(u16);

impl EventRwFlags {
    pub fn from_posix(flags: i16) -> Self {
        let mut result = EventRwFlags(0);
        if bitwise_and_i16(flags, libc::POLLHUP) != 0 {
            result.0 = with_nth_bit_set(result.0, 1);
        }
        result
    }
}

impl From<EventRwFlags> for u16 {
    fn from(flags: EventRwFlags) -> Self {
        flags.0
    }
}

//#[with_ghost_var(trace: &mut Trace)]
pub fn sock_domain_to_posix(domain: u32) -> RuntimeResult<i32> {
    if domain == 1 {
        return Ok(libc::AF_INET);
    }
    Err(RuntimeError::Enotsup)
}

//#[with_ghost_var(trace: &mut Trace)]
pub fn sock_type_to_posix(ty: u32) -> RuntimeResult<i32> {
    if ty == 6 {
        return Ok(libc::SOCK_STREAM);
    }
    if ty == 5 {
        return Ok(libc::SOCK_DGRAM);
    }
    Err(RuntimeError::Enotsup)
}

// protocol 1 = TCP 2 = UDP
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(feature = "verify"), derive(Debug))]
#[repr(C)]
pub struct NetEndpoint {
    // domain: u32,
    // ty: u32,
    pub protocol: WasiProto,
    pub addr: u32,
    pub port: u32,
}

pub type Netlist = [NetEndpoint; 4];

#[pure]
pub fn addr_in_netlist(netlist: &Netlist, addr: u32, port: u32) -> bool {
    if addr_matches_netlist_entry(&netlist, addr, port, 0) {
        return true;
    }
    if addr_matches_netlist_entry(&netlist, addr, port, 1) {
        return true;
    }
    if addr_matches_netlist_entry(&netlist, addr, port, 2) {
        return true;
    }
    if addr_matches_netlist_entry(&netlist, addr, port, 3) {
        return true;
    }

    false
}

// Higher level protocols
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(feature = "verify"), derive(Debug))]
#[repr(C)]
pub enum WasiProto {
    Unknown,
    Tcp,
    Udp,
}

impl WasiProto {
    // domain and type are enough to identify tcp and udp, the only protocols allowed
    pub fn new(domain: i32, ty: i32, _family: i32) -> Self {
        if domain as i32 == libc::AF_INET && ty as i32 == libc::SOCK_STREAM {
            WasiProto::Tcp
        } else if domain as i32 == libc::AF_INET && ty as i32 == libc::SOCK_DGRAM {
            WasiProto::Udp
        } else {
            WasiProto::Unknown
        }
    }
}

pub enum Alignment {
    One,
    Two,
    Four,
    Eight,
}

impl Alignment {
    pub fn align_down_mask(&self) -> u32 {
        match self {
            Alignment::One => 0xFFFF_FFFF,
            Alignment::Two => 0xFFFF_FFFE,
            Alignment::Four => 0xFFFF_FFFC,
            Alignment::Eight => 0xFFFF_FFF8,
        }
    }

    pub fn remainder_mask(&self) -> u32 {
        match self {
            Alignment::One => 0x0,
            Alignment::Two => 0x1,
            Alignment::Four => 0x3,
            Alignment::Eight => 0x7,
        }
    }
}

pub fn is_aligned(alignment: Alignment, value: u32) -> bool {
    bitwise_and_u32(value, alignment.remainder_mask()) == 0
}

pub struct Dirent {
    pub ino: u64,
    pub reclen: u16,
    pub name_start: usize,
    pub out_namlen: usize,
    pub typ: u8,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct WasmIoVec {
    pub iov_base: u32,
    pub iov_len: u32,
}

// Wrapper around Vec<NativeIoVecs> used to make proof cleaner
pub struct WasmIoVecs {
    pub iovs: Vec<WasmIoVec>,
}

impl WasmIoVecs {
    #[pure]
    pub fn len(&self) -> usize {
        self.iovs.len()
    }

    #[ensures(result.len() == 0)]
    pub fn new() -> Self {
        Self { iovs: Vec::new() }
    }

    // Have to dispatch to trusted function because Prusti won't let me
    // inspect a vector inside a proof
    #[pure]
    #[requires(index < self.len())]
    pub fn lookup(&self, index: usize) -> WasmIoVec {
        wasm_iovs_checked_lookup(self, index)
    }
}

#[repr(C)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct NativeIoVec {
    pub iov_base: usize,
    pub iov_len: usize,
}

// Wrapper around Vec<NativeIoVecs> used to make proof cleaner
pub struct NativeIoVecs {
    pub iovs: Vec<NativeIoVec>,
}

impl NativeIoVecs {
    #[pure]
    pub fn len(&self) -> usize {
        self.iovs.len()
    }

    #[ensures(result.len() == 0)]
    pub fn new() -> Self {
        Self { iovs: Vec::new() }
    }

    // Have to dispatch to trusted function because Prusti won't let me
    // inspect a vector inside a proof
    #[pure]
    #[requires(index < self.len())]
    pub fn lookup(&self, index: usize) -> NativeIoVec {
        iovs_checked_lookup(self, index)
    }
}
