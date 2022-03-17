use crate::effects;
use crate::tcb::misc::*;
use crate::tcb::path::addr_matches_netlist_entry;
#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use prusti_contracts::*;
use std::convert::TryFrom;
use std::ops::Sub;
use wave_macros::{external_calls, external_methods, with_ghost_var};

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

pub const MAX_SBOX_FDS: u32 = 8;
pub const MAX_HOST_FDS: usize = 1024;
pub const PATH_MAX: u32 = 1024;

pub const PAGE_SIZE: usize = 4096;
pub const LINEAR_MEM_SIZE: usize = 4294965096; //4GB

pub const HOMEDIR_FD: SboxFd = 3; //4GB

// Note: prusti does not like derive(Debug)

pub type SboxPtr = u32;

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
    Enotdir,
    Eloop,
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;

// Apparently wasi errors are not actually the same numbers as posix errors :(
// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#errno
// WASI constants: https://github.com/WebAssembly/wasi-libc/blob/659ff414560721b1660a19685110e484a081c3d4/libc-bottom-half/headers/public/wasi/api.h#L117-L497
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
            RuntimeError::Enotdir => 54,
            RuntimeError::Eloop => 32,
        };
        result as u32
    }
}

impl From<RuntimeError> for u16 {
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
            RuntimeError::Enotdir => 54,
            RuntimeError::Eloop => 32,
        };
        result as u16
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
            libc::EBADF => Self::Ebadf,
            libc::EMFILE => Self::Emfile,
            libc::EFAULT => Self::Efault,
            libc::EINVAL => Self::Einval,
            libc::EOVERFLOW => Self::Eoverflow,
            libc::EIO => Self::Eio,
            libc::ENOSPC => Self::Enospc,
            libc::EACCES => Self::Eacces,
            libc::ENOTSOCK => Self::Enotsock,
            libc::ENOTDIR => Self::Enotdir,
            libc::ELOOP => Self::Eloop,
            libc::EEXIST => Self::Eexist,
            libc::ENOTEMPTY => Self::Enotempty,
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

#[cfg_attr(not(feature = "verify"), derive(Debug))]
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

/// Wasi timestamp in nanoseconds
#[repr(transparent)]
#[derive(Eq, PartialEq, Ord, PartialOrd, Clone)]
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

    // annoyingly, these flags are different between the two syscalls
    pub fn to_linkat_posix(&self) -> i32 {
        let mut flags = 0;
        if nth_bit_set_u32(self.0, 0) {
            flags = bitwise_or(flags, libc::AT_SYMLINK_FOLLOW);
        }
        flags
    }

    pub fn to_openat_posix(&self) -> i32 {
        let mut flags = 0;
        if !nth_bit_set_u32(self.0, 0) {
            flags = bitwise_or(flags, libc::O_NOFOLLOW);
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
            0 => {
                let v_clock_id = ctx.read_u32((ptr + 16) as usize);
                let timeout = ctx.read_u64((ptr + 24) as usize);
                let v_precision = ctx.read_u64((ptr + 32) as usize);
                let v_flags = ctx.read_u64((ptr + 40) as usize);

                let precision = Timestamp::new(v_precision);
                let flags = SubClockFlags::try_from(v_flags)?;

                Ok(Subscription {
                    userdata,
                    subscription_u: SubscriptionInner::Clock(SubscriptionClock {
                        id: v_clock_id,
                        timeout,
                        precision,
                        flags,
                    }),
                })
            }
            1 => {
                let v_fd = ctx.read_u32((ptr + 16) as usize);

                Ok(Subscription {
                    userdata,
                    subscription_u: SubscriptionInner::Fd(SubscriptionFdReadWrite {
                        v_fd,
                        typ: SubscriptionFdType::Read,
                    }),
                })
            }
            2 => {
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
    pub timeout: u64,
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
