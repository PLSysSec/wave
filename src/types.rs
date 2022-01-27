use crate::no_effect;
use crate::tcb::misc::*;
#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use extra_args::with_ghost_var;
use prusti_contracts::*;
use std::convert::TryFrom;
use std::ops::Sub;

pub const MAX_SBOX_FDS: u32 = 8;
// pub const MAX_SBOX_FDS_I32: i32 = 8;
pub const MAX_HOST_FDS: usize = 1024;
pub const PATH_MAX: u32 = 1024;

pub const PAGE_SIZE: usize = 4096;
pub const LINEAR_MEM_SIZE: usize = 4294965096; //4GB

pub const HOMEDIR_FD: SboxFd = 3; //4GB

// Note: prusti does not like derive(Debug)

// #[cfg(feature = "verify")]
// predicate! {
//     fn safe(ctx: &VmCtx) -> bool {
//         true
//     }
// }

//typedef char* hostptr;
// pub type HostPtr = usize;
pub type SboxPtr = u32;

// pub type HostFd = usize;
#[derive(Clone, Copy)]
#[cfg_attr(not(feature = "verify"), derive(Debug))]
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
#[cfg_attr(not(feature = "verify"), derive(Debug))]
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
    Enotsup,
    Enotcapable,
    Enotsock,
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;

// Apparently wasi errors are not actually the same numbers as posix errors :(
// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#errno
impl From<RuntimeError> for u32 {
    fn from(item: RuntimeError) -> Self {
        let result = match item {
            RuntimeError::Success => 0,
            RuntimeError::Ebadf => 8,
            RuntimeError::Emfile => 41,
            RuntimeError::Efault => 21,
            RuntimeError::Einval => 28,
            RuntimeError::Eoverflow => 61,
            RuntimeError::Eio => 29,
            RuntimeError::Enospc => 51,
            RuntimeError::Eacces => 2,
            RuntimeError::Eexist => 20,
            RuntimeError::Enotempty => 55,
            RuntimeError::Enotsup => 58,
            RuntimeError::Enotcapable => 76,
            RuntimeError::Enotsock => 57,
        };
        result as u32
    }
}

impl RuntimeError {
    /// Returns Ok(()) if the syscall return doesn't correspond to an Errno value.
    /// Returns Err(RuntimeError) if it does.
    #[with_ghost_var(trace: &mut Trace)]
    #[ensures(no_effect!( old(trace), trace))]
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
        if ret <= 4096 {
            return Err(Self::Einval);
        }

        let ret = -ret;
        let errno = match ret as i32 {
            libc::EBADF => Self::Ebadf,
            libc::EMFILE => Self::Emfile,
            libc::EFAULT => Self::Efault,
            libc::EINVAL => Self::Einval,
            libc::EOVERFLOW => Self::Eoverflow,
            libc::EIO => Self::Eio,
            libc::ENOSPC => Self::Enospc,
            libc::EACCES => Self::Eacces,
            libc::ENOTSOCK => Self::Enotsock,
            _ => Self::Einval, // TODO: what to put here? can't panic cause validator
        };

        Err(errno)
    }
}

#[repr(transparent)]
pub struct SyscallRet(usize);

