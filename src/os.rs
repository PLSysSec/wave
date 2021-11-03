use crate::tcb::os_specs::linux::*;
#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::types::*;
use crate::{effect, four_effects, no_effect, one_effect, three_effects, two_effects};
use extra_args::{external_call, external_method, with_ghost_var};
use prusti_contracts::*;
use syscall::syscall;

/// This module contains our syscall specifications
/// These functions must be trusted because we don't know what the os actually does
/// on a syscall

#[with_ghost_var(trace: &mut Trace)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::PathAccess))]
pub fn trace_open(ctx: &VmCtx, pathname: SandboxedPath, flags: i32) -> usize {
    let os_path: Vec<u8> = pathname.into();
    os_open(os_path, flags)
}

#[with_ghost_var(trace: &mut Trace)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_close(ctx: &VmCtx, fd: HostFd) -> usize {
    let os_fd: usize = fd.into();
    os_close(os_fd)
}

#[with_ghost_var(trace: &mut Trace)]
#[external_method(into)]
#[requires(buf.len() >= cnt)]
#[requires(cnt < ctx.memlen)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(buf.len() >= cnt)]
#[ensures(result <= cnt)]
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::WriteN(count)))]
/// read writes `cnt` bytes to sandbox memory
pub fn trace_read(ctx: &VmCtx, fd: HostFd, buf: &mut [u8], cnt: usize) -> usize {
    let os_fd: usize = fd.into();
    os_read(os_fd, buf, cnt)
}

#[with_ghost_var(trace: &mut Trace)]
#[external_method(into)]
#[requires(buf.capacity() >= cnt)]
#[requires(cnt < ctx.memlen)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(buf.len() == result)]
#[ensures(buf.capacity() >= cnt)]
/// pread writes `cnt` bytes to sandbox memory
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::WriteN(count)))]
pub fn trace_pread(ctx: &VmCtx, fd: HostFd, buf: &mut Vec<u8>, cnt: usize) -> usize {
    let os_fd: usize = fd.into();
    os_pread(os_fd, buf, cnt)
}

#[with_ghost_var(trace: &mut Trace)]
#[external_method(into)]
#[requires(buf.len() >= cnt)]
#[requires(trace_safe(ctx, trace))]
#[requires(cnt < ctx.memlen)]
#[ensures(trace_safe(ctx, trace))]
// write reads `cnt` bytes to the sandbox
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::ReadN(count)))]
pub fn trace_write(ctx: &VmCtx, fd: HostFd, buf: &[u8], cnt: usize) -> usize {
    let os_fd: usize = fd.into();
    os_write(os_fd, buf, cnt)
}

#[with_ghost_var(trace: &mut Trace)]
#[external_method(into)]
#[requires(buf.len() >= cnt)]
#[requires(cnt < ctx.memlen)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
// pwrite writes `cnt` bytes to the sandbox
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::ReadN(count)))]
pub fn trace_pwrite(ctx: &VmCtx, fd: HostFd, buf: &Vec<u8>, cnt: usize) -> usize {
    let os_fd: usize = fd.into();
    os_pwrite(os_fd, buf, cnt)
}

#[with_ghost_var(trace: &mut Trace)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_seek(ctx: &VmCtx, fd: HostFd, offset: i64, whence: i32) -> usize {
    let os_fd: usize = fd.into();
    os_seek(os_fd, offset, whence)
}

#[with_ghost_var(trace: &mut Trace)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_advise(ctx: &VmCtx, fd: HostFd, offset: i64, len: i64, advice: i32) -> usize {
    let os_fd: usize = fd.into();
    os_advise(os_fd, offset, len, advice)
}

#[with_ghost_var(trace: &mut Trace)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_allocate(ctx: &VmCtx, fd: HostFd, offset: i64, len: i64) -> usize {
    let os_fd: usize = fd.into();
    os_allocate(os_fd, offset, len)
}

#[with_ghost_var(trace: &mut Trace)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_sync(ctx: &VmCtx, fd: HostFd) -> usize {
    let os_fd: usize = fd.into();
    os_sync(os_fd)
}

#[with_ghost_var(trace: &mut Trace)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_datasync(ctx: &VmCtx, fd: HostFd) -> usize {
    let os_fd: usize = fd.into();
    os_datasync(os_fd)
}

#[with_ghost_var(trace: &mut Trace)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_fstat(ctx: &VmCtx, fd: HostFd, stat: &mut libc::stat) -> usize {
    let os_fd: usize = fd.into();
    os_fstat(os_fd, stat)
}

