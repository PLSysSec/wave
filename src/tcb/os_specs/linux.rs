#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::{effect, four_effects, no_effect, one_effect, three_effects, two_effects};
use extra_args::{external_call, external_method, with_ghost_var};
use prusti_contracts::*;
use syscall::syscall;

//https://man7.org/linux/man-pages/man2/open.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, Effect::PathAccess))]
pub fn os_open(pathname: Vec<u8>, flags: i32) -> usize {
    unsafe { syscall!(OPEN, pathname.as_ptr(), flags) }
}

//https://man7.org/linux/man-pages/man2/close.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn os_close(fd: usize) -> usize {
    unsafe { syscall!(CLOSE, fd) }
}

// https://man7.org/linux/man-pages/man2/read.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(buf.len() >= cnt)]
#[ensures(buf.len() >= result)]
#[ensures(result <= cnt)]
#[trusted]
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::WriteN(count) if count == cnt))]
//TODO: fix the result handling
pub fn os_read(fd: usize, buf: &mut [u8], cnt: usize) -> usize {
    unsafe { syscall!(READ, fd, buf.as_mut_ptr(), cnt) }
}

//https://man7.org/linux/man-pages/man2/pread.2.html
#[with_ghost_var(trace: &mut Trace)]
#[external_method(set_len)]
#[requires(buf.len() >= cnt)]
#[ensures(buf.len() >= result)]
#[ensures(result <= cnt)]
#[trusted]
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::WriteN(count) if count == cnt))]
pub fn os_pread(fd: usize, buf: &mut [u8], cnt: usize) -> usize {
    unsafe { syscall!(PREAD64, fd, buf.as_mut_ptr(), cnt) }
}

//https://man7.org/linux/man-pages/man2/write.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(buf.len() >= cnt)]
#[trusted]
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::ReadN(count) if count == cnt))]
pub fn os_write(fd: usize, buf: &[u8], cnt: usize) -> usize {
    unsafe { syscall!(WRITE, fd, buf.as_ptr(), cnt) }
}

//https://man7.org/linux/man-pages/man2/pwrite.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(buf.len() >= cnt)]
#[trusted]
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::ReadN(count) if count == cnt))]
pub fn os_pwrite(fd: usize, buf: &Vec<u8>, cnt: usize) -> usize {
    unsafe { syscall!(WRITE, fd, buf.as_ptr(), cnt) }
}

//https://man7.org/linux/man-pages/man2/lseek.2.html
// TODO: could be cleaner to do a typedef SyscallRet = usize or something for From traits
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn os_seek(fd: usize, offset: i64, whence: i32) -> usize {
    unsafe { syscall!(LSEEK, fd, offset, whence) }
}

//https://man7.org/linux/man-pages/man2/fadvise64.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn os_advise(fd: usize, offset: i64, len: i64, advice: i32) -> usize {
    unsafe { syscall!(FADVISE64, fd, offset, len, advice) }
}

// https://man7.org/linux/man-pages/man2/fallocate.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn os_allocate(fd: usize, offset: i64, len: i64) -> usize {
    unsafe { syscall!(FALLOCATE, fd, offset, len) }
}

//https://man7.org/linux/man-pages/man2/fsync.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn os_sync(fd: usize) -> usize {
    unsafe { syscall!(FSYNC, fd) }
}

//https://man7.org/linux/man-pages/man2/fdatasync.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn os_datasync(fd: usize) -> usize {
    unsafe { syscall!(FDATASYNC, fd) }
}

//https://man7.org/linux/man-pages/man2/fstat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn os_fstat(fd: usize, stat: &mut libc::stat) -> usize {
    unsafe { syscall!(FSTAT, fd, stat as *mut libc::stat) }
}

//https://man7.org/linux/man-pages/man2/fstatat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::PathAccess))]
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

//https://man7.org/linux/man-pages/man2/fcntl.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn os_fgetfl(fd: usize) -> usize {
    unsafe { syscall!(FCNTL, fd, libc::F_GETFL, 0) }
}

//https://man7.org/linux/man-pages/man2/fcntl.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn os_fsetfl(fd: usize, flags: libc::c_int) -> usize {
    unsafe { syscall!(FCNTL, fd, libc::F_SETFL, flags) }
}

//https://man7.org/linux/man-pages/man2/ftruncate.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn os_ftruncate(fd: usize, length: libc::off_t) -> usize {
    unsafe { syscall!(FTRUNCATE, fd, length) }
}

//https://man7.org/linux/man-pages/man2/linkat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(four_effects!(old(trace), trace, Effect::FdAccess, Effect::FdAccess, Effect::PathAccess, Effect::PathAccess))]
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

//https://man7.org/linux/man-pages/man2/mkdirat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::PathAccess))]
pub fn os_mkdirat(dir_fd: usize, pathname: Vec<u8>, mode: libc::mode_t) -> usize {
    unsafe { syscall!(MKDIRAT, dir_fd, pathname.as_ptr(), mode) }
}

