#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::{effect, four_effects, no_effect, one_effect, three_effects, two_effects};
//use crate::trace::*;
use crate::types::*;
use extra_args::{external_call, external_method, with_ghost_var};
use prusti_contracts::*;
use syscall::syscall;

/// This module contains our syscall specifications
/// These functions must be trusted because we don't know what the os actually does
/// on a syscall

#[with_ghost_var(trace: &mut Trace)]
#[external_method(into)]
#[external_call(os_open)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::PathAccess))]
pub fn trace_open(ctx: &VmCtx, pathname: SandboxedPath, flags: i32) -> usize {
    effect!(trace, Effect::PathAccess);
    let os_path: Vec<u8> = pathname.into();
    os_open(os_path, flags)
}

//https://man7.org/linux/man-pages/man2/open.2.html
#[trusted]
pub fn os_open(pathname: Vec<u8>, flags: i32) -> usize {
    unsafe { syscall!(OPEN, pathname.as_ptr(), flags) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_close)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_close(ctx: &VmCtx, fd: HostFd) -> usize {
    effect!(trace, Effect::FdAccess);
    let os_fd: usize = fd.into();
    os_close(os_fd)
}

//https://man7.org/linux/man-pages/man2/close.2.html
#[trusted]
pub fn os_close(fd: usize) -> usize {
    unsafe { syscall!(CLOSE, fd) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_method(into)]
#[external_call(os_read)]
#[requires(buf.len() >= cnt)]
#[requires(cnt < ctx.memlen)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(buf.len() >= cnt)]
#[ensures(result <= cnt)]
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::WriteN(count)))]
/// read writes `cnt` bytes to sandbox memory
// #[ensures(one_effect!(old(trace), trace, Effect::WriteN(count) if count == cnt ))]
pub fn trace_read(ctx: &VmCtx, fd: HostFd, buf: &mut [u8], cnt: usize) -> usize {
    effect!(trace, Effect::FdAccess);
    effect!(trace, Effect::WriteN(cnt));
    let os_fd: usize = fd.into();
    os_read(os_fd, buf, cnt)
}

// https://man7.org/linux/man-pages/man2/read.2.html
#[requires(buf.len() >= cnt)]
#[ensures(buf.len() >= cnt)]
#[ensures(result <= cnt)]
#[trusted]
//TODO: fix the result handling
pub fn os_read(fd: usize, buf: &mut [u8], cnt: usize) -> usize {
    unsafe { syscall!(READ, fd, buf.as_mut_ptr(), cnt) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_pread)]
#[external_method(into)]
#[requires(buf.capacity() >= cnt)]
#[requires(cnt < ctx.memlen)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(buf.len() == result)]
#[ensures(buf.capacity() >= cnt)]
/// pread writes `cnt` bytes to sandbox memory
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::WriteN(count)))]
// #[ensures(one_effect!(old(trace), trace, Effect::WriteN(count) if count == cnt ))]
// #[trusted]
pub fn trace_pread(ctx: &VmCtx, fd: HostFd, buf: &mut Vec<u8>, cnt: usize) -> usize {
    let os_fd: usize = fd.into();
    effect!(trace, Effect::FdAccess);
    effect!(trace, Effect::WriteN(cnt));
    os_pread(os_fd, buf, cnt)
}

//https://man7.org/linux/man-pages/man2/pread.2.html
#[requires(buf.capacity() >= cnt)]
#[ensures(buf.len() == result)]
#[ensures(buf.capacity() >= cnt)]
#[trusted]
pub fn os_pread(fd: usize, buf: &mut Vec<u8>, cnt: usize) -> usize {
    unsafe {
        let result = syscall!(PREAD64, fd, buf.as_mut_ptr(), cnt);
        buf.set_len(result);
        result
    }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_write)]
#[external_method(into)]
#[requires(buf.len() >= cnt)]
#[requires(trace_safe(ctx, trace))]
#[requires(cnt < ctx.memlen)]
#[ensures(trace_safe(ctx, trace))]
// write reads `cnt` bytes to the sandbox
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::ReadN(count)))]
// #[ensures(one_effect!(old(trace), trace, Effect::ReadN(count) if count == cnt ))]
pub fn trace_write(ctx: &VmCtx, fd: HostFd, buf: &[u8], cnt: usize) -> usize {
    effect!(trace, Effect::FdAccess);
    effect!(trace, Effect::ReadN(cnt));
    let os_fd: usize = fd.into();
    os_write(os_fd, buf, cnt)
}

