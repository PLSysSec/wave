use crate::tcb::os_specs::linux::*;
#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::types::*;
use crate::{effect, four_effects, no_effect, one_effect, three_effects, two_effects};
use extra_args::with_ghost_var;
use prusti_contracts::*;
use syscall::syscall;

/// This module contains our syscall specifications
/// These functions must be trusted because we don't know what the os actually does
/// on a syscall

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(one_effect!(old(trace), trace, effect!(PathAccess)))]
pub fn trace_openat(
    ctx: &VmCtx,
    dir_fd: HostFd,
    pathname: SandboxedPath,
    flags: i32,
) -> RuntimeResult<usize> {
    let os_fd: usize = dir_fd.into();
    let os_path: Vec<u8> = pathname.into();
    let r = os_openat(os_fd, os_path, flags);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
pub fn trace_close(ctx: &VmCtx, fd: HostFd) -> RuntimeResult<usize> {
    let os_fd: usize = fd.into();
    let r = os_close(os_fd);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx.fits_in_lin_mem(ptr, cnt as u32, trace))]
#[requires(cnt < ctx.memlen)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(WriteN, addr, count)))]
/// read writes `cnt` bytes to sandbox memory
pub fn trace_read(ctx: &mut VmCtx, fd: HostFd, ptr: SboxPtr, cnt: usize) -> RuntimeResult<usize> {
    let slice = ctx.slice_mem_mut(ptr, cnt as u32);
    let os_fd: usize = fd.into();
    let r = os_read(os_fd, slice, cnt);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx.fits_in_lin_mem(ptr, cnt as u32, trace))]
//#[requires(buf.capacity() >= cnt)]
#[requires(cnt < ctx.memlen)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
//#[ensures(buf.len() == result)]
//#[ensures(buf.capacity() >= cnt)]
/// pread writes `cnt` bytes to sandbox memory
#[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(WriteN, addr, count)))]
pub fn trace_pread(
    ctx: &mut VmCtx,
    fd: HostFd,
    ptr: SboxPtr,
    cnt: usize,
    offset: usize,
) -> RuntimeResult<usize> {
    let slice = ctx.slice_mem_mut(ptr, cnt as u32);
    let os_fd: usize = fd.into();
    let r = os_pread(os_fd, slice, cnt, offset);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx.fits_in_lin_mem(ptr, cnt as u32, trace))]
#[requires(cnt < ctx.memlen)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
// write reads `cnt` bytes to the sandbox
#[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(ReadN, addr, count)))]
pub fn trace_write(ctx: &mut VmCtx, fd: HostFd, ptr: SboxPtr, cnt: usize) -> RuntimeResult<usize> {
    let slice = ctx.slice_mem_mut(ptr, cnt as u32);
    let os_fd: usize = fd.into();
    let r = os_write(os_fd, slice, cnt);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx.fits_in_lin_mem(ptr, cnt as u32, trace))]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[requires(cnt < ctx.memlen)]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
