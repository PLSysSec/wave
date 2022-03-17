
use super::super::*;
use crate::tcb::misc::*;
use libc;
use syscall::platform::SyscallReturn;

/// On Mac, posix_advise doesn't exist. Just have the call do nothing.
impl From<Advice> for i32 {
    fn from(advice: Advice) -> Self {
        0
    }
}

impl TryFrom<i32> for Advice {
    type Error = RuntimeError;
    fn try_from(advice: i32) -> RuntimeResult<Self> {
        Ok(Advice::Normal)
    }
}

impl FdFlags {
    pub fn to_posix(&self) -> i32 {
        let mut flags = 0;
        // TODO: DSYNC, RSYNC, and SYNC do not exist on mac. Ignoring for now...
        if nth_bit_set(self.0, 0) {
            flags = bitwise_or(flags, libc::O_APPEND)
        }
        if nth_bit_set(self.0, 2) {
            flags = bitwise_or(flags, libc::O_NONBLOCK)
        }
        flags
    }

    pub fn from_posix(flags: i32) -> Self {
        // FdFlags(flags as u16)
        //let mut result = FdFlags(0);
        // TODO: DSYNC, RSYNC, and SYNC do not exist on mac. Ignoring for now...
        let mut result = FdFlags(0);
        if bitwise_and(flags, libc::O_APPEND) != 0 {
            result.0 = with_nth_bit_set(result.0, 0);
        }
        if bitwise_and(flags, libc::O_NONBLOCK) != 0 {
            result.0 = with_nth_bit_set(result.0, 2);
        }
        result
    }
}

impl RuntimeError {
    /// Returns Ok(()) if the syscall return doesn't correspond to an Errno value.
    /// Returns Err(RuntimeError) if it does.
    #[with_ghost_var(trace: &mut Trace)]
    #[ensures(effects!(old(trace), trace))]
    pub fn from_syscall_ret(ret: SyscallReturn) -> RuntimeResult<usize> {
        let (ret_value, is_error) = ret;
	if !is_error {
            return Ok(ret_value as usize);
        }

        let errno = match ret_value as i32 {
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
