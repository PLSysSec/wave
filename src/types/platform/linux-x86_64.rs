use super::*;
use crate::tcb::misc::*;
use libc;
use std::convert::TryFrom;
use tcb::os_specs::SyscallReturn;

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

impl FdFlags {
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

impl RuntimeError {
    /// Returns Ok(()) if the syscall return doesn't correspond to an Errno value.
    /// Returns Err(RuntimeError) if it does.
    #[with_ghost_var(trace: &mut Trace)]
    #[ensures(effects!(old(trace), trace))]
    #[ensures(old(ret >= 0) ==> (match result {
        Ok(r) => r == ret as usize,
        _ => false,
    }))]
    pub fn from_syscall_ret(ret: SyscallReturn) -> RuntimeResult<usize> {
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
}