//https://man7.org/linux/man-pages/man2/write.2.html
#[requires(buf.len() >= cnt)]
#[trusted]
pub fn os_write(fd: usize, buf: &[u8], cnt: usize) -> usize {
    unsafe { syscall!(WRITE, fd, buf.as_ptr(), cnt) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_pwrite)]
#[external_method(into)]
#[requires(buf.len() >= cnt)]
#[requires(cnt < ctx.memlen)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
// pwrite writes `cnt` bytes to the sandbox
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::ReadN(count)))]
// #[ensures(one_effect!(old(trace), trace, Effect::ReadN(count) if count == cnt ))]
pub fn trace_pwrite(ctx: &VmCtx, fd: HostFd, buf: &Vec<u8>, cnt: usize) -> usize {
    effect!(trace, Effect::FdAccess);
    effect!(trace, Effect::ReadN(cnt));
    let os_fd: usize = fd.into();
    os_pwrite(os_fd, buf, cnt)
}

//https://man7.org/linux/man-pages/man2/pwrite.2.html
#[requires(buf.len() >= cnt)]
#[trusted]
pub fn os_pwrite(fd: usize, buf: &Vec<u8>, cnt: usize) -> usize {
    unsafe { syscall!(WRITE, fd, buf.as_ptr(), cnt) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_seek)] // Do not add trace to os_seek
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_seek(ctx: &VmCtx, fd: HostFd, offset: i64, whence: i32) -> usize {
    effect!(trace, Effect::FdAccess);
    let os_fd: usize = fd.into();
    os_seek(os_fd, offset, whence)
}