// pwrite writes `cnt` bytes to the sandbox
#[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(ReadN, addr, count)))]
pub fn trace_pwrite(
    ctx: &mut VmCtx,
    fd: HostFd,
    ptr: SboxPtr,
    cnt: usize,
    offset: usize,
) -> RuntimeResult<usize> {
    let slice = ctx.slice_mem_mut(ptr, cnt as u32);
    let os_fd: usize = fd.into();
    let r = os_pwrite(os_fd, slice, cnt, offset);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
pub fn trace_seek(ctx: &VmCtx, fd: HostFd, offset: i64, whence: i32) -> RuntimeResult<usize> {
    let os_fd: usize = fd.into();
    let r = os_seek(os_fd, offset, whence);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
pub fn trace_advise(
    ctx: &VmCtx,
    fd: HostFd,
    offset: i64,
    len: i64,
    advice: i32,
) -> RuntimeResult<usize> {
    let os_fd: usize = fd.into();
    let r = os_advise(os_fd, offset, len, advice);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
pub fn trace_allocate(ctx: &VmCtx, fd: HostFd, offset: i64, len: i64) -> RuntimeResult<usize> {
    let os_fd: usize = fd.into();
    let r = os_allocate(os_fd, offset, len);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
pub fn trace_sync(ctx: &VmCtx, fd: HostFd) -> RuntimeResult<usize> {
    let os_fd: usize = fd.into();
    let r = os_sync(os_fd);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
pub fn trace_datasync(ctx: &VmCtx, fd: HostFd) -> RuntimeResult<usize> {
    let os_fd: usize = fd.into();
    let r = os_datasync(os_fd);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
pub fn trace_fstat(ctx: &VmCtx, fd: HostFd, stat: &mut libc::stat) -> RuntimeResult<usize> {
    let os_fd: usize = fd.into();
    let r = os_fstat(os_fd, stat);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccess)))]
pub fn trace_fstatat(
    ctx: &VmCtx,
    fd: HostFd,
    path: SandboxedPath,
    stat: &mut libc::stat,
    flags: i32,
) -> RuntimeResult<usize> {
    let os_fd: usize = fd.into();
    let os_path: Vec<u8> = path.into();
    let r = os_fstatat(os_fd, os_path, stat, flags);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
pub fn trace_fgetfl(ctx: &VmCtx, fd: HostFd) -> RuntimeResult<usize> {
    let os_fd: usize = fd.into();
    let r = os_fgetfl(os_fd);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
pub fn trace_fsetfl(ctx: &VmCtx, fd: HostFd, flags: libc::c_int) -> RuntimeResult<usize> {
    let os_fd: usize = fd.into();
    let r = os_fsetfl(os_fd, flags);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
pub fn trace_ftruncate(ctx: &VmCtx, fd: HostFd, length: libc::off_t) -> RuntimeResult<usize> {
    let os_fd: usize = fd.into();
    let r = os_ftruncate(os_fd, length);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(four_effects!(old(trace), trace, effect!(FdAccess), effect!(FdAccess), effect!(PathAccess), effect!(PathAccess)))]
pub fn trace_linkat(
    ctx: &VmCtx,
    old_fd: HostFd,
    old_path: SandboxedPath,
    new_fd: HostFd,
    new_path: SandboxedPath,
    flags: i32,
) -> RuntimeResult<usize> {
    let os_old_fd: usize = old_fd.into();
    let os_new_fd: usize = new_fd.into();
    let os_old_path: Vec<u8> = old_path.into();
    let os_new_path: Vec<u8> = new_path.into();
    let r = os_linkat(os_old_fd, os_old_path, os_new_fd, os_new_path, flags);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccess)))]
pub fn trace_mkdirat(
    ctx: &VmCtx,
    dir_fd: HostFd,
    pathname: SandboxedPath,
    mode: libc::mode_t,
) -> RuntimeResult<usize> {
    let os_fd: usize = dir_fd.into();
    let os_path: Vec<u8> = pathname.into();
    let r = os_mkdirat(os_fd, os_path, mode);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx.fits_in_lin_mem(ptr, cnt as u32, trace))]
#[requires(cnt < ctx.memlen)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(three_effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccess), effect!(WriteN, addr, count)))]
pub fn trace_readlinkat(
    ctx: &mut VmCtx,
    dir_fd: HostFd,
    pathname: SandboxedPath,
    ptr: SboxPtr,
    cnt: usize,
) -> RuntimeResult<usize> {
    let slice = ctx.slice_mem_mut(ptr, cnt as u32);
    let os_fd: usize = dir_fd.into();
    let os_path: Vec<u8> = pathname.into();
    let r = os_readlinkat(os_fd, os_path, slice, cnt);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccess)))]
pub fn trace_unlinkat(
    ctx: &VmCtx,
    dir_fd: HostFd,
    pathname: SandboxedPath,
    flags: libc::c_int,
) -> RuntimeResult<usize> {
    let os_fd: usize = dir_fd.into();
    let os_path: Vec<u8> = pathname.into();
    let r = os_unlinkat(os_fd, os_path, flags);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(four_effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccess), effect!(FdAccess), effect!(PathAccess)))]
pub fn trace_renameat(
    ctx: &VmCtx,
    old_dir_fd: HostFd,
    old_pathname: SandboxedPath,
    new_dir_fd: HostFd,
    new_pathname: SandboxedPath,
) -> RuntimeResult<usize> {
    let os_old_fd: usize = old_dir_fd.into();
    let os_old_path: Vec<u8> = old_pathname.into();
    let os_new_fd: usize = new_dir_fd.into();
    let os_new_path: Vec<u8> = new_pathname.into();
    let r = os_renameat(os_old_fd, os_old_path, os_new_fd, os_new_path);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(three_effects!(old(trace), trace,  effect!(PathAccess), effect!(FdAccess), effect!(PathAccess)))]
pub fn trace_symlinkat(
    ctx: &VmCtx,
    old_pathname: SandboxedPath,
    dir_fd: HostFd,
    new_pathname: SandboxedPath,
) -> RuntimeResult<usize> {
    let os_fd: usize = dir_fd.into();
    let os_old_path: Vec<u8> = old_pathname.into();
    let os_new_path: Vec<u8> = new_pathname.into();
    let r = os_symlinkat(os_old_path, os_fd, os_new_path);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(specs.capacity() >= 2)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
pub fn trace_futimens(
    ctx: &VmCtx,
    fd: HostFd,
    specs: &Vec<libc::timespec>,
) -> RuntimeResult<usize> {
    let os_fd: usize = fd.into();
    let r = os_futimens(os_fd, specs);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(specs.capacity() >= 2)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccess)))]
