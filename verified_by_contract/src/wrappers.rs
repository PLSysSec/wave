use crate::external_specs::result::*;
use crate::os::*;
use crate::runtime::*;
use crate::types::*;
use prusti_contracts::*;
use std::convert::TryInto;
use RuntimeError::*;

predicate! {
    fn safe(ctx: &VmCtx) -> bool {
        true
    }
}

// predicate SFISafe(ctx) =
// not exists. a. a < ctx.membase | a >= ctx.membase + ctx.memlength. access(a)

// predicate FdSafe(ctx) =
// not exists. fd. inRevFdMap(ctx, fd) & os_read_fd(fd)

// WASIRead(ctx): ... write at most v_cnt bytes etc.

// validCtx(ctx), SFISafe(ctx), FdSafe(ctx) = ...

//pre: {..., }
//post: {..., inFDMap(ctx, fd), inRevFDMap(ctx, translate_fd(fd) )}
// #[trusted]
// #[requires(safe(ctx))]
// #[ensures(safe(ctx))]
// pub fn wasi_open(ctx: &mut VmCtx, pathname: SboxPtr, flags: i32) -> isize {
//     let host_buffer_opt = ctx.copy_buf_from_sandbox(pathname, PATH_MAX);
//     if host_buffer_opt.is_none(){
//       return -1;
//     }
//     let host_buffer = host_buffer_opt.unwrap();

//     let host_pathname = ctx.resolve_path(host_buffer);
//     let fd = os_open(host_pathname as *mut u8, flags);
//     let sbox_fd = ctx.fdmap.create(fd, ctx);
//     return sbox_fd as isize;
// }

#[trusted]
#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_close(ctx: &mut VmCtx, v_fd: i32) -> i32 {
    if (v_fd < 0) || (v_fd >= MAX_SBOX_FDS_I32) {
        ctx.errno = Ebadf;
        return -1;
    }
    let sbox_fd: SboxFd = v_fd as SboxFd;
    if let Ok(fd) = ctx.fdmap.lookup(sbox_fd) {
        ctx.fdmap.delete(sbox_fd);
        return os_close(fd);
    }
    ctx.errno = Ebadf;
    return -1;
}

#[trusted]
#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_read(ctx: &mut VmCtx, v_fd: i32, v_buf: SboxPtr, v_cnt: usize) -> isize {
    if !ctx.fits_in_lin_mem(v_buf, v_cnt) {
        ctx.errno = Efault;
        return -1;
    }
    if (v_fd < 0) || (v_fd >= MAX_SBOX_FDS_I32) {
        ctx.errno = Ebadf;
        return -1;
    }
    let sbox_fd: SboxFd = v_fd as SboxFd;
    if let Ok(fd) = ctx.fdmap.lookup(sbox_fd) {
        os_read(fd, ctx.mem[v_buf] as *mut u8, v_cnt);
    }
    ctx.errno = Ebadf;
    return -1;
}

#[trusted]
#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_write(ctx: &mut VmCtx, v_fd: i32, v_buf: SboxPtr, v_cnt: usize) -> isize {
    if !ctx.fits_in_lin_mem(v_buf, v_cnt) {
        ctx.errno = Efault;
        return -1;
    }
    if (v_fd < 0) || (v_fd >= MAX_SBOX_FDS_I32) {
        ctx.errno = Ebadf;
        return -1;
    }
    let sbox_fd: SboxFd = v_fd as SboxFd;
    if let Ok(fd) = ctx.fdmap.lookup(sbox_fd) {
        return os_write(fd, ctx.mem[v_buf] as *mut u8, v_cnt);
    }
    ctx.errno = Ebadf;
    return -1;
}