//https://man7.org/linux/man-pages/man2/lseek.2.html
// TODO: could be cleaner to do a typedef SyscallRet = usize or something for From traits
#[trusted]
pub fn os_seek(fd: usize, offset: i64, whence: i32) -> usize {
    unsafe { syscall!(LSEEK, fd, offset, whence) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_advise)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_advise(ctx: &VmCtx, fd: HostFd, offset: i64, len: i64, advice: i32) -> usize {
    effect!(trace, Effect::FdAccess);
    let os_fd: usize = fd.into();
    os_advise(os_fd, offset, len, advice)
}

//https://man7.org/linux/man-pages/man2/fadvise64.2.html
#[trusted]
pub fn os_advise(fd: usize, offset: i64, len: i64, advice: i32) -> usize {
    unsafe { syscall!(FADVISE64, fd, offset, len, advice) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_allocate)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_allocate(ctx: &VmCtx, fd: HostFd, offset: i64, len: i64) -> usize {
    effect!(trace, Effect::FdAccess);
    let os_fd: usize = fd.into();
    os_allocate(os_fd, offset, len)
}

// https://man7.org/linux/man-pages/man2/fallocate.2.html
#[trusted]
pub fn os_allocate(fd: usize, offset: i64, len: i64) -> usize {
    unsafe { syscall!(FALLOCATE, fd, offset, len) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_sync)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_sync(ctx: &VmCtx, fd: HostFd) -> usize {
    effect!(trace, Effect::FdAccess);
    let os_fd: usize = fd.into();
    os_sync(os_fd)
}

//https://man7.org/linux/man-pages/man2/fsync.2.html
#[trusted]
pub fn os_sync(fd: usize) -> usize {
    unsafe { syscall!(FSYNC, fd) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_datasync)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_datasync(ctx: &VmCtx, fd: HostFd) -> usize {
    effect!(trace, Effect::FdAccess);
    let os_fd: usize = fd.into();
    os_datasync(os_fd)
}

//https://man7.org/linux/man-pages/man2/fdatasync.2.html
#[trusted]
pub fn os_datasync(fd: usize) -> usize {
    unsafe { syscall!(FDATASYNC, fd) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_fstat)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_fstat(ctx: &VmCtx, fd: HostFd, stat: &mut libc::stat) -> usize {
    effect!(trace, Effect::FdAccess);
    let os_fd: usize = fd.into();
    os_fstat(os_fd, stat)
}

//https://man7.org/linux/man-pages/man2/fstat.2.html
#[trusted]
pub fn os_fstat(fd: usize, stat: &mut libc::stat) -> usize {
    unsafe { syscall!(FSTAT, fd, stat as *mut libc::stat) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_fstatat)]
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
    effect!(trace, Effect::FdAccess);
    effect!(trace, Effect::PathAccess);
    let os_fd: usize = fd.into();
    let os_path: Vec<u8> = path.into();
    os_fstatat(os_fd, os_path, stat, flags)
}

//https://man7.org/linux/man-pages/man2/fstatat.2.html
#[trusted]
pub fn os_fstatat(fd: usize, path: Vec<u8>, stat: &mut libc::stat, flags: i32) -> usize {
    unsafe {
        syscall!(
            NEWFSTATAT,
            fd,
            path.as_ptr(),
            stat as *mut libc::stat,
            flags
        )
    }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_fgetfl)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_fgetfl(ctx: &VmCtx, fd: HostFd) -> usize {
    effect!(trace, Effect::FdAccess);
    let os_fd: usize = fd.into();
    os_fgetfl(os_fd)
}

//https://man7.org/linux/man-pages/man2/fcntl.2.html
#[trusted]
pub fn os_fgetfl(fd: usize) -> usize {
    unsafe { syscall!(FCNTL, fd, libc::F_GETFL, 0) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_fsetfl)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_fsetfl(ctx: &VmCtx, fd: HostFd, flags: libc::c_int) -> usize {
    effect!(trace, Effect::FdAccess);
    let os_fd: usize = fd.into();
    os_fsetfl(os_fd, flags)
}

//https://man7.org/linux/man-pages/man2/fcntl.2.html
#[trusted]
pub fn os_fsetfl(fd: usize, flags: libc::c_int) -> usize {
    unsafe { syscall!(FCNTL, fd, libc::F_SETFL, flags) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_ftruncate)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_ftruncate(ctx: &VmCtx, fd: HostFd, length: libc::off_t) -> usize {
    effect!(trace, Effect::FdAccess);
    let os_fd: usize = fd.into();
    os_ftruncate(os_fd, length)
}

//https://man7.org/linux/man-pages/man2/ftruncate.2.html
#[trusted]
pub fn os_ftruncate(fd: usize, length: libc::off_t) -> usize {
    unsafe { syscall!(FTRUNCATE, fd, length) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_linkat)]
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
    effect!(trace, Effect::FdAccess);
    effect!(trace, Effect::FdAccess);
    effect!(trace, Effect::PathAccess);
    effect!(trace, Effect::PathAccess);
    let os_old_fd: usize = old_fd.into();
    let os_new_fd: usize = new_fd.into();
    let os_old_path: Vec<u8> = old_path.into();
    let os_new_path: Vec<u8> = new_path.into();
    os_linkat(os_old_fd, os_old_path, os_new_fd, os_new_path, flags)
}

//https://man7.org/linux/man-pages/man2/linkat.2.html
#[trusted]
pub fn os_linkat(
    old_fd: usize,
    old_path: Vec<u8>,
    new_fd: usize,
    new_path: Vec<u8>,
    flags: i32,
) -> usize {
    unsafe {
        syscall!(
            LINKAT,
            old_fd,
            old_path.as_ptr(),
            new_fd,
            new_path.as_ptr(),
            flags
        )
    }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_mkdirat)]
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
    effect!(trace, Effect::FdAccess);
    effect!(trace, Effect::PathAccess);
    os_mkdirat(os_fd, os_path, mode)
}

//https://man7.org/linux/man-pages/man2/mkdirat.2.html
#[trusted]
pub fn os_mkdirat(dir_fd: usize, pathname: Vec<u8>, mode: libc::mode_t) -> usize {
    unsafe { syscall!(MKDIRAT, dir_fd, pathname.as_ptr(), mode) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_readlinkat)]
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
    effect!(trace, Effect::FdAccess);
    effect!(trace, Effect::PathAccess);
    effect!(trace, Effect::WriteN(cnt));
    let os_fd: usize = dir_fd.into();
    let os_path: Vec<u8> = pathname.into();
    os_readlinkat(os_fd, os_path, buf, cnt)
}

//https://man7.org/linux/man-pages/man2/readlinkat.2.html
#[requires(buf.capacity() >= cnt)]
#[ensures(buf.len() == result)]
#[ensures(buf.capacity() >= cnt)]
#[trusted]
pub fn os_readlinkat(dir_fd: usize, pathname: Vec<u8>, buf: &mut Vec<u8>, cnt: usize) -> usize {
    unsafe {
        let result = syscall!(READLINKAT, dir_fd, pathname.as_ptr(), buf.as_mut_ptr(), cnt);
        buf.set_len(result);
        result
    }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_unlinkat)]
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
    effect!(trace, Effect::FdAccess);
    effect!(trace, Effect::PathAccess);
    let os_fd: usize = dir_fd.into();
    let os_path: Vec<u8> = pathname.into();
    os_unlinkat(os_fd, os_path, flags)
}

