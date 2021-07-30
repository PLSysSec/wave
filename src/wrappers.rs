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

fn is_syscall_error(val: u32) -> bool {
    // syscall returns between -1 and -4095 are errors, source:
    // https://code.woboq.org/userspace/glibc/sysdeps/unix/sysv/linux/x86_64/sysdep.h.html#369
    val >= -4095i32 as u32
}

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_path_open(ctx: &mut VmCtx, pathname: u32, flags: i32) -> u32 {
    if !ctx.fits_in_lin_mem(pathname, PATH_MAX) {
        exit_with_errno!(ctx, Efault);
    }

    let host_buffer = ctx.copy_buf_from_sandbox(pathname, PATH_MAX);
    let host_pathname = ctx.resolve_path(host_buffer);
    let fd = os_open(host_pathname, flags);
    let sbox_fd = ctx.fdmap.create(fd.into());
    if let Ok(s_fd) = sbox_fd {
        return s_fd;
    }
    exit_with_errno!(ctx, Ebadf);
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

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_seek(ctx: &mut VmCtx, v_fd: u32, v_filedelta: i64, v_whence: Whence) -> u32 {
    if v_fd >= MAX_SBOX_FDS {
        exit_with_errno!(ctx, Ebadf);
    }

    if let Ok(fd) = ctx.fdmap.m[v_fd as usize] {
        let ret = os_seek(fd, v_filedelta, v_whence.into()) as u32;
        if is_syscall_error(ret) {
            let errno = ret.into();
            exit_with_errno!(ctx, errno);
        } else {
            return ret;
        }
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
        let ret = os_sync(fd) as u32;
        if is_syscall_error(ret) {
            let errno = ret.into();
            exit_with_errno!(ctx, errno);
        } else {
            return ret;
        }
    }
    exit_with_errno!(ctx, Ebadf);
}
