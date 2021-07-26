#[cfg(feature = "verify")]
use crate::external_specs::result::*;
use crate::os::*;
use crate::runtime::*;
use crate::types::*;
use prusti_contracts::*;
use std::convert::TryInto;
use RuntimeError::*;

#[cfg(feature = "verify")]
predicate! {
    fn safe(ctx: &VmCtx) -> bool {
        true
    }
}

macro_rules! exit_with_errno {
    ($ctx:ident, $errno:ident) => {
        $ctx.errno = $errno;
        return -1
    };
}

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_open(ctx: &mut VmCtx, pathname: SboxPtr, flags: i32) -> isize {
    let host_buffer_opt = ctx.copy_buf_from_sandbox(pathname, PATH_MAX);
    if host_buffer_opt.is_none(){
      exit_with_errno!(ctx, Efault);
    }
    let host_buffer = host_buffer_opt.unwrap();
    let mut host_pathname = ctx.resolve_path(host_buffer);
    let fd = os_open(ctx, &mut host_pathname, flags); 
    let sbox_fd = ctx.fdmap.create(fd as usize);
    if let Ok(s_fd) = sbox_fd{
        return s_fd as isize;
    }
    exit_with_errno!(ctx, Ebadf);
}

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_close(ctx: &mut VmCtx, v_fd: i32) -> i32 {
    if (v_fd < 0) || (v_fd >= MAX_SBOX_FDS_I32) {
        exit_with_errno!(ctx, Ebadf);
    }
    let sbox_fd = v_fd as SboxFd;
    if let Ok(fd) = ctx.fdmap.m[sbox_fd] { 
        ctx.fdmap.delete(sbox_fd);
        return os_close(ctx, fd);
    }
    exit_with_errno!(ctx, Ebadf);
}

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_read(ctx: &mut VmCtx, v_fd: i32, v_buf: SboxPtr, v_cnt: usize) -> isize {
    if !ctx.fits_in_lin_mem(v_buf, v_cnt) {
        exit_with_errno!(ctx, Efault);
    }
    if (v_fd < 0) || (v_fd >= MAX_SBOX_FDS_I32) {
        exit_with_errno!(ctx, Ebadf);
    }
    let sbox_fd: SboxFd = v_fd as SboxFd;
    if let Ok(fd) = ctx.fdmap.m[sbox_fd] { 
        let mut buf: Vec<u8> = Vec::new();
        buf.reserve_exact(v_cnt);
        let result = os_read(ctx, fd, &mut buf, v_cnt);
        let copy_ok = ctx.copy_buf_to_sandbox(v_buf, &buf, v_cnt);
        if copy_ok.is_none(){
            exit_with_errno!(ctx, Efault);
        }
        return result;
    }
    exit_with_errno!(ctx, Ebadf);
}

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_write(ctx: &mut VmCtx, v_fd: i32, v_buf: SboxPtr, v_cnt: usize) -> isize {
    if !ctx.fits_in_lin_mem(v_buf, v_cnt) {
        exit_with_errno!(ctx, Efault);
    }
    if (v_fd < 0) || (v_fd >= MAX_SBOX_FDS_I32) {
        exit_with_errno!(ctx, Ebadf);
    }

    let host_buffer_opt = ctx.copy_buf_from_sandbox(v_buf, v_cnt);
    if host_buffer_opt.is_none(){
        exit_with_errno!(ctx, Efault);
    }
    let host_buffer = host_buffer_opt.unwrap();

    let sbox_fd: SboxFd = v_fd as SboxFd;
    if let Ok(fd) = ctx.fdmap.m[sbox_fd] {
        return os_write(ctx, fd, &host_buffer, v_cnt);
    }
    exit_with_errno!(ctx, Ebadf);
}