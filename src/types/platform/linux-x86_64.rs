use super::*;
use crate::tcb::misc::*;
use libc;
use std::convert::TryFrom;

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

impl Dirent {
    // #[requires(in_idx < host_buf.len())]
    // #[flux::sig(fn (host_buf: &RVec<u8>[@len], in_idx: usize{in_idx + 18 < len}) -> Result<Dirent, RuntimeError>)]
    pub fn parse(host_buf: &RVec<u8>, in_idx: usize) -> Result<Dirent, RuntimeError> {
        if in_idx + 18 >= host_buf.len() {
            return Err(RuntimeError::Eoverflow);
        }
        // Inode number
        let d_ino = u64::from_le_bytes([
            host_buf[in_idx + 0],
            host_buf[in_idx + 1],
            host_buf[in_idx + 2],
            host_buf[in_idx + 3],
            host_buf[in_idx + 4],
            host_buf[in_idx + 5],
            host_buf[in_idx + 6],
            host_buf[in_idx + 7],
        ]);

        // Offset to next linux_dirent
        let d_offset = u64::from_le_bytes([
            host_buf[in_idx + 8],
            host_buf[in_idx + 9],
            host_buf[in_idx + 10],
            host_buf[in_idx + 11],
            host_buf[in_idx + 12],
            host_buf[in_idx + 13],
            host_buf[in_idx + 14],
            host_buf[in_idx + 15],
        ]);

        // File type
        let d_type = u8::from_le_bytes([host_buf[in_idx + 18]]);

        // Length of this linux_dirent
        let d_reclen = u16::from_le_bytes([host_buf[in_idx + 16], host_buf[in_idx + 17]]);

        // If we would overflow - don't :)
        if d_reclen < 19 || (in_idx + d_reclen as usize) > host_buf.len() {
            return Err(RuntimeError::Eoverflow);
        }

        let out_namlen = first_null(&host_buf, in_idx, 19, d_reclen as usize);
        // let out_namlen = 3;

        let dirent = Dirent {
            ino: d_ino,
            reclen: d_reclen,
            name_start: 19,
            out_namlen,
            typ: d_type,
        };

        Ok(dirent)
    }
}

impl SockAddr {
    // FLUX-TODO2: type-alias: have to duplicate this as sin_family: u16 vs u8 on mac vs linux (can't use alias!)
    #[flux::trusted]
    #[flux::sig(fn(sin_family: u16, sin_port: u16, sin_addr: u32) -> SockAddr[sin_port, sin_addr])]
    pub fn new(sin_family: u16, sin_port: u16, sin_addr: u32) -> Self {
        SockAddr {
            inner: libc::sockaddr_in {
                // i'll be lazy, should refactor to os-specific code...
                #[cfg(target_os = "macos")]
                sin_len: 0,
                sin_family,
                sin_port,
                sin_addr: libc::in_addr { s_addr: sin_addr },
                sin_zero: [0; 8],
            },
        }
    }
}
