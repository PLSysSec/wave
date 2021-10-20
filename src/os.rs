#[cfg(feature = "verify")]
use crate::verifier::*;
use crate::{effect, no_effect, one_effect};
//use crate::trace::*;
use crate::types::*;
use extra_args::{external_call, external_method, with_ghost_var};
use prusti_contracts::*;
use syscall::syscall;

/// This module contains our syscall specifications
/// These functions must be trusted because we don't know what the os actually does
/// on a syscall

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_open)]
#[ensures(one_effect!(old(trace), trace, Effect::PathAccess))]
pub fn trace_open(pathname: SandboxedPath, flags: i32) -> usize {
    effect!(trace, Effect::PathAccess);
    os_open(pathname, flags)
}

#[trusted]
pub fn os_open(pathname: SandboxedPath, flags: i32) -> usize {
    let os_path: Vec<u8> = pathname.into();
    unsafe { syscall!(OPEN, os_path.as_ptr(), flags) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_close)]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_close(fd: HostFd) -> usize {
    effect!(trace, Effect::FdAccess);
    os_close(fd)
}

#[trusted]
pub fn os_close(fd: HostFd) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(CLOSE, os_fd) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_read)]
#[requires(buf.len() >= cnt)]
// #[requires(trace_safe(ctx, trace))]
// #[ensures(trace_safe(ctx, trace))]
#[ensures(buf.len() >= cnt)]
#[ensures(result <= cnt)]
#[ensures(one_effect!(old(trace), trace, Effect::ReadN(count) if count == cnt ))]
pub fn trace_read(ctx: &VmCtx, fd: HostFd, buf: &mut [u8], cnt: usize) -> usize {
    effect!(trace, Effect::ReadN(cnt));
    os_read(fd, buf, cnt)
}

#[requires(buf.len() >= cnt)]
#[ensures(buf.len() >= cnt)]
#[ensures(result <= cnt)]
#[trusted]
pub fn os_read(fd: HostFd, buf: &mut [u8], cnt: usize) -> usize {
    let os_fd: usize = fd.into();
    unsafe {
        let result = syscall!(READ, os_fd, buf.as_mut_ptr(), cnt);
        // TODO: this violates the safety requirements of set_len if result is an errno
        //       i.e. -4095 is probably > buf.capacity. Would need to also update
        //       post-conditions to reflect errno case.
        //       See: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.set_len
        result
    }
}

#[requires(buf.capacity() >= cnt)]
#[ensures(buf.len() == result)]
#[ensures(buf.capacity() >= cnt)]
#[trusted]
pub fn os_pread(fd: HostFd, buf: &mut Vec<u8>, cnt: usize) -> usize {
    let os_fd: usize = fd.into();
    unsafe {
        let result = syscall!(PREAD64, os_fd, buf.as_mut_ptr(), cnt);
        buf.set_len(result);
        result
    }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_write)]
#[requires(buf.len() >= cnt)]
//#[ensures(one_effect!(old(trace), trace, Effect::WriteN(count) if count == cnt ))]
pub fn trace_write(ctx: &VmCtx, fd: HostFd, buf: &[u8], cnt: usize) -> usize {
    effect!(trace, Effect::WriteN(cnt));
    os_write(fd, buf, cnt)
}

#[requires(buf.len() >= cnt)]
#[trusted]
pub fn os_write(fd: HostFd, buf: &[u8], cnt: usize) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(WRITE, os_fd, buf.as_ptr(), cnt) }
}

#[requires(buf.len() >= cnt)]
#[trusted]
pub fn os_pwrite(fd: HostFd, buf: &Vec<u8>, cnt: usize) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(WRITE, os_fd, buf.as_ptr(), cnt) }
}

// TODO: could be cleaner to do a typedef SyscallRet = usize or something for From traits
#[trusted]
pub fn os_seek(fd: HostFd, offset: i64, whence: i32) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(LSEEK, os_fd, offset, whence) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_seek)] // Do not add trace to os_seek
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_seek(ctx: &VmCtx, fd: HostFd, offset: i64, whence: i32) -> usize {
    effect!(trace, Effect::FdAccess);
    os_seek(fd, offset, whence)
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_advise)] // Do not add trace to os_advise
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_advise(ctx: &VmCtx, fd: HostFd, offset: i64, len: i64, advice: i32) -> usize {
    effect!(trace, Effect::FdAccess);
    os_advise(fd, offset, len, advice)
}

#[trusted]
pub fn os_advise(fd: HostFd, offset: i64, len: i64, advice: i32) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(FADVISE64, os_fd, offset, len, advice) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_allocate)] // Do not add trace to os_allocate
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_allocate(ctx: &VmCtx, fd: HostFd, offset: i64, len: i64) -> usize {
    effect!(trace, Effect::FdAccess);
    os_allocate(fd, offset, len)
}

