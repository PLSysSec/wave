use crate::effect;
use crate::trace::*;
use crate::types::*;
use prusti_contracts::*;
use syscall::syscall;

/// This module contains our syscall specifications
/// These functions must be trusted because we don't know what the os actually does
/// on a syscall

#[trusted]
pub fn os_open(pathname: SandboxedPath, flags: i32) -> usize {
    let os_path: Vec<u8> = pathname.into();
    unsafe { syscall!(OPEN, os_path.as_ptr(), flags) }
}

#[trusted]
pub fn os_close(fd: HostFd) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(CLOSE, os_fd) }
}

#[requires(buf.len() >= cnt)]
#[ensures(buf.len() >= cnt)]
// TODO: the top checking is kinda gross, will figure out refactor later
// TODO: the following three predicates will be common...is there a way to modularize?
//       cannot use predicate! afaik cause we cannot use old(trace) in them...
#[ensures(trace.len() == old(trace.len()) + 1)]
#[ensures(match trace.lookup(trace.len()-1) {
    Effect::ReadN { count } => count == cnt,
    _ => false,
})]
#[ensures(forall(|i: usize| (i < old(trace.len())) ==>
                trace.lookup(i) == old(trace.lookup(i))))]
pub fn trace_read(fd: HostFd, buf: &mut [u8], cnt: usize, trace: &mut Trace) -> usize {
    effect!(trace, Effect::ReadN { count: cnt });
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

#[requires(buf.len() >= cnt)]
// TODO: the top checking is kinda gross, will figure out refactor later
// TODO: the following three predicates will be common...is there a way to modularize?
//       cannot use predicate! afaik cause we cannot use old(trace) in them...
#[ensures(trace.len() == old(trace.len()) + 1)]
#[ensures(match trace.lookup(trace.len()-1) {
    Effect::WriteN { count } => count == cnt,
    _ => false,
})]
#[ensures(forall(|i: usize| (i < old(trace.len())) ==>
                trace.lookup(i) == old(trace.lookup(i))))]
pub fn trace_write(fd: HostFd, buf: &[u8], cnt: usize, trace: &mut Trace) -> usize {
    effect!(trace, Effect::WriteN { count: cnt });
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

#[trusted]
pub fn os_advise(fd: HostFd, offset: i64, len: i64, advice: i32) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(FADVISE64, os_fd, offset, len, advice) }
}

#[trusted]
pub fn os_allocate(fd: HostFd, offset: i64, len: i64) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(FALLOCATE, os_fd, offset, len) }
}

#[trusted]
pub fn os_sync(fd: HostFd) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(FSYNC, os_fd) }
}

#[trusted]
pub fn os_datasync(fd: HostFd) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(FDATASYNC, os_fd) }
}

#[trusted]
pub fn os_fstat(fd: HostFd, stat: &mut libc::stat) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(FSTAT, os_fd, stat as *mut libc::stat) }
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

#[trusted]
pub fn os_fgetfl(fd: HostFd) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(FCNTL, os_fd, libc::F_GETFL, 0) }
}

#[trusted]
pub fn os_fsetfl(fd: HostFd, flags: libc::c_int) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(FCNTL, os_fd, libc::F_SETFL, flags) }
}

#[trusted]
pub fn os_ftruncate(fd: HostFd, length: libc::off_t) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(FTRUNCATE, os_fd, length) }
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

#[trusted]
pub fn os_mkdirat(dir_fd: HostFd, pathname: SandboxedPath, mode: libc::mode_t) -> usize {
    let os_fd: usize = dir_fd.into();
    let os_path: Vec<u8> = pathname.into();
    unsafe { syscall!(MKDIRAT, os_fd, os_path.as_ptr(), mode) }
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

#[trusted]
pub fn os_unlinkat(dir_fd: HostFd, pathname: SandboxedPath, flags: libc::c_int) -> usize {
    let os_fd: usize = dir_fd.into();
    let os_path: Vec<u8> = pathname.into();
    unsafe { syscall!(UNLINKAT, os_fd, os_path.as_ptr(), flags) }
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

#[requires(specs.capacity() >= 2)]
#[trusted]
pub fn os_futimens(fd: HostFd, specs: &Vec<libc::timespec>) -> usize {
    let os_fd: usize = fd.into();
    // Linux impls futimens as UTIMENSAT with null path
    // source: https://code.woboq.org/userspace/glibc/sysdeps/unix/sysv/linux/futimens.c.html
    unsafe { syscall!(UTIMENSAT, os_fd, 0, specs.as_ptr(), 0) }
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

#[trusted]
pub fn os_clock_get_time(clock_id: libc::clockid_t, spec: &mut libc::timespec) -> usize {
    unsafe { syscall!(CLOCK_GETTIME, clock_id, spec as *mut libc::timespec) }
}

#[trusted]
pub fn os_clock_get_res(clock_id: libc::clockid_t, spec: &mut libc::timespec) -> usize {
    unsafe { syscall!(CLOCK_GETRES, clock_id, spec as *mut libc::timespec) }
}

#[requires(buf.capacity() >= cnt)]
#[ensures(buf.len() == result)]
#[ensures(buf.capacity() >= cnt)]
#[trusted]
pub fn os_getrandom(buf: &mut Vec<u8>, cnt: usize, flags: u32) -> usize {
    unsafe { syscall!(GETRANDOM, buf.as_mut_ptr(), cnt, flags) }
}

#[requires(buf.capacity() >= cnt)]
#[ensures(buf.len() == result)]
#[ensures(buf.capacity() >= cnt)]
#[trusted]
pub fn os_recv(fd: HostFd, buf: &mut Vec<u8>, cnt: usize, flags: u32) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(RECVFROM, os_fd, buf.as_mut_ptr(), cnt, flags, 0, 0) }
}

#[requires(buf.len() >= cnt)]
#[trusted]
pub fn os_send(fd: HostFd, buf: &Vec<u8>, cnt: usize, flags: u32) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(SENDTO, os_fd, buf.as_ptr(), cnt, flags, 0, 0) }
}

#[ensures(trace.len() == old(trace.len()) + 1)]
#[ensures( matches!(trace.lookup(trace.len()-1), Effect::Shutdown) )]
#[ensures(forall(|i: usize| (i < old(trace.len())) ==>
                trace.lookup(i) == old(trace.lookup(i))))]
pub fn trace_shutdown(fd: HostFd, how: libc::c_int, trace: &mut Trace) -> usize {
    effect!(trace, Effect::Shutdown);
    os_shutdown(fd, how)
}

#[trusted]
pub fn os_shutdown(fd: HostFd, how: libc::c_int) -> usize {
    let os_fd: usize = fd.into();
    unsafe { syscall!(SHUTDOWN, os_fd, how) }
}

//#[trusted]
//pub fn os_

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

// can make more efficient using slice of pollfds
#[trusted]
pub fn os_poll(pollfd: &mut libc::pollfd, timeout: libc::c_int) -> usize {
    unsafe { syscall!(POLL, pollfd as *const libc::pollfd, 1, timeout) }
}
