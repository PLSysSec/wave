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
pub fn wasi_fd_close(ctx: &mut VmCtx, v_fd: u32) -> u32 {
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