#[trusted]
pub fn os_allocate(fd: HostFd, offset: i64, len: i64) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(FALLOCATE, os_fd, offset, len) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_sync)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_sync(ctx: &VmCtx, fd: HostFd) -> usize {
    effect!(trace, Effect::FdAccess);
    os_sync(fd)
}

#[trusted]
pub fn os_sync(fd: HostFd) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(FSYNC, os_fd) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_datasync)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_datasync(ctx: &VmCtx, fd: HostFd) -> usize {
    effect!(trace, Effect::FdAccess);
    os_datasync(fd)
}

#[trusted]
pub fn os_datasync(fd: HostFd) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(FDATASYNC, os_fd) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_fstat)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_fstat(ctx: &VmCtx, fd: HostFd, stat: &mut libc::stat) -> usize {
    effect!(trace, Effect::FdAccess);
    os_fstat(fd, stat)
}

#[trusted]
pub fn os_fstat(fd: HostFd, stat: &mut libc::stat) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(FSTAT, os_fd, stat as *mut libc::stat) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_fstatat)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_fstatat(
    ctx: &VmCtx,
    fd: HostFd,
    path: SandboxedPath,
    stat: &mut libc::stat,
    flags: i32,
) -> usize {
    effect!(trace, Effect::FdAccess);
    os_fstatat(fd, path, stat, flags)
}

#[trusted]
pub fn os_fstatat(fd: HostFd, path: SandboxedPath, stat: &mut libc::stat, flags: i32) -> usize {
    let os_fd: usize = fd.into();
    let os_path: Vec<u8> = path.into();
    unsafe {
        syscall!(
            NEWFSTATAT,
            os_fd,
            os_path.as_ptr(),
            stat as *mut libc::stat,
            flags
        )
    }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_fgetfl)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_fgetfl(ctx: &VmCtx, fd: HostFd) -> usize {
    effect!(trace, Effect::FdAccess);
    os_fgetfl(fd)
}

#[trusted]
pub fn os_fgetfl(fd: HostFd) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(FCNTL, os_fd, libc::F_GETFL, 0) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_fsetfl)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_fsetfl(ctx: &VmCtx, fd: HostFd, flags: libc::c_int) -> usize {
    effect!(trace, Effect::FdAccess);
    os_fsetfl(fd, flags)
}

#[trusted]
pub fn os_fsetfl(fd: HostFd, flags: libc::c_int) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(FCNTL, os_fd, libc::F_SETFL, flags) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_ftruncate)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_ftruncate(ctx: &VmCtx, fd: HostFd, length: libc::off_t) -> usize {
    effect!(trace, Effect::FdAccess);
    os_ftruncate(fd, length)
}

#[trusted]
pub fn os_ftruncate(fd: HostFd, length: libc::off_t) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(FTRUNCATE, os_fd, length) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_linkat)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::PathAccess))]
pub fn trace_linkat(
    ctx: &VmCtx,
    old_fd: HostFd,
    old_path: SandboxedPath,
    new_fd: HostFd,
    new_path: SandboxedPath,
    flags: i32,
) -> usize {
    effect!(trace, Effect::PathAccess);
    os_linkat(old_fd, old_path, new_fd, new_path, flags)
}

#[trusted]
pub fn os_linkat(
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
    unsafe {
        syscall!(
            LINKAT,
            os_old_fd,
            os_old_path.as_ptr(),
            os_new_fd,
            os_new_path.as_ptr(),
            flags
        )
    }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_mkdirat)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::PathAccess))]
pub fn trace_mkdirat(
    ctx: &VmCtx,
    dir_fd: HostFd,
    pathname: SandboxedPath,
    mode: libc::mode_t,
) -> usize {
    effect!(trace, Effect::PathAccess);
    os_mkdirat(dir_fd, pathname, mode)
}

#[trusted]
pub fn os_mkdirat(dir_fd: HostFd, pathname: SandboxedPath, mode: libc::mode_t) -> usize {
    let os_fd: usize = dir_fd.into();
    let os_path: Vec<u8> = pathname.into();
    unsafe { syscall!(MKDIRAT, os_fd, os_path.as_ptr(), mode) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_readlinkat)]
#[requires(trace_safe(ctx, trace))]
#[requires(buf.capacity() >= cnt)]
#[ensures(buf.len() == result)]
#[ensures(buf.capacity() >= cnt)]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::PathAccess))]
pub fn trace_readlinkat(
    ctx: &VmCtx,
    dir_fd: HostFd,
    pathname: SandboxedPath,
    buf: &mut Vec<u8>,
    cnt: usize,
) -> usize {
    effect!(trace, Effect::PathAccess);
    os_readlinkat(dir_fd, pathname, buf, cnt)
}

