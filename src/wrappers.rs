#[cfg(feature = "verify")]
use crate::external_specs::result::*;
use crate::os::*;
use crate::runtime::*;
use crate::types::*;
use prusti_contracts::*;
use std::convert::TryInto;
use RuntimeError::*;

// Note: Prusti can't really handle iterators, so we need to use while loops

#[cfg(feature = "verify")]
predicate! {
    fn safe(ctx: &VmCtx) -> bool {
        true
    }
}

macro_rules! exit_with_errno {
    ($ctx:ident, $errno:ident) => {
        $ctx.errno = $errno;
        return u32::MAX
    };
}

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_path_open(ctx: &mut VmCtx, pathname: u32, flags: i32) -> u32 {
    if !ctx.fits_in_lin_mem(pathname, PATH_MAX) {
        exit_with_errno!(ctx, Efault);
    }

    let host_buffer = ctx.copy_buf_from_sandbox(pathname, PATH_MAX);
    if let Ok(host_pathname) = ctx.resolve_path(host_buffer) {
        let fd = os_open(host_pathname, flags);
        let sbox_fd = ctx.fdmap.create(fd.into());
        if let Ok(s_fd) = sbox_fd {
            return s_fd;
        }
        exit_with_errno!(ctx, Ebadf);
    }
    exit_with_errno!(ctx, Eacces);
}

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_close(ctx: &mut VmCtx, v_fd: u32) -> u32 {
    if v_fd >= MAX_SBOX_FDS {
        exit_with_errno!(ctx, Ebadf);
    }
    if let Ok(fd) = ctx.fdmap.m[v_fd as usize] {
        ctx.fdmap.delete(v_fd);
        return os_close(fd) as u32;
    }
    exit_with_errno!(ctx, Ebadf);
}

//TODO: fix return type
#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_fd_read(ctx: &mut VmCtx, v_fd: u32, iovs: u32, iovcnt: u32) -> u32 {
    if v_fd >= MAX_SBOX_FDS {
        exit_with_errno!(ctx, Ebadf);
    }
    if let Ok(fd) = ctx.fdmap.m[v_fd as usize] {
        let mut num: u32 = 0;
        let mut i = 0;
        while i < iovcnt {
            let start = (iovs + i * 8) as usize;
            let ptr = ctx.read_u32(start);
            let len = ctx.read_u32(start + 4);
            if !ctx.fits_in_lin_mem(ptr, len) {
                exit_with_errno!(ctx, Efault);
            }
            let mut buf: Vec<u8> = Vec::new();
            buf.reserve_exact(len as usize);
            let result = os_read(fd, &mut buf, len as usize) as u32;
            if result > len {
                //TODO: pass through os_read's errno?
                return u32::MAX;
            }
            let copy_ok = ctx.copy_buf_to_sandbox(ptr, &buf, result);
            if copy_ok.is_none() {
                exit_with_errno!(ctx, Efault);
            }
            num += result;
            i += 1;
        }
        return num;
    }
    exit_with_errno!(ctx, Ebadf);
}

//TODO: fix return type
#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_fd_write(ctx: &mut VmCtx, v_fd: u32, iovs: u32, iovcnt: u32) -> u32 {
    if v_fd >= MAX_SBOX_FDS {
        exit_with_errno!(ctx, Ebadf);
    }

    if let Ok(fd) = ctx.fdmap.m[v_fd as usize] {
        let mut num: u32 = 0;
        let mut i = 0;
        while i < iovcnt {
            let start = (iovs + i * 8) as usize;
            let ptr = ctx.read_u32(start);
            let len = ctx.read_u32(start + 4);
            if !ctx.fits_in_lin_mem(ptr, len) {
                exit_with_errno!(ctx, Efault);
            }
            let host_buffer = ctx.copy_buf_from_sandbox(ptr, len);
            let result = os_write(fd, &host_buffer, len as usize) as u32;
            num += result;
            i += 1;
        }
        return num;
    }
    exit_with_errno!(ctx, Ebadf);
}

// TODO: how are return types handled? Right now exit_with_errno etc return u32.
//       wasi_seek, and wasi_tell should return FileSize (u64). Just change type?
//       What about types that return (), etc.

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_seek(ctx: &mut VmCtx, v_fd: u32, v_filedelta: i64, v_whence: Whence) -> u32 {
    if v_fd >= MAX_SBOX_FDS {
        exit_with_errno!(ctx, Ebadf);
    }

    if let Ok(fd) = ctx.fdmap.m[v_fd as usize] {
        let ret = os_seek(fd, v_filedelta, v_whence.into());
        if let Some(errno) = RuntimeError::from_syscall_ret(ret) {
            exit_with_errno!(ctx, errno);
        }

        return ret as u32;
    }
    exit_with_errno!(ctx, Ebadf);
}

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_tell(ctx: &mut VmCtx, v_fd: u32) -> u32 {
    wasi_seek(ctx, v_fd, 0, Whence::Cur)
}

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_sync(ctx: &mut VmCtx, v_fd: u32) -> u32 {
    if v_fd >= MAX_SBOX_FDS {
        exit_with_errno!(ctx, Ebadf);
    }

    if let Ok(fd) = ctx.fdmap.m[v_fd as usize] {
        let ret = os_sync(fd);
        if let Some(errno) = RuntimeError::from_syscall_ret(ret) {
            exit_with_errno!(ctx, errno);
        }

        return ret as u32;
    }
    exit_with_errno!(ctx, Ebadf);
}

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_clock_res_get(ctx: &mut VmCtx, id: ClockId) -> Timestamp {
    let mut spec = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };

    let ret = os_clock_get_res(id.into(), &mut spec);

    if let Some(errno) = RuntimeError::from_syscall_ret(ret) {
        // TODO: exit with errno for non u32
        ctx.errno = errno;
        return Timestamp::MAX;
    }

    // convert to ns
    (spec.tv_sec * 1_000_000_000 + spec.tv_nsec) as Timestamp
}

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_clock_time_get(ctx: &mut VmCtx, id: ClockId, precision: Timestamp) -> Timestamp {
    // TODO: should inval clock be handled in higher level, or have Unkown ClockId variant
    //       and handle here?
    // TODO: how to handle `precision` arg? Looks like some runtimes ignore it...
    let mut spec = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };

    let ret = os_clock_get_time(id.into(), &mut spec);

    if let Some(errno) = RuntimeError::from_syscall_ret(ret) {
        // TODO: exit with errno for non u32
        ctx.errno = errno;
        return Timestamp::MAX;
    }

    (spec.tv_sec * 1_000_000_000 + spec.tv_nsec) as Timestamp
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Instant;

    // some basic sanity tests
    #[test]
    fn test_time_get() {
        let mut ctx = fresh_ctx(String::from("."));
        let ret = wasi_clock_time_get(&mut ctx, ClockId::Realtime, 0);
        let reference = Instant::now();

        assert_ne!(ret, 0);
        assert_eq!(ctx.errno, RuntimeError::Success);
    }

    #[test]
    fn test_res_get() {
        let mut ctx = fresh_ctx(String::from("."));
        let ret = wasi_clock_res_get(&mut ctx, ClockId::Realtime);
        let reference = Instant::now();

        assert_ne!(ret, 0);
        assert_eq!(ctx.errno, RuntimeError::Success);
    }
}
