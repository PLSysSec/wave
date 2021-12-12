
use super::*;
use std::convert::TryFrom;
use crate::tcb::misc::*;
use libc;

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
    pub fn to_os_flags(&self) -> i32 {
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

    pub fn from_os_flags(flags: i32) -> Self {
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
