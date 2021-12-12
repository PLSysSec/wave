
use super::*;
use crate::tcb::misc::*;
use libc;

/// On Mac, posix_advise doesn't exist. Just have the call do nothing.
impl From<Advice> for i32 {
    fn from(advice: Advice) -> Self {
        0
    }
}

impl TryFrom<i32> for Advice {
    type Error = RuntimeError;
    fn try_from(advice: i32) -> RuntimeResult<Self> {
        Ok(Advice::Normal),
    }
}

impl FdFlags {
    pub fn to_os_flags(&self) -> i32 {
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

    pub fn from_os_flags(flags: i32) -> Self {
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