#[with_ghost_var(trace: &mut Trace)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::PathAccess))]
pub fn trace_fstatat(
    ctx: &VmCtx,
    fd: HostFd,
    path: SandboxedPath,
    stat: &mut libc::stat,
    flags: i32,
) -> usize {
    let os_fd: usize = fd.into();
    let os_path: Vec<u8> = path.into();
    os_fstatat(os_fd, os_path, stat, flags)
}

#[with_ghost_var(trace: &mut Trace)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_fgetfl(ctx: &VmCtx, fd: HostFd) -> usize {
    let os_fd: usize = fd.into();
    os_fgetfl(os_fd)
}

#[with_ghost_var(trace: &mut Trace)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_fsetfl(ctx: &VmCtx, fd: HostFd, flags: libc::c_int) -> usize {
    let os_fd: usize = fd.into();
    os_fsetfl(os_fd, flags)
}

#[with_ghost_var(trace: &mut Trace)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_ftruncate(ctx: &VmCtx, fd: HostFd, length: libc::off_t) -> usize {
    let os_fd: usize = fd.into();
    os_ftruncate(os_fd, length)
}

#[with_ghost_var(trace: &mut Trace)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(four_effects!(old(trace), trace, Effect::FdAccess, Effect::FdAccess, Effect::PathAccess, Effect::PathAccess))]
pub fn trace_linkat(
    ctx: &VmCtx,
    old_fd: HostFd,
    old_path: SandboxedPath,
    new_fd: HostFd,
    new_path: SandboxedPath,
    flags: i32,
) -> usize {
    let os_old_fd: usize = old_fd.into();
    let os_new_fd: usize = new_fd.into();
    let os_old_path: Vec<u8> = old_path.into();
    let os_new_path: Vec<u8> = new_path.into();
    os_linkat(os_old_fd, os_old_path, os_new_fd, os_new_path, flags)
}

#[with_ghost_var(trace: &mut Trace)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::PathAccess))]
pub fn trace_mkdirat(
    ctx: &VmCtx,
    dir_fd: HostFd,
    pathname: SandboxedPath,
    mode: libc::mode_t,
) -> usize {
    let os_fd: usize = dir_fd.into();
    let os_path: Vec<u8> = pathname.into();
    os_mkdirat(os_fd, os_path, mode)
}

#[with_ghost_var(trace: &mut Trace)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[requires(cnt < ctx.memlen)]
#[requires(buf.capacity() >= cnt)]
#[ensures(buf.len() == result)]
#[ensures(buf.capacity() >= cnt)]
#[ensures(trace_safe(ctx, trace))]
#[ensures(three_effects!(old(trace), trace, Effect::FdAccess, Effect::PathAccess, Effect::WriteN(count)))]
pub fn trace_readlinkat(
    ctx: &VmCtx,
    dir_fd: HostFd,
    pathname: SandboxedPath,
    buf: &mut Vec<u8>,
    cnt: usize,
) -> usize {
    let os_fd: usize = dir_fd.into();
    let os_path: Vec<u8> = pathname.into();
    os_readlinkat(os_fd, os_path, buf, cnt)
}

#[with_ghost_var(trace: &mut Trace)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::PathAccess))]
pub fn trace_unlinkat(
    ctx: &VmCtx,
    dir_fd: HostFd,
    pathname: SandboxedPath,
    flags: libc::c_int,
) -> usize {
    let os_fd: usize = dir_fd.into();
    let os_path: Vec<u8> = pathname.into();
    os_unlinkat(os_fd, os_path, flags)
}

#[with_ghost_var(trace: &mut Trace)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(four_effects!(old(trace), trace, Effect::FdAccess, Effect::PathAccess, Effect::FdAccess, Effect::PathAccess))]
pub fn trace_renameat(
    ctx: &VmCtx,
    old_dir_fd: HostFd,
    old_pathname: SandboxedPath,
    new_dir_fd: HostFd,
    new_pathname: SandboxedPath,
) -> usize {
    let os_old_fd: usize = old_dir_fd.into();
    let os_old_path: Vec<u8> = old_pathname.into();
    let os_new_fd: usize = new_dir_fd.into();
    let os_new_path: Vec<u8> = new_pathname.into();
    os_renameat(os_old_fd, os_old_path, os_new_fd, os_new_path)
}