#[cfg_attr(not(feature = "verify"), derive(Debug))]
pub struct FdMap {
    pub m: Vec<RuntimeResult<HostFd>>,
    pub sockinfo: Vec<RuntimeResult<WasiProto>>,
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
    pub log_path: String,
    pub netlist: Netlist,
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
        match bitwise_and_u32(filetype, libc::S_IFMT) {
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

impl From<Filetype> for libc::mode_t {
    // TODO: returns 0 on unknown, is that correct?
    fn from(filetype: Filetype) -> Self {
        match filetype {
            Filetype::Unknown => 0,
            Filetype::BlockDevice => libc::S_IFBLK,
            Filetype::CharacterDevice => libc::S_IFCHR,
            Filetype::Directory => libc::S_IFDIR,
            Filetype::RegularFile => libc::S_IFREG,
            Filetype::SocketDgram => libc::S_IFSOCK,
            Filetype::SocketStream => libc::S_IFSOCK,
            Filetype::SymbolicLink => libc::S_IFLNK,
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

    pub fn to_posix(&self) -> i32 {
        let mut flags = 0;
        if nth_bit_set(self.0, 0) {
            flags = bitwise_or(flags, libc::O_APPEND)
        }
        if nth_bit_set(self.0, 1) {
            flags = bitwise_or(flags, libc::O_DSYNC)
        }
        if nth_bit_set(self.0, 2) {
            flags = bitwise_or(flags, libc::O_NONBLOCK)
        }
        if nth_bit_set(self.0, 3) {
            flags = bitwise_or(flags, libc::O_RSYNC)
        }
        if nth_bit_set(self.0, 4) {
            flags = bitwise_or(flags, libc::O_SYNC)
        }
        flags
    }

    pub fn from_posix(flags: i32) -> Self {
        // FdFlags(flags as u16)
        //let mut result = FdFlags(0);
        let mut result = FdFlags(0);
        if bitwise_and(flags, libc::O_APPEND) != 0 {
            result.0 = with_nth_bit_set(result.0, 0);
        }
        if bitwise_and(flags, libc::O_DSYNC) != 0 {
            result.0 = with_nth_bit_set(result.0, 1);
        }
        if bitwise_and(flags, libc::O_NONBLOCK) != 0 {
            result.0 = with_nth_bit_set(result.0, 2);
        }
        if bitwise_and(flags, libc::O_RSYNC) != 0 {
            result.0 = with_nth_bit_set(result.0, 3);
        }
        if bitwise_and(flags, libc::O_SYNC) != 0 {
            result.0 = with_nth_bit_set(result.0, 4);
        }
        result
    }
}
// create transparent wrapper around wasi
impl From<libc::c_int> for FdFlags {
    fn from(flags: libc::c_int) -> Self {
        FdFlags(flags as u16)
        // let mut result = FdFlags(0);
        // if bitwise_and(flags, libc::O_APPEND) != 0 {
        //     result.0 = with_nth_bit_set(result.0, 0);
        // }
        // if bitwise_and(flags, libc::O_DSYNC) != 0 {
        //     result.0 = with_nth_bit_set(result.0, 1);
        // }
        // if bitwise_and(flags, libc::O_NONBLOCK) != 0 {
        //     result.0 = with_nth_bit_set(result.0, 2);
        // }
        // if bitwise_and(flags, libc::O_RSYNC) != 0 {
        //     result.0 = with_nth_bit_set(result.0, 3);
        // }
        // if bitwise_and(flags, libc::O_SYNC) != 0 {
        //     result.0 = with_nth_bit_set(result.0, 4);
        // }
        // result
    }
}

// TODO: This doesn't exactly match due to layout issues. Could use repr tags to try and make
//       it match, or could have a translation between this and wasm FdStat
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

pub struct LookupFlags(u32);
impl LookupFlags {
    pub fn new(flags: u32) -> Self {
        LookupFlags(flags)
    }

    pub fn to_posix(&self) -> i32 {
        let mut flags = 0;
        if !nth_bit_set_u32(self.0, 0) {
            flags = bitwise_or(flags, libc::O_NOFOLLOW)
        }
        flags
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
    pub fn new(flags: u16) -> Self {
        FstFlags(flags)
    }

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
        nth_bit_set(self.0, 0)
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

//#[with_ghost_var(trace: &mut Trace)]
pub fn sock_domain_to_posix(domain: u32) -> RuntimeResult<i32> {
    if domain == 1 {
        return Ok(libc::AF_INET);
    }
    return Err(RuntimeError::Enotsup);
}

//#[with_ghost_var(trace: &mut Trace)]
pub fn sock_type_to_posix(ty: u32) -> RuntimeResult<i32> {
    if ty == 6 {
        return Ok(libc::SOCK_STREAM);
    }
    if ty == 5 {
        return Ok(libc::SOCK_DGRAM);
    }
    return Err(RuntimeError::Enotsup);
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

// Higher level protocols
#[derive(Clone, Copy, PartialEq, Eq)]
#[cfg_attr(not(feature = "verify"), derive(Debug))]
#[repr(C)]
pub enum WasiProto {
    Tcp,
    Udp,
    Unknown,
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