//https://man7.org/linux/man-pages/man2/readlinkat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[external_method(set_len)]
#[requires(buf.capacity() >= cnt)]
#[ensures(buf.len() == result)]
#[ensures(buf.capacity() >= cnt)]
#[trusted]
#[ensures(three_effects!(old(trace), trace, Effect::FdAccess, Effect::PathAccess, Effect::WriteN(count) if count == cnt))]
pub fn os_readlinkat(dir_fd: usize, pathname: Vec<u8>, buf: &mut Vec<u8>, cnt: usize) -> usize {
    unsafe {
        let result = syscall!(READLINKAT, dir_fd, pathname.as_ptr(), buf.as_mut_ptr(), cnt);
        buf.set_len(result);
        result
    }
}

//https://man7.org/linux/man-pages/man2/unlinkat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::PathAccess))]
pub fn os_unlinkat(dir_fd: usize, pathname: Vec<u8>, flags: libc::c_int) -> usize {
    unsafe { syscall!(UNLINKAT, dir_fd, pathname.as_ptr(), flags) }
}

//https://man7.org/linux/man-pages/man2/renameat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(four_effects!(old(trace), trace, Effect::FdAccess, Effect::PathAccess, Effect::FdAccess, Effect::PathAccess))]
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

//https://man7.org/linux/man-pages/man2/symlinkat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(three_effects!(old(trace), trace,  Effect::PathAccess, Effect::FdAccess, Effect::PathAccess))]
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

//https://man7.org/linux/man-pages/man2/utimensat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(specs.capacity() >= 2)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn os_futimens(fd: usize, specs: &Vec<libc::timespec>) -> usize {
    // Linux impls futimens as UTIMENSAT with null path
    // source: https://code.woboq.org/userspace/glibc/sysdeps/unix/sysv/linux/futimens.c.html
    unsafe { syscall!(UTIMENSAT, fd, 0, specs.as_ptr(), 0) }
}

//https://man7.org/linux/man-pages/man2/utimensat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(specs.capacity() >= 2)]
#[trusted]
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::PathAccess))]
pub fn os_utimensat(
    fd: usize,
    pathname: Vec<u8>,
    specs: &Vec<libc::timespec>,
    flags: libc::c_int,
) -> usize {
    unsafe { syscall!(UTIMENSAT, fd, pathname.as_ptr(), specs.as_ptr(), flags) }
}

//https://man7.org/linux/man-pages/man2/clock_gettime.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(no_effect!(old(trace), trace))]
pub fn os_clock_get_time(clock_id: libc::clockid_t, spec: &mut libc::timespec) -> usize {
    unsafe { syscall!(CLOCK_GETTIME, clock_id, spec as *mut libc::timespec) }
}

//https://man7.org/linux/man-pages/man2/clock_getres.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(no_effect!(old(trace), trace))]
pub fn os_clock_get_res(clock_id: libc::clockid_t, spec: &mut libc::timespec) -> usize {
    unsafe { syscall!(CLOCK_GETRES, clock_id, spec as *mut libc::timespec) }
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(buf.capacity() >= cnt)]
#[ensures(buf.len() == result)]
#[ensures(buf.capacity() >= cnt)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, Effect::WriteN(count) if count == cnt))]
pub fn os_getrandom(buf: &mut Vec<u8>, cnt: usize, flags: u32) -> usize {
    unsafe { syscall!(GETRANDOM, buf.as_mut_ptr(), cnt, flags) }
}

//https://man7.org/linux/man-pages/man2/recvfrom.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(buf.capacity() >= cnt)]
#[ensures(buf.len() == result)]
#[ensures(buf.capacity() >= cnt)]
#[trusted]
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::WriteN(count) if count == cnt))]
pub fn os_recv(fd: usize, buf: &mut Vec<u8>, cnt: usize, flags: u32) -> usize {
    unsafe { syscall!(RECVFROM, fd, buf.as_mut_ptr(), cnt, flags, 0, 0) }
}

//https://man7.org/linux/man-pages/man2/sendto.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(buf.len() >= cnt)]
#[trusted]
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::ReadN(count) if count == cnt))]
pub fn os_send(fd: usize, buf: &Vec<u8>, cnt: usize, flags: u32) -> usize {
    unsafe { syscall!(SENDTO, fd, buf.as_ptr(), cnt, flags, 0, 0) }
}

//https://man7.org/linux/man-pages/man2/shutdown.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(two_effects!(old(trace), trace, Effect::Shutdown, Effect::FdAccess))]
pub fn os_shutdown(fd: usize, how: libc::c_int) -> usize {
    unsafe { syscall!(SHUTDOWN, fd, how) }
}

//https://man7.org/linux/man-pages/man2/nanosleep.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(no_effect!(old(trace), trace))]
pub fn os_nanosleep(req: &libc::timespec, rem: &mut libc::timespec) -> usize {
    unsafe {
        syscall!(
            NANOSLEEP,
            req as *const libc::timespec,
            rem as *mut libc::timespec
        )
    }
}

//https://man7.org/linux/man-pages/man2/poll.2.html
// can make more efficient using slice of pollfds
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(no_effect!(old(trace), trace))]
pub fn os_poll(pollfd: &mut libc::pollfd, timeout: libc::c_int) -> usize {
    unsafe { syscall!(POLL, pollfd as *const libc::pollfd, 1, timeout) }
}