//https://man7.org/linux/man-pages/man2/unlinkat.2.html
#[trusted]
pub fn os_unlinkat(dir_fd: usize, pathname: Vec<u8>, flags: libc::c_int) -> usize {
    unsafe { syscall!(UNLINKAT, dir_fd, pathname.as_ptr(), flags) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_renameat)]
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
    effect!(trace, Effect::FdAccess);
    effect!(trace, Effect::PathAccess);
    effect!(trace, Effect::FdAccess);
    effect!(trace, Effect::PathAccess);
    let os_old_fd: usize = old_dir_fd.into();
    let os_old_path: Vec<u8> = old_pathname.into();
    let os_new_fd: usize = new_dir_fd.into();
    let os_new_path: Vec<u8> = new_pathname.into();
    os_renameat(os_old_fd, os_old_path, os_new_fd, os_new_path)
}

//https://man7.org/linux/man-pages/man2/renameat.2.html
#[trusted]
pub fn os_renameat(
    old_dir_fd: usize,
    old_pathname: Vec<u8>,
    new_dir_fd: usize,
    new_pathname: Vec<u8>,
) -> usize {
    unsafe {
        syscall!(
            RENAMEAT,
            old_dir_fd,
            old_pathname.as_ptr(),
            new_dir_fd,
            new_pathname.as_ptr()
        )
    }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_symlinkat)]
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
    effect!(trace, Effect::PathAccess);
    effect!(trace, Effect::FdAccess);
    effect!(trace, Effect::PathAccess);
    let os_fd: usize = dir_fd.into();
    let os_old_path: Vec<u8> = old_pathname.into();
    let os_new_path: Vec<u8> = new_pathname.into();
    os_symlinkat(os_old_path, os_fd, os_new_path)
}

//https://man7.org/linux/man-pages/man2/symlinkat.2.html
#[trusted]
pub fn os_symlinkat(old_pathname: Vec<u8>, dir_fd: usize, new_pathname: Vec<u8>) -> usize {
    unsafe {
        syscall!(
            SYMLINKAT,
            old_pathname.as_ptr(),
            dir_fd,
            new_pathname.as_ptr()
        )
    }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_futimens)]
#[external_method(into)]
#[requires(specs.capacity() >= 2)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn trace_futimens(ctx: &VmCtx, fd: HostFd, specs: &Vec<libc::timespec>) -> usize {
    effect!(trace, Effect::FdAccess);
    let os_fd: usize = fd.into();
    os_futimens(os_fd, specs)
}

//https://man7.org/linux/man-pages/man2/utimensat.2.html
#[requires(specs.capacity() >= 2)]
#[trusted]
pub fn os_futimens(fd: usize, specs: &Vec<libc::timespec>) -> usize {
    // Linux impls futimens as UTIMENSAT with null path
    // source: https://code.woboq.org/userspace/glibc/sysdeps/unix/sysv/linux/futimens.c.html
    unsafe { syscall!(UTIMENSAT, fd, 0, specs.as_ptr(), 0) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_utimensat)]
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
    effect!(trace, Effect::FdAccess);
    effect!(trace, Effect::PathAccess);
    let os_fd: usize = fd.into();
    let os_path: Vec<u8> = pathname.into();
    os_utimensat(os_fd, os_path, specs, flags)
}

