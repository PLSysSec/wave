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
pub fn os_open(pathname: Vec<u8>, flags: i32) -> isize {
    unsafe { syscall!(OPEN, pathname.as_ptr(), flags) as isize }
}

//https://man7.org/linux/man-pages/man2/close.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn os_close(fd: usize) -> isize {
    unsafe { syscall!(CLOSE, fd) as isize }
}

// https://man7.org/linux/man-pages/man2/read.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(buf.len() >= cnt)]
#[ensures(result >= 0 ==> buf.len() >= result as usize)]
#[ensures(result >= 0 ==> result as usize <= cnt)]
#[trusted]
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::WriteN(count) if count == cnt))]
//TODO: fix the result handling
pub fn os_read(fd: usize, buf: &mut [u8], cnt: usize) -> isize {
    unsafe { syscall!(READ, fd, buf.as_mut_ptr(), cnt) as isize }
}

//https://man7.org/linux/man-pages/man2/pread.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(buf.len() >= cnt)]
#[ensures(result >= 0 ==> buf.len() >= result as usize)]
#[ensures(result >= 0 ==> result as usize <= cnt)]
#[trusted]
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::WriteN(count) if count == cnt))]
pub fn os_pread(fd: usize, buf: &mut [u8], cnt: usize, offset: usize) -> isize {
    unsafe { syscall!(PREAD64, fd, buf.as_mut_ptr(), cnt, offset) as isize }
}

//https://man7.org/linux/man-pages/man2/write.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(buf.len() >= cnt)]
#[trusted]
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::ReadN(count) if count == cnt))]
pub fn os_write(fd: usize, buf: &[u8], cnt: usize) -> isize {
    unsafe { syscall!(WRITE, fd, buf.as_ptr(), cnt) as isize }
}

//https://man7.org/linux/man-pages/man2/pwrite.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(buf.len() >= cnt)]
#[trusted]
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::ReadN(count) if count == cnt))]
pub fn os_pwrite(fd: usize, buf: &[u8], cnt: usize, offset: usize) -> isize {
    unsafe { syscall!(PWRITE64, fd, buf.as_ptr(), cnt, offset) as isize }
}

//https://man7.org/linux/man-pages/man2/lseek.2.html
// TODO: could be cleaner to do a typedef SyscallRet = usize or something for From traits
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn os_seek(fd: usize, offset: i64, whence: i32) -> isize {
    unsafe { syscall!(LSEEK, fd, offset, whence) as isize }
}

//https://man7.org/linux/man-pages/man2/fadvise64.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn os_advise(fd: usize, offset: i64, len: i64, advice: i32) -> isize {
    unsafe { syscall!(FADVISE64, fd, offset, len, advice) as isize }
}

// https://man7.org/linux/man-pages/man2/fallocate.2.html
// hardcode mode to 0 to behave more like posix_fallocate
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn os_allocate(fd: usize, offset: i64, len: i64) -> isize {
    unsafe { syscall!(FALLOCATE, fd, 0, offset, len) as isize }
}

//https://man7.org/linux/man-pages/man2/fsync.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn os_sync(fd: usize) -> isize {
    unsafe { syscall!(FSYNC, fd) as isize }
}

//https://man7.org/linux/man-pages/man2/fdatasync.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn os_datasync(fd: usize) -> isize {
    unsafe { syscall!(FDATASYNC, fd) as isize }
}

//https://man7.org/linux/man-pages/man2/fstat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn os_fstat(fd: usize, stat: &mut libc::stat) -> isize {
    unsafe { syscall!(FSTAT, fd, stat as *mut libc::stat) as isize }
}

//https://man7.org/linux/man-pages/man2/fstatat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::PathAccess))]
pub fn os_fstatat(fd: usize, path: Vec<u8>, stat: &mut libc::stat, flags: i32) -> isize {
    unsafe {
        syscall!(
            NEWFSTATAT,
            fd,
            path.as_ptr(),
            stat as *mut libc::stat,
            flags
        ) as isize
    }
}

//https://man7.org/linux/man-pages/man2/fcntl.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn os_fgetfl(fd: usize) -> isize {
    unsafe { syscall!(FCNTL, fd, libc::F_GETFL, 0) as isize }
}