pub fn trace_utimensat(
    ctx: &VmCtx,
    fd: HostFd,
    pathname: SandboxedPath,
    specs: &Vec<libc::timespec>,
    flags: libc::c_int,
) -> RuntimeResult<usize> {
    let os_fd: usize = fd.into();
    let os_path: Vec<u8> = pathname.into();
    let r = os_utimensat(os_fd, os_path, specs, flags);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(no_effect!(old(trace), trace))]
pub fn trace_clock_get_time(
    ctx: &VmCtx,
    clock_id: libc::clockid_t,
    spec: &mut libc::timespec,
) -> RuntimeResult<usize> {
    let r = os_clock_get_time(clock_id, spec);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(no_effect!(old(trace), trace))]
pub fn trace_clock_get_res(
    ctx: &VmCtx,
    clock_id: libc::clockid_t,
    spec: &mut libc::timespec,
) -> RuntimeResult<usize> {
    let r = os_clock_get_res(clock_id, spec);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx.fits_in_lin_mem(ptr, cnt as u32, trace))]
#[requires(cnt < ctx.memlen)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(one_effect!(old(trace), trace, effect!(WriteN, addr, count)))]
pub fn trace_getrandom(
    ctx: &mut VmCtx,
    ptr: SboxPtr,
    cnt: usize,
    flags: u32,
) -> RuntimeResult<usize> {
    let slice = ctx.slice_mem_mut(ptr, cnt as u32);
    let r = os_getrandom(slice, cnt, flags);
    RuntimeError::from_syscall_ret(r)
}

//https://man7.org/linux/man-pages/man2/getrandom.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx.fits_in_lin_mem(ptr, cnt as u32, trace))]
#[requires(cnt < ctx.memlen)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(WriteN, addr, count)))]
pub fn trace_recv(
    ctx: &mut VmCtx,
    fd: HostFd,
    ptr: SboxPtr,
    cnt: usize,
    flags: u32,
) -> RuntimeResult<usize> {
    let slice = ctx.slice_mem_mut(ptr, cnt as u32);
    let os_fd: usize = fd.into();
    let r = os_recv(os_fd, slice, cnt, flags);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx.fits_in_lin_mem(ptr, cnt as u32, trace))]
#[requires(cnt < ctx.memlen)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(ReadN, addr, count)))]
pub fn trace_send(
    ctx: &mut VmCtx,
    fd: HostFd,
    ptr: SboxPtr,
    cnt: usize,
    flags: u32,
) -> RuntimeResult<usize> {
    let slice = ctx.slice_mem_mut(ptr, cnt as u32);
    let os_fd: usize = fd.into();
    let r = os_send(os_fd, slice, cnt, flags);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(two_effects!(old(trace), trace, effect!(Shutdown), effect!(FdAccess)))]
pub fn trace_shutdown(ctx: &VmCtx, fd: HostFd, how: libc::c_int) -> RuntimeResult<usize> {
    let os_fd: usize = fd.into();
    let r = os_shutdown(os_fd, how);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(no_effect!(old(trace), trace))]
pub fn trace_nanosleep(
    ctx: &VmCtx,
    req: &libc::timespec,
    rem: &mut libc::timespec,
) -> RuntimeResult<usize> {
    let r = os_nanosleep(req, rem);
    RuntimeError::from_syscall_ret(r)
}

//TODO: not sure what the spec for this is yet.
#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(no_effect!(old(trace), trace))]
pub fn trace_poll(
    ctx: &VmCtx,
    pollfd: &mut libc::pollfd,
    timeout: libc::c_int,
) -> RuntimeResult<usize> {
    let r = os_poll(pollfd, timeout);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(dirp.capacity() >= count)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
// pub fn os_getdents64(fd: usize, dirp: &mut libc::dirent, count: usize) -> usize {
//buf: &mut Vec<u8>
pub fn trace_getdents64(
    ctx: &VmCtx,
    fd: HostFd,
    dirp: &mut Vec<u8>,
    count: usize,
) -> RuntimeResult<usize> {
    let os_fd: usize = fd.into();
    let r = os_getdents64(os_fd, dirp, count);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(no_effect!(old(trace), trace))]
pub fn trace_socket(ctx: &VmCtx, domain: i32, ty: i32, protocol: i32) -> RuntimeResult<usize> {
    let r = os_socket(domain, ty, protocol);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
// #[requires(addr.sin_addr.s_addr addr.sin_port)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(NetAccess, protocol, ip, port)))]
pub fn trace_connect(
    ctx: &VmCtx,
    sockfd: HostFd,
    addr: &libc::sockaddr_in,
    addrlen: u32,
) -> RuntimeResult<usize> {
    let os_fd: usize = sockfd.into();
    let r = os_connect(os_fd, addr, addrlen);
    RuntimeError::from_syscall_ret(r)
}
