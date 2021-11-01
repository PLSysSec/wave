use crate::no_effect;
use crate::tcb::misc::nth_bit_set;
#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use extra_args::{external_call, external_method, with_ghost_var};
use prusti_contracts::*;
use std::convert::TryFrom;
use std::ops::Sub;

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
    Eexist,
    Enotempty,
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;

impl From<RuntimeError> for u32 {
    fn from(item: RuntimeError) -> Self {
        let result = match item {
            RuntimeError::Success => 0,
            RuntimeError::Ebadf => libc::EBADF,
            RuntimeError::Emfile => libc::EMFILE,
            RuntimeError::Efault => libc::EFAULT,
            RuntimeError::Einval => libc::EINVAL,
            RuntimeError::Eoverflow => libc::EOVERFLOW,
            RuntimeError::Eio => libc::EIO,
            RuntimeError::Enospc => libc::ENOSPC,
            RuntimeError::Eacces => libc::EACCES,
            RuntimeError::Eexist => libc::EEXIST,
            RuntimeError::Enotempty => libc::ENOTEMPTY,
        };
        result as u32
    }
}

impl RuntimeError {
    /// Returns Ok(()) if the syscall return doesn't correspond to an Errno value.
    /// Returns Err(RuntimeError) if it does.
    #[with_ghost_var(trace: &mut Trace)]
    #[external_call(Ok)]
    #[external_call(Err)]
    #[ensures(no_effect!( old(trace), trace))]
    pub fn from_syscall_ret(ret: usize) -> RuntimeResult<()> {
        // syscall returns between -1 and -4095 are errors, source:
        // https://code.woboq.org/userspace/glibc/sysdeps/unix/sysv/linux/x86_64/sysdep.h.html#369
        if ret < -4095isize as usize {
            return Ok(());
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

        Err(errno)
    }
}

#[repr(transparent)]
pub struct SyscallRet(usize);

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
    pub arg_buffer: Vec<u8>,
    pub argc: usize,
    pub env_buffer: Vec<u8>,
    pub envc: usize,
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

pub struct RelativePath(Vec<u8>);
impl From<RelativePath> for Vec<u8> {
    fn from(w: RelativePath) -> Vec<u8> {
        w.0
    }
}

impl From<Vec<u8>> for RelativePath {
    fn from(w: Vec<u8>) -> RelativePath {
        RelativePath(w)
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

impl ClockId {
    pub fn from_u32(id: u32) -> Option<Self> {
        match id {
            0 => Some(ClockId::Realtime),
            1 => Some(ClockId::Monotonic),
            2 => Some(ClockId::ProcessCpuTimeId),
            3 => Some(ClockId::ThreadCpuTime),
            _ => None,
        }
    }
}

/// Wasi timestamp in nanoseconds
#[repr(transparent)]
#[derive(Eq, PartialEq, Ord, PartialOrd)]
#[cfg_attr(test, derive(Debug))] // needed for assert_eq!
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

    pub fn nsec(&self) -> u64 {
        self.0
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

impl TryFrom<i32> for Advice {
    type Error = RuntimeError;
    fn try_from(advice: i32) -> RuntimeResult<Self> {
        match advice {
            libc::POSIX_FADV_NORMAL => Ok(Advice::Normal),
            libc::POSIX_FADV_SEQUENTIAL => Ok(Advice::Sequential),
            libc::POSIX_FADV_RANDOM => Ok(Advice::Random),
            libc::POSIX_FADV_WILLNEED => Ok(Advice::WillNeed),
            libc::POSIX_FADV_DONTNEED => Ok(Advice::DontNeed),
            libc::POSIX_FADV_NOREUSE => Ok(Advice::NoReuse),
            _ => Err(RuntimeError::Einval),
        }
    }
}

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

impl From<libc::mode_t> for Filetype {
    // must be trusted, bitwise ops not supported in prusti...
    #[trusted]
    fn from(filetype: libc::mode_t) -> Self {
        match filetype & libc::S_IFMT {
            libc::S_IFBLK => Filetype::BlockDevice,
            libc::S_IFCHR => Filetype::CharacterDevice,
            libc::S_IFDIR => Filetype::Directory,
            libc::S_IFREG => Filetype::RegularFile,
            // TODO: need to get socket type, just do unknown for now cause we don't support
            // sockets anyway...
            libc::S_IFSOCK => Filetype::Unknown,
            libc::S_IFLNK => Filetype::SymbolicLink,
            _ => Filetype::Unknown,
        }
    }
}

// TODO: can't use bitflags crate due to prusti issues.
// TODO: pruti doesn't support bitwise operators
//       hmm instead we could have a lame struct full of bools....
type Rights = u64;

#[repr(transparent)]
pub struct FdFlags(u16);

impl FdFlags {
    pub fn empty() -> FdFlags {
        FdFlags(0)
    }

    // trusted due to bitwise ops, can refactor later...
    #[trusted]
    pub fn to_posix(&self) -> i32 {
        let mut flags = 0;
        if self.0 & 1 << 0 != 0 {
            flags |= libc::O_APPEND
        }
        if self.0 & 1 << 1 != 0 {
            flags |= libc::O_DSYNC
        }
        if self.0 & 1 << 2 != 0 {
            flags |= libc::O_NONBLOCK
        }
        if self.0 & 1 << 3 != 0 {
            flags |= libc::O_RSYNC
        }
        if self.0 & 1 << 4 != 0 {
            flags |= libc::O_SYNC
        }
        flags
    }
}

impl From<libc::c_int> for FdFlags {
    // must be trusted, bitwise ops not supported in prusti...
    #[trusted]
    fn from(flags: libc::c_int) -> Self {
        let mut result = FdFlags(0);
        if flags & libc::O_APPEND != 0 {
            result.0 |= 1 << 0;
        }
        if flags & libc::O_DSYNC != 0 {
            result.0 |= 1 << 1;
        }
        if flags & libc::O_NONBLOCK != 0 {
            result.0 |= 1 << 2;
        }
        if flags & libc::O_RSYNC != 0 {
            result.0 |= 1 << 3;
        }
        if flags & libc::O_SYNC != 0 {
            result.0 |= 1 << 4;
        }
        result
    }
}

// TODO: This doesn't exactly match due to layout issues. Could use repr tags to try and make
//       it match, or could have a translation between this and wasm FdStat
//       See: https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#fdstat
pub struct FdStat {
    pub fs_filetype: Filetype,
    pub fs_flags: FdFlags,
    pub fs_rights_base: Rights,
    pub fs_rights_inheriting: Rights,
}

pub struct FileStat {
    dev: u64,
    ino: u64,
    filetype: Filetype,
    nlink: u64,
    size: u64,
    atim: Timestamp,
    mtim: Timestamp,
    ctim: Timestamp,
}

impl From<libc::stat> for FileStat {
    fn from(stat: libc::stat) -> Self {
        FileStat {
            dev: stat.st_dev,
            ino: stat.st_ino,
            filetype: stat.st_mode.into(),
            nlink: stat.st_nlink,
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

pub type LookupFlags = u32;

pub struct FstFlags(u16);

impl FstFlags {
    pub fn new(flags: u16) -> Self {
        FstFlags(flags)
    }

    // must impl flag checking as trusted due to bitwise ops not being supported by prusti
    pub fn atim(&self) -> bool {
        nth_bit_set(self.0, 0)
        //self.0 & (1 << 0) != 0
    }

    pub fn atim_now(&self) -> bool {
        nth_bit_set(self.0, 1)
        //self.0 & (1 << 1) != 0
    }

    pub fn mtim(&self) -> bool {
        nth_bit_set(self.0, 2)
        //self.0 & (1 << 2) != 0
    }

    pub fn mtim_now(&self) -> bool {
        nth_bit_set(self.0, 0)
        //self.0 & (1 << 4) != 0
    }
}

pub struct SdFlags(u8);

impl SdFlags {
    pub fn new(num: u32) -> Self {
        SdFlags(num as u8)
    }

    pub fn rd(&self) -> bool {
        nth_bit_set(self.0.into(), 0)
        //self.0 & (1 << 0) != 0
    }

    pub fn wr(&self) -> bool {
        nth_bit_set(self.0.into(), 1)
        //self.0 & (1 << 1) != 0
    }
}

impl From<SdFlags> for libc::c_int {
    fn from(flags: SdFlags) -> Self {
        if flags.rd() && flags.wr() {
            libc::SHUT_RDWR
        } else if flags.rd() {
            libc::SHUT_RD
        } else if flags.wr() {
            libc::SHUT_WR
        } else {
            // TODO: correct behavior here?
            0
        }
    }
}

// impl TryFrom<libc::c_int> for SdFlags {
//     type Error = RuntimeError;
//     fn try_from(flags: libc::c_int) -> RuntimeResult<Self> {
//         match flags{
//             libc::SHUT_RDRW => Ok()
//             libc::SHUT_RD =>
//             libc::SHUT_WR =>
//         }
//         // if flags.rd() && flags.wr() {
//         //     libc::SHUT_RDWR
//         // } else if flags.rd() {
//         //     libc::SHUT_RD
//         // } else if flags.wr() {
//         //     libc::SHUT_WR
//         // } else {
//         //     // TODO: correct behavior here? (Should it be TryFrom?)
//         //     0
//         // }
//     }
// }

#[repr(C)]
pub struct Subscription {
    userdata: u64,
    subscription_u: SubscriptionInner,
}

#[repr(C, u8)]
pub enum SubscriptionInner {
    Clock(SubscriptionClock),
    FdRead(SubscriptionFdReadWrite),
    FdWrite(SubscriptionFdReadWrite),
}

#[repr(C)]
pub struct SubscriptionClock {
    id: ClockId,
    timeout: Timestamp,
    precision: Timestamp,
    flags: SubClockFlags,
}

#[repr(C)]
pub struct SubscriptionFdReadWrite {
    fd: u32,
}

#[repr(transparent)]
pub struct SubClockFlags(u16);

impl SubClockFlags {
    pub fn subscription_clock_abstime(&self) -> bool {
        nth_bit_set(self.0, 0)
        //self.0 & (1 << 0) != 0
    }
}

impl From<u16> for SubClockFlags {
    fn from(raw: u16) -> Self {
        SubClockFlags(raw)
    }
}

pub struct Event {
    userdata: u64,
    error: Option<RuntimeError>,
    typ: EventType,
    fd_readwrite: EventFdReadWrite,
}

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
    nbytes: u64,
    flags: u16,
}