//https://man7.org/linux/man-pages/man2/fcntl.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn os_fsetfl(fd: usize, flags: libc::c_int) -> isize {
    unsafe { syscall!(FCNTL, fd, libc::F_SETFL, flags) as isize }
}

//https://man7.org/linux/man-pages/man2/ftruncate.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn os_ftruncate(fd: usize, length: libc::off_t) -> isize {
    unsafe { syscall!(FTRUNCATE, fd, length) as isize }
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
) -> isize {
    unsafe {
        syscall!(
            LINKAT,
            old_fd,
            old_path.as_ptr(),
            new_fd,
            new_path.as_ptr(),
            flags
        ) as isize
    }
}

//https://man7.org/linux/man-pages/man2/mkdirat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::PathAccess))]
pub fn os_mkdirat(dir_fd: usize, pathname: Vec<u8>, mode: libc::mode_t) -> isize {
    unsafe { syscall!(MKDIRAT, dir_fd, pathname.as_ptr(), mode) as isize }
}

//https://man7.org/linux/man-pages/man2/readlinkat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(buf.len() >= cnt)]
#[ensures(result >= 0 ==> buf.len() == result as usize)]
#[ensures(result >= 0 ==> result as usize <= cnt)]
#[trusted]
#[ensures(three_effects!(old(trace), trace, Effect::FdAccess, Effect::PathAccess, Effect::WriteN(count) if count == cnt))]
pub fn os_readlinkat(dir_fd: usize, pathname: Vec<u8>, buf: &mut [u8], cnt: usize) -> isize {
    unsafe { syscall!(READLINKAT, dir_fd, pathname.as_ptr(), buf.as_mut_ptr(), cnt) as isize }
}

//https://man7.org/linux/man-pages/man2/unlinkat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::PathAccess))]
pub fn os_unlinkat(dir_fd: usize, pathname: Vec<u8>, flags: libc::c_int) -> isize {
    unsafe { syscall!(UNLINKAT, dir_fd, pathname.as_ptr(), flags) as isize }
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
) -> isize {
    unsafe {
        syscall!(
            RENAMEAT,
            old_dir_fd,
            old_pathname.as_ptr(),
            new_dir_fd,
            new_pathname.as_ptr()
        ) as isize
    }
}

//https://man7.org/linux/man-pages/man2/symlinkat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(three_effects!(old(trace), trace,  Effect::PathAccess, Effect::FdAccess, Effect::PathAccess))]
pub fn os_symlinkat(old_pathname: Vec<u8>, dir_fd: usize, new_pathname: Vec<u8>) -> isize {
    unsafe {
        syscall!(
            SYMLINKAT,
            old_pathname.as_ptr(),
            dir_fd,
            new_pathname.as_ptr()
        ) as isize
    }
}

//https://man7.org/linux/man-pages/man2/utimensat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(specs.capacity() >= 2)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, Effect::FdAccess))]
pub fn os_futimens(fd: usize, specs: &Vec<libc::timespec>) -> isize {
    // Linux impls futimens as UTIMENSAT with null path
    // source: https://code.woboq.org/userspace/glibc/sysdeps/unix/sysv/linux/futimens.c.html
    unsafe { syscall!(UTIMENSAT, fd, 0, specs.as_ptr(), 0) as isize }
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
) -> isize {
    unsafe { syscall!(UTIMENSAT, fd, pathname.as_ptr(), specs.as_ptr(), flags) as isize }
}

//https://man7.org/linux/man-pages/man2/clock_gettime.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(no_effect!(old(trace), trace))]
pub fn os_clock_get_time(clock_id: libc::clockid_t, spec: &mut libc::timespec) -> isize {
    unsafe { syscall!(CLOCK_GETTIME, clock_id, spec as *mut libc::timespec) as isize }
}

