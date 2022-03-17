#[cfg(feature = "time_syscalls")]
use crate::stats::timing::{push_syscall_result, start_timer, stop_timer};
use crate::tcb::sbox_mem::as_sbox_ptr;
#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
#[cfg(not(feature = "time_syscalls"))]
use crate::verifier_interface::{push_syscall_result, start_timer, stop_timer};
use crate::{effect, effects};
use wave_macros::{external_call, external_method, with_ghost_var};
use prusti_contracts::*;
use syscall::syscall;

#[cfg_attr(target_os = "linux",
           path="platform/linux.rs")]
#[cfg_attr(target_os = "macos",
           path="platform/mac.rs")]
mod platform;
pub use platform::*;

//https://man7.org/linux/man-pages/man2/open.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(PathAccessAt, dirfd)))]
pub fn os_openat(dirfd: usize, pathname: Vec<u8>, flags: i32) -> isize {
    let __start_ts = start_timer();
    // all created files should be rdwr
    let result = unsafe { syscall!(OPENAT, dirfd, pathname.as_ptr(), flags, 0o666) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("openat", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/close.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
pub fn os_close(fd: usize) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(CLOSE, fd) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("close", __start_ts, __end_ts);
    result
}

// https://man7.org/linux/man-pages/man2/read.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(buf.len() >= cnt)]
#[ensures(result >= 0 ==> buf.len() >= result as usize)]
#[ensures(result >= 0 ==> result as usize <= cnt)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess), effect!(WriteN, addr, count) if addr == old(as_sbox_ptr(buf)) && count == cnt))]
pub fn os_read(fd: usize, buf: &mut [u8], cnt: usize) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(READ, fd, buf.as_mut_ptr(), cnt) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("read", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/write.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(buf.len() >= cnt)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess), effect!(ReadN, addr, count) if addr == old(as_sbox_ptr(buf)) && count == cnt))]
pub fn os_write(fd: usize, buf: &[u8], cnt: usize) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(WRITE, fd, buf.as_ptr(), cnt) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("write", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/lseek.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
pub fn os_seek(fd: usize, offset: i64, whence: i32) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(LSEEK, fd, offset, whence) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("seek", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/fsync.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
pub fn os_sync(fd: usize) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(FSYNC, fd) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("sync", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/fdatasync.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
pub fn os_datasync(fd: usize) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(FDATASYNC, fd) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("datasync", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/fstat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
pub fn os_fstat(fd: usize, stat: &mut libc::stat) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(FSTAT, fd, stat as *mut libc::stat) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("fstat", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/fcntl.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
pub fn os_fgetfl(fd: usize) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(FCNTL, fd, libc::F_GETFL, 0) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("fgetfl", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/fcntl.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
pub fn os_fsetfl(fd: usize, flags: libc::c_int) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(FCNTL, fd, libc::F_SETFL, flags) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("fsetfl", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/ftruncate.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
pub fn os_ftruncate(fd: usize, length: libc::off_t) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(FTRUNCATE, fd, length) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("ftruncate", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/linkat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess), effect!(FdAccess), effect!(PathAccessAt, old_fd), effect!(PathAccessAt, new_fd)))]
pub fn os_linkat(
    old_fd: usize,
    old_path: Vec<u8>,
    new_fd: usize,
    new_path: Vec<u8>,
    flags: i32,
) -> isize {
    let __start_ts = start_timer();
    let result = unsafe {
        syscall!(
            LINKAT,
            old_fd,
            old_path.as_ptr(),
            new_fd,
            new_path.as_ptr(),
            flags
        ) as isize
    };
    let __end_ts = stop_timer();
    push_syscall_result("linkat", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/mkdirat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccessAt, dir_fd)))]
pub fn os_mkdirat(dir_fd: usize, pathname: Vec<u8>, mode: libc::mode_t) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(MKDIRAT, dir_fd, pathname.as_ptr(), mode) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("mkdirat", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/readlinkat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(buf.len() >= cnt)]
#[ensures(result >= 0 ==> buf.len() == result as usize)]
#[ensures(result >= 0 ==> result as usize <= cnt)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccessAt, dir_fd), effect!(WriteN, addr, count) if addr == old(as_sbox_ptr(buf)) && count == cnt))]
pub fn os_readlinkat(dir_fd: usize, pathname: Vec<u8>, buf: &mut [u8], cnt: usize) -> isize {
    let __start_ts = start_timer();
    let result =
        unsafe { syscall!(READLINKAT, dir_fd, pathname.as_ptr(), buf.as_mut_ptr(), cnt) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("readlinkat", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/unlinkat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccessAt, dir_fd)))]
pub fn os_unlinkat(dir_fd: usize, pathname: Vec<u8>, flags: libc::c_int) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(UNLINKAT, dir_fd, pathname.as_ptr(), flags) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("unlinkat", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/renameat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccessAt, old_dir_fd), effect!(FdAccess), effect!(PathAccessAt, new_dir_fd)))]
pub fn os_renameat(
    old_dir_fd: usize,
    old_pathname: Vec<u8>,
    new_dir_fd: usize,
    new_pathname: Vec<u8>,
) -> isize {
    let __start_ts = start_timer();
    let result = unsafe {
        syscall!(
            RENAMEAT,
            old_dir_fd,
            old_pathname.as_ptr(),
            new_dir_fd,
            new_pathname.as_ptr()
        ) as isize
    };
    let __end_ts = stop_timer();
    push_syscall_result("renameat", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/symlinkat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(effects!(old(trace), trace,  effect!(PathAccessAt, dir_fd), effect!(FdAccess)))]
pub fn os_symlinkat(old_pathname: Vec<u8>, dir_fd: usize, new_pathname: Vec<u8>) -> isize {
    let __start_ts = start_timer();
    let result = unsafe {
        syscall!(
            SYMLINKAT,
            old_pathname.as_ptr(),
            dir_fd,
            new_pathname.as_ptr()
        ) as isize
    };
    let __end_ts = stop_timer();
    push_syscall_result("symlinkat", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/recvfrom.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(buf.len() >= cnt)]
#[ensures(result >= 0 ==> buf.len() >= result as usize)]
#[ensures(result >= 0 ==> result as usize <= cnt)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess), effect!(WriteN, addr, count) if addr == old(as_sbox_ptr(buf)) && count == cnt))]
pub fn os_recv(fd: usize, buf: &mut [u8], cnt: usize, flags: i32) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(RECVFROM, fd, buf.as_mut_ptr(), cnt, flags, 0, 0) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("recv", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/sendto.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(buf.len() >= cnt)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess), effect!(ReadN, addr, count) if addr == old(as_sbox_ptr(buf)) && count == cnt))]
pub fn os_send(fd: usize, buf: &[u8], cnt: usize, flags: i32) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(SENDTO, fd, buf.as_ptr(), cnt, flags, 0, 0) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("send", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/shutdown.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(Shutdown), effect!(FdAccess)))]
pub fn os_shutdown(fd: usize, how: libc::c_int) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(SHUTDOWN, fd, how) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("shutdown", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/poll.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
pub fn os_poll(pollfds: &mut [libc::pollfd], timeout: libc::c_int) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(POLL, pollfds.as_mut_ptr(), pollfds.len(), timeout) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("poll", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/socket.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(SockCreation, d, t) if d == (domain as usize) && t == (ty as usize) ))]
pub fn os_socket(domain: i32, ty: i32, protocol: i32) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(SOCKET, domain, ty, protocol) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("socket", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/connect.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
// TODO: finish spec
#[ensures(effects!(old(trace), trace, effect!(FdAccess), effect!(NetAccess, protocol, ip, port) if ip == addr.sin_addr.s_addr as usize && port == addr.sin_port as usize))]
pub fn os_connect(sockfd: usize, addr: &libc::sockaddr_in, addrlen: u32) -> isize {
    let __start_ts = start_timer();
    let result =
        unsafe { syscall!(CONNECT, sockfd, addr as *const libc::sockaddr_in, addrlen) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("connect", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/ioctl.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
// TODO: finish spec
#[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
pub fn os_fionread(fd: usize) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(IOCTL, fd, libc::FIONREAD) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("fionread", __start_ts, __end_ts);
    result
}