#[with_ghost_var(trace: &mut Trace)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(three_effects!(old(trace), trace,  Effect::PathAccess, Effect::FdAccess, Effect::PathAccess))]
pub fn trace_symlinkat(
    ctx: &VmCtx,
    old_pathname: SandboxedPath,
    dir_fd: HostFd,
    new_pathname: SandboxedPath,
) -> usize {
    let os_fd: usize = dir_fd.into();
    let os_old_path: Vec<u8> = old_pathname.into();
    let os_new_path: Vec<u8> = new_pathname.into();
    os_symlinkat(os_old_path, os_fd, os_new_path)
}

#[with_ghost_var(trace: &mut Trace)]
#[external_method(into)]
#[requires(specs.capacity() >= 2)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_futimens(ctx: &VmCtx, fd: HostFd, specs: &Vec<libc::timespec>) -> usize {
    let os_fd: usize = fd.into();
    os_futimens(os_fd, specs)
}

#[with_ghost_var(trace: &mut Trace)]
#[external_method(into)]
#[requires(specs.capacity() >= 2)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::PathAccess))]
pub fn trace_utimensat(
    ctx: &VmCtx,
    fd: HostFd,
    pathname: SandboxedPath,
    specs: &Vec<libc::timespec>,
    flags: libc::c_int,
) -> usize {
    let os_fd: usize = fd.into();
    let os_path: Vec<u8> = pathname.into();
    os_utimensat(os_fd, os_path, specs, flags)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(no_effect!(old(trace), trace))]
pub fn trace_clock_get_time(
    ctx: &VmCtx,
    clock_id: libc::clockid_t,
    spec: &mut libc::timespec,
) -> usize {
    os_clock_get_time(clock_id, spec)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(no_effect!(old(trace), trace))]
pub fn trace_clock_get_res(
    ctx: &VmCtx,
    clock_id: libc::clockid_t,
    spec: &mut libc::timespec,
) -> usize {
    os_clock_get_res(clock_id, spec)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(ctx, trace))]
#[requires(buf.capacity() >= cnt)]
#[requires(cnt < ctx.memlen)]
#[ensures(buf.len() == result)]
#[ensures(buf.capacity() >= cnt)]
#[ensures(one_effect!(old(trace), trace, Effect::WriteN(count)))]
#[ensures(trace_safe(ctx, trace))]
pub fn trace_getrandom(ctx: &VmCtx, buf: &mut Vec<u8>, cnt: usize, flags: u32) -> usize {
    os_getrandom(buf, cnt, flags)
}

//https://man7.org/linux/man-pages/man2/getrandom.2.html
#[with_ghost_var(trace: &mut Trace)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[requires(cnt < ctx.memlen)]
#[ensures(trace_safe(ctx, trace))]
#[requires(buf.capacity() >= cnt)]
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::WriteN(count)))]
#[ensures(buf.len() == result)]
#[ensures(buf.capacity() >= cnt)]
pub fn trace_recv(ctx: &VmCtx, fd: HostFd, buf: &mut Vec<u8>, cnt: usize, flags: u32) -> usize {
    let os_fd: usize = fd.into();
    os_recv(os_fd, buf, cnt, flags)
}

#[with_ghost_var(trace: &mut Trace)]
#[external_method(into)]
#[requires(buf.len() >= cnt)]
#[requires(cnt < ctx.memlen)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::ReadN(count)))]
pub fn trace_send(ctx: &VmCtx, fd: HostFd, buf: &Vec<u8>, cnt: usize, flags: u32) -> usize {
    let os_fd: usize = fd.into();
    os_send(os_fd, buf, cnt, flags)
}

#[with_ghost_var(trace: &mut Trace)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(two_effects!(old(trace), trace, Effect::Shutdown, Effect::FdAccess))]
pub fn trace_shutdown(ctx: &VmCtx, fd: HostFd, how: libc::c_int) -> usize {
    let os_fd: usize = fd.into();
    os_shutdown(os_fd, how)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(no_effect!(old(trace), trace))]
pub fn trace_nanosleep(ctx: &VmCtx, req: &libc::timespec, rem: &mut libc::timespec) -> usize {
    os_nanosleep(req, rem)
}

//TODO: not sure what the spec for this is yet.
#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(no_effect!(old(trace), trace))]
pub fn trace_poll(ctx: &VmCtx, pollfd: &mut libc::pollfd, timeout: libc::c_int) -> usize {
    os_poll(pollfd, timeout)
}