//https://man7.org/linux/man-pages/man2/clock_getres.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(no_effect!(old(trace), trace))]
pub fn os_clock_get_res(clock_id: libc::clockid_t, spec: &mut libc::timespec) -> isize {
    unsafe { syscall!(CLOCK_GETRES, clock_id, spec as *mut libc::timespec) as isize }
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(buf.len() >= cnt)]
#[ensures(result >= 0 ==> buf.len() >= result as usize)]
#[ensures(result >= 0 ==> result as usize <= cnt)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, Effect::WriteN(count) if count == cnt))]
pub fn os_getrandom(buf: &mut [u8], cnt: usize, flags: u32) -> isize {
    unsafe { syscall!(GETRANDOM, buf.as_mut_ptr(), cnt, flags) as isize }
}

//https://man7.org/linux/man-pages/man2/recvfrom.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(buf.len() >= cnt)]
#[ensures(result >= 0 ==> buf.len() >= result as usize)]
#[ensures(result >= 0 ==> result as usize <= cnt)]
#[trusted]
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::WriteN(count) if count == cnt))]
pub fn os_recv(fd: usize, buf: &mut [u8], cnt: usize, flags: u32) -> isize {
    unsafe { syscall!(RECVFROM, fd, buf.as_mut_ptr(), cnt, flags, 0, 0) as isize }
}

//https://man7.org/linux/man-pages/man2/sendto.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(buf.len() >= cnt)]
#[trusted]
#[ensures(two_effects!(old(trace), trace, Effect::FdAccess, Effect::ReadN(count) if count == cnt))]
pub fn os_send(fd: usize, buf: &[u8], cnt: usize, flags: u32) -> isize {
    unsafe { syscall!(SENDTO, fd, buf.as_ptr(), cnt, flags, 0, 0) as isize }
}

//https://man7.org/linux/man-pages/man2/shutdown.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(two_effects!(old(trace), trace, Effect::Shutdown, Effect::FdAccess))]
pub fn os_shutdown(fd: usize, how: libc::c_int) -> isize {
    unsafe { syscall!(SHUTDOWN, fd, how) as isize }
}

//https://man7.org/linux/man-pages/man2/nanosleep.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(no_effect!(old(trace), trace))]
pub fn os_nanosleep(req: &libc::timespec, rem: &mut libc::timespec) -> isize {
    unsafe {
        syscall!(
            NANOSLEEP,
            req as *const libc::timespec,
            rem as *mut libc::timespec
        ) as isize
    }
}

//https://man7.org/linux/man-pages/man2/poll.2.html
// can make more efficient using slice of pollfds
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(no_effect!(old(trace), trace))]
pub fn os_poll(pollfd: &mut libc::pollfd, timeout: libc::c_int) -> isize {
    unsafe { syscall!(POLL, pollfd as *const libc::pollfd, 1, timeout) as isize }
}

//https://man7.org/linux/man-pages/man2/getdents64.2.html
//  long syscall(SYS_getdents, unsigned int fd, struct linux_dirent *dirp, unsigned int count);
#[with_ghost_var(trace: &mut Trace)]
#[external_method(set_len)]
#[trusted]
#[ensures(no_effect!(old(trace), trace))]
// TODO: what effect should this have?
//#[ensures(no_effect!(old(trace), trace))]
pub fn os_getdents64(fd: usize, dirp: &mut Vec<u8>, count: usize) -> isize {
    unsafe {
        let result = syscall!(GETDENTS64, fd, dirp.as_mut_ptr(), count);
        dirp.set_len(result);
        result as isize
    }
}

//https://man7.org/linux/man-pages/man2/socket.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(no_effect!(old(trace), trace))]
// TODO: finish spec
//#[ensures(two_effects!(old(trace), trace, Effect::Shutdown, Effect::FdAccess))]
pub fn os_socket(domain: i32, ty: i32, protocol: i32) -> isize {
    unsafe { syscall!(SOCKET, domain, ty, protocol) as isize }
}

//https://man7.org/linux/man-pages/man2/connect.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(no_effect!(old(trace), trace))]
// TODO: finish spec
// #[ensures(two_effects!(old(trace), trace, Effect::Shutdown, Effect::FdAccess))]
pub fn os_connect(sockfd: usize, addr: &libc::sockaddr_in, addrlen: u32) -> isize {
    unsafe { syscall!(CONNECT, sockfd, addr as *const libc::sockaddr_in, addrlen) as isize }
}