//https://man7.org/linux/man-pages/man2/utimensat.2.html
#[requires(specs.capacity() >= 2)]
#[trusted]
pub fn os_utimensat(
    fd: usize,
    pathname: Vec<u8>,
    specs: &Vec<libc::timespec>,
    flags: libc::c_int,
) -> usize {
    unsafe { syscall!(UTIMENSAT, fd, pathname.as_ptr(), specs.as_ptr(), flags) }
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

//https://man7.org/linux/man-pages/man2/clock_gettime.2.html
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

//https://man7.org/linux/man-pages/man2/clock_getres.2.html
#[trusted]
pub fn os_clock_get_res(clock_id: libc::clockid_t, spec: &mut libc::timespec) -> usize {
    unsafe { syscall!(CLOCK_GETRES, clock_id, spec as *mut libc::timespec) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_getrandom)]
#[requires(trace_safe(ctx, trace))]
#[requires(buf.capacity() >= cnt)]
#[requires(cnt < ctx.memlen)]
#[ensures(buf.len() == result)]
#[ensures(buf.capacity() >= cnt)]
#[ensures(one_effect!(old(trace), trace, Effect::WriteN(count)))]
// #[ensures(trace.len() == old(trace.len()) + 1)]
// #[ensures(match trace.lookup(trace.len()-1) {
//     Effect::WriteN(count) => count == cnt,
//     _ => false,
// })]
// #[ensures(forall(|i: usize| (i < old(trace.len())) ==>
//                     trace.lookup(i) == old(trace.lookup(i))))]
#[ensures(trace_safe(ctx, trace))]
pub fn trace_getrandom(ctx: &VmCtx, buf: &mut Vec<u8>, cnt: usize, flags: u32) -> usize {
    effect!(trace, Effect::WriteN(cnt));
    os_getrandom(buf, cnt, flags)
}

//https://man7.org/linux/man-pages/man2/getrandom.2.html
#[requires(buf.capacity() >= cnt)]
#[ensures(buf.len() == result)]
#[ensures(buf.capacity() >= cnt)]
#[trusted]
pub fn os_getrandom(buf: &mut Vec<u8>, cnt: usize, flags: u32) -> usize {
    unsafe { syscall!(GETRANDOM, buf.as_mut_ptr(), cnt, flags) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_recv)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[requires(cnt < ctx.memlen)]
#[ensures(trace_safe(ctx, trace))]
#[requires(buf.capacity() >= cnt)]
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::WriteN(count)))]
#[ensures(buf.len() == result)]
#[ensures(buf.capacity() >= cnt)]
pub fn trace_recv(ctx: &VmCtx, fd: HostFd, buf: &mut Vec<u8>, cnt: usize, flags: u32) -> usize {
    effect!(trace, Effect::FdAccess);
    effect!(trace, Effect::WriteN(cnt));
    let os_fd: usize = fd.into();
    os_recv(os_fd, buf, cnt, flags)
}

//https://man7.org/linux/man-pages/man2/recvfrom.2.html
#[requires(buf.capacity() >= cnt)]
#[ensures(buf.len() == result)]
#[ensures(buf.capacity() >= cnt)]
#[trusted]
pub fn os_recv(fd: usize, buf: &mut Vec<u8>, cnt: usize, flags: u32) -> usize {
    unsafe { syscall!(RECVFROM, fd, buf.as_mut_ptr(), cnt, flags, 0, 0) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_send)]
#[external_method(into)]
#[requires(buf.len() >= cnt)]
#[requires(cnt < ctx.memlen)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::ReadN(count)))]
pub fn trace_send(ctx: &VmCtx, fd: HostFd, buf: &Vec<u8>, cnt: usize, flags: u32) -> usize {
    effect!(trace, Effect::FdAccess);
    effect!(trace, Effect::ReadN(cnt));
    let os_fd: usize = fd.into();
    os_send(os_fd, buf, cnt, flags)
}

//https://man7.org/linux/man-pages/man2/sendto.2.html
#[requires(buf.len() >= cnt)]
#[trusted]
pub fn os_send(fd: usize, buf: &Vec<u8>, cnt: usize, flags: u32) -> usize {
    unsafe { syscall!(SENDTO, fd, buf.as_ptr(), cnt, flags, 0, 0) }
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_shutdown)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(two_effects!(old(trace), trace, Effect::Shutdown, Effect::FdAccess))]
pub fn trace_shutdown(ctx: &VmCtx, fd: HostFd, how: libc::c_int) -> usize {
    effect!(trace, Effect::Shutdown);
    effect!(trace, Effect::FdAccess);
    os_shutdown(fd, how)
}

//https://man7.org/linux/man-pages/man2/shutdown.2.html
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

//https://man7.org/linux/man-pages/man2/nanosleep.2.html
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

//TODO: not sure what the spec for this is yet.
#[with_ghost_var(trace: &mut Trace)]
#[external_call(os_poll)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(no_effect!(old(trace), trace))]
pub fn trace_poll(ctx: &VmCtx, pollfd: &mut libc::pollfd, timeout: libc::c_int) -> usize {
    os_poll(pollfd, timeout)
}

//https://man7.org/linux/man-pages/man2/poll.2.html
// can make more efficient using slice of pollfds
#[trusted]
pub fn os_poll(pollfd: &mut libc::pollfd, timeout: libc::c_int) -> usize {
    unsafe { syscall!(POLL, pollfd as *const libc::pollfd, 1, timeout) }
}