#[requires(buf.capacity() >= cnt)]
#[ensures(buf.len() == result)]
#[ensures(buf.capacity() >= cnt)]
#[trusted]
pub fn os_readlinkat(
    dir_fd: HostFd,
    pathname: SandboxedPath,
    buf: &mut Vec<u8>,
    cnt: usize,
) -> usize {
    let os_fd: usize = dir_fd.into();
    let os_path: Vec<u8> = pathname.into();
    unsafe {
        let result = syscall!(READLINKAT, os_fd, os_path.as_ptr(), buf.as_mut_ptr(), cnt);
        buf.set_len(result);
        result
    }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_unlinkat)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::PathAccess))]
pub fn trace_unlinkat(
    ctx: &VmCtx,
    dir_fd: HostFd,
    pathname: SandboxedPath,
    flags: libc::c_int,
) -> usize {
    effect!(trace, Effect::PathAccess);
    os_unlinkat(dir_fd, pathname, flags)
}

#[trusted]
pub fn os_unlinkat(dir_fd: HostFd, pathname: SandboxedPath, flags: libc::c_int) -> usize {
    let os_fd: usize = dir_fd.into();
    let os_path: Vec<u8> = pathname.into();
    unsafe { syscall!(UNLINKAT, os_fd, os_path.as_ptr(), flags) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_renameat)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::PathAccess))]
pub fn trace_renameat(
    ctx: &VmCtx,
    old_dir_fd: HostFd,
    old_pathname: SandboxedPath,
    dir_fd: HostFd,
    new_pathname: SandboxedPath,
) -> usize {
    effect!(trace, Effect::PathAccess);
    os_renameat(old_dir_fd, old_pathname, dir_fd, new_pathname)
}

#[trusted]
pub fn os_renameat(
    old_dir_fd: HostFd,
    old_pathname: SandboxedPath,
    new_dir_fd: HostFd,
    new_pathname: SandboxedPath,
) -> usize {
    let os_old_fd: usize = old_dir_fd.into();
    let os_old_path: Vec<u8> = old_pathname.into();
    let os_new_fd: usize = new_dir_fd.into();
    let os_new_path: Vec<u8> = new_pathname.into();
    unsafe {
        syscall!(
            RENAMEAT,
            os_old_fd,
            os_old_path.as_ptr(),
            os_new_fd,
            os_new_path.as_ptr()
        )
    }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_symlinkat)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::PathAccess))]
pub fn trace_symlinkat(
    ctx: &VmCtx,
    old_pathname: SandboxedPath,
    dir_fd: HostFd,
    new_pathname: SandboxedPath,
) -> usize {
    effect!(trace, Effect::PathAccess);
    os_symlinkat(old_pathname, dir_fd, new_pathname)
}

#[trusted]
pub fn os_symlinkat(
    old_pathname: SandboxedPath,
    dir_fd: HostFd,
    new_pathname: SandboxedPath,
) -> usize {
    let os_fd: usize = dir_fd.into();
    let os_old_path: Vec<u8> = old_pathname.into();
    let os_new_path: Vec<u8> = new_pathname.into();
    unsafe { syscall!(SYMLINKAT, os_old_path.as_ptr(), os_fd, os_new_path.as_ptr()) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_futimens)]
#[requires(specs.capacity() >= 2)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_futimens(ctx: &VmCtx, fd: HostFd, specs: &Vec<libc::timespec>) -> usize {
    effect!(trace, Effect::FdAccess);
    os_futimens(fd, specs)
}

#[requires(specs.capacity() >= 2)]
#[trusted]
pub fn os_futimens(fd: HostFd, specs: &Vec<libc::timespec>) -> usize {
    let os_fd: usize = fd.into();
    // Linux impls futimens as UTIMENSAT with null path
    // source: https://code.woboq.org/userspace/glibc/sysdeps/unix/sysv/linux/futimens.c.html
    unsafe { syscall!(UTIMENSAT, os_fd, 0, specs.as_ptr(), 0) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_utimensat)]
#[requires(specs.capacity() >= 2)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
// #[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_utimensat(
    ctx: &VmCtx,
    fd: HostFd,
    pathname: SandboxedPath,
    specs: &Vec<libc::timespec>,
    flags: libc::c_int,
) -> usize {
    effect!(trace, Effect::PathAccess);
    os_utimensat(fd, pathname, specs, flags)
}

#[requires(specs.capacity() >= 2)]
#[trusted]
pub fn os_utimensat(
    fd: HostFd,
    pathname: SandboxedPath,
    specs: &Vec<libc::timespec>,
    flags: libc::c_int,
) -> usize {
    let os_fd: usize = fd.into();
    let os_path: Vec<u8> = pathname.into();
    unsafe { syscall!(UTIMENSAT, os_fd, os_path.as_ptr(), specs.as_ptr(), flags) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_clock_get_time)]
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

#[trusted]
pub fn os_clock_get_time(clock_id: libc::clockid_t, spec: &mut libc::timespec) -> usize {
    unsafe { syscall!(CLOCK_GETTIME, clock_id, spec as *mut libc::timespec) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_clock_get_res)]
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

#[trusted]
pub fn os_clock_get_res(clock_id: libc::clockid_t, spec: &mut libc::timespec) -> usize {
    unsafe { syscall!(CLOCK_GETRES, clock_id, spec as *mut libc::timespec) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_getrandom)]
#[requires(trace_safe(ctx, trace))]
#[requires(buf.capacity() >= cnt)]
#[ensures(buf.len() == result)]
#[ensures(buf.capacity() >= cnt)]
#[ensures(trace_safe(ctx, trace))]
// #[ensures(one_effect!(old(trace), trace, Effect::WriteN(count) if count == cnt))]
pub fn trace_getrandom(ctx: &VmCtx, buf: &mut Vec<u8>, cnt: usize, flags: u32) -> usize {
    //effect!(trace, Effect::WriteN(cnt));
    os_getrandom(buf, cnt, flags)
}

#[requires(buf.capacity() >= cnt)]
#[ensures(buf.len() == result)]
#[ensures(buf.capacity() >= cnt)]
#[trusted]
pub fn os_getrandom(buf: &mut Vec<u8>, cnt: usize, flags: u32) -> usize {
    unsafe { syscall!(GETRANDOM, buf.as_mut_ptr(), cnt, flags) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_recv)]
#[requires(buf.capacity() >= cnt)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
#[ensures(buf.len() == result)]
#[ensures(buf.capacity() >= cnt)]
pub fn trace_recv(ctx: &VmCtx, fd: HostFd, buf: &mut Vec<u8>, cnt: usize, flags: u32) -> usize {
    effect!(trace, Effect::FdAccess);
    os_recv(fd, buf, cnt, flags)
}

#[requires(buf.capacity() >= cnt)]
#[ensures(buf.len() == result)]
#[ensures(buf.capacity() >= cnt)]
#[trusted]
pub fn os_recv(fd: HostFd, buf: &mut Vec<u8>, cnt: usize, flags: u32) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(RECVFROM, os_fd, buf.as_mut_ptr(), cnt, flags, 0, 0) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_send)]
#[requires(buf.len() >= cnt)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_send(ctx: &VmCtx, fd: HostFd, buf: &Vec<u8>, cnt: usize, flags: u32) -> usize {
    effect!(trace, Effect::FdAccess);
    os_send(fd, buf, cnt, flags)
}

#[requires(buf.len() >= cnt)]
#[trusted]
pub fn os_send(fd: HostFd, buf: &Vec<u8>, cnt: usize, flags: u32) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(SENDTO, os_fd, buf.as_ptr(), cnt, flags, 0, 0) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_shutdown)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::Shutdown))]
pub fn trace_shutdown(ctx: &VmCtx, fd: HostFd, how: libc::c_int) -> usize {
    effect!(trace, Effect::Shutdown);
    os_shutdown(fd, how)
}

#[trusted]
pub fn os_shutdown(fd: HostFd, how: libc::c_int) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(SHUTDOWN, os_fd, how) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_nanosleep)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(no_effect!(old(trace), trace))]
pub fn trace_nanosleep(ctx: &VmCtx, req: &libc::timespec, rem: &mut libc::timespec) -> usize {
    os_nanosleep(req, rem)
}

#[trusted]
pub fn os_nanosleep(req: &libc::timespec, rem: &mut libc::timespec) -> usize {
    unsafe {
        syscall!(
            NANOSLEEP,
            req as *const libc::timespec,
            rem as *mut libc::timespec
        )
    }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_poll)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(no_effect!(old(trace), trace))]
pub fn trace_poll(ctx: &VmCtx, pollfd: &mut libc::pollfd, timeout: libc::c_int) -> usize {
    os_poll(pollfd, timeout)
}

// can make more efficient using slice of pollfds
#[trusted]
pub fn os_poll(pollfd: &mut libc::pollfd, timeout: libc::c_int) -> usize {
    unsafe { syscall!(POLL, pollfd as *const libc::pollfd, 1, timeout) }
}
