#[cfg(feature = "time_syscalls")]
use crate::stats::timing::{push_syscall_result, start_timer, stop_timer};
use crate::tcb::misc::flag_set;
use crate::tcb::sbox_mem::raw_ptr;
#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
#[cfg(not(feature = "time_syscalls"))]
use crate::verifier_interface::{push_syscall_result, start_timer, stop_timer};
use crate::{effect, effects, path_effect, map_effects};
use prusti_contracts::*;
use syscall::syscall;
use wave_macros::{external_call, external_method, with_ghost_var};
use crate::types::{NativeIoVec, NativeIoVecs};

#[cfg_attr(target_os = "linux", path = "platform/linux.rs")]
#[cfg_attr(target_os = "macos", path = "platform/mac.rs")]
mod platform;
pub use platform::*;


#[cfg(feature = "verify")]
predicate! {
    pub fn iov_eq(ev: Effect, iov: &NativeIoVec) -> bool {
        match ev {
            effect!(ReadN,addr,count) => 
                addr == iov.iov_base && 
                count == iov.iov_len,
            _ => false,
        }
    }
}


// https://man7.org/linux/man-pages/man2/open.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
// follows terminal sylink if O_NOFOLLOW is not set
#[ensures(effects!(old(trace), trace, path_effect!(PathAccessAt, fd, p, f) if fd == dirfd && p == old(path) && f == !flag_set(flags, libc::O_NOFOLLOW) ))]
pub fn os_openat(dirfd: usize, path: [u8; 4096], flags: i32) -> isize {
    let __start_ts = start_timer();
    // all created files should be rdwr
    let result = unsafe { syscall!(OPENAT, dirfd, path.as_ptr(), flags, 0o666) as isize };
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
#[trusted]
#[ensures(old(raw_ptr(buf)) == raw_ptr(buf))]
#[ensures(effects!(old(trace), trace, effect!(FdAccess), effect!(WriteN, addr, count) if addr == old(raw_ptr(buf)) && count == cnt))]
pub fn os_read(fd: usize, buf: &mut [u8], cnt: usize) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(READ, fd, buf.as_mut_ptr(), cnt) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("read", __start_ts, __end_ts);
    result
}

// https://man7.org/linux/man-pages/man2/read.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
// #[ensures(effects!(old(trace), trace))]
// #[ensures(
//     map_effects!(
//         old(trace), 
//         trace, 
//         buf, 
//         iovcnt, 
//         effect!(WriteN, addr, count) if addr == this.iov_base && count == iovcnt
// ))]
// #[ensures(
//     old(buf.len()) == iovcnt && 
//     takes_n_steps(old(trace), trace, iovcnt) &&
//     forall(|idx: usize|  (idx < old(buf.len())) ==> {
//         let this = old(buf.lookup(idx)); 
//         match trace.lookup(trace.len() - old(buf.len()) + idx) {
//             effect!(WriteN,addr,count) => 
//                 addr == this.iov_base && 
//                 count == this.iov_len,
//             _ => false,
//         }
//     })
// )]

// // #[ensures(old(raw_ptr(buf)) == raw_ptr(buf))]
// #[ensures(
//     buf.len() == old(buf.len()) && 
//     forall(|idx: usize|  (idx < buf.len()) ==>
//         buf.lookup(idx) == old(buf.lookup(idx))
//     // self.fits_in_lin_mem_usize(iov.iov_base, iov.iov_len, trace)
//     // })
// ))]





#[ensures(
    // effects_from_iov(trace, buf)
    trace.len() == old(trace.len() + buf.len()) &&
    forall(|i: usize| (i < trace.len()) ==> 
    {
        if i < old(trace.len()) 
            { trace.lookup(i) == old(trace.lookup(i)) }
        else
        {
            let this = buf.lookup(i - old(trace.len())); 
            let ev = trace.lookup(i);
            iov_eq(ev, &this)
        }
    }
)
)]
// #[ensures(effects!(old(trace), trace))]

pub fn os_readv(fd: usize, buf: &NativeIoVecs, iovcnt: usize) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(READV, fd, buf.iovs.as_ptr(), iovcnt) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("readv", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/write.2.html
#[with_ghost_var(trace: &mut Trace)]
//#[requires(buf.len() >= cnt)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess), effect!(ReadN, addr, count) if addr == old(raw_ptr(buf)) && count == cnt))]
pub fn os_write(fd: usize, buf: &[u8], cnt: usize) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(WRITE, fd, buf.as_ptr(), cnt) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("write", __start_ts, __end_ts);
    result
}

//man7.org/linux/man-pages/man2/read.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(
    // effects_from_iov(trace, buf)
    trace.len() == old(trace.len() + buf.len()) &&
    forall(|i: usize| (i < trace.len()) ==> 
    {
        if i < old(trace.len()) 
            { trace.lookup(i) == old(trace.lookup(i)) }
        else
        {
            let this = buf.lookup(i - old(trace.len())); 
            let ev = trace.lookup(i);
            iov_eq(ev, &this)
        }
    }
)
)]
// #[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
// #[ensures(effects!(old(trace), trace, effect!(FdAccess), effect!(WriteN, addr, count) if addr == old(raw_ptr(buf)) && count == cnt))]
pub fn os_writev(fd: usize, buf: &NativeIoVecs, iovcnt: usize) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(WRITEV, fd, buf.iovs.as_ptr(), iovcnt) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("writev", __start_ts, __end_ts);
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

// https://man7.org/linux/man-pages/man2/linkat.2.html
// follows terminal symlink: true
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess), 
    effect!(FdAccess), 
    path_effect!(PathAccessAt, fd1, old_p, f) if fd1 == old_fd && old_p == old(old_path) && f == flag_set(flags, libc::AT_SYMLINK_FOLLOW), 
    path_effect!(PathAccessAt, fd2, new_p, f) if fd2 == new_fd && new_p == old(new_path) && f == flag_set(flags, libc::AT_SYMLINK_FOLLOW) 
))]
pub fn os_linkat(
    old_fd: usize,
    old_path: [u8; 4096],
    new_fd: usize,
    new_path: [u8; 4096],
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

// https://man7.org/linux/man-pages/man2/mkdirat.2.html
// follows terminal symlink: true
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess), path_effect!(PathAccessAt, fd, p, true) if fd == dirfd && p == old(path) ))]
pub fn os_mkdirat(dirfd: usize, path: [u8; 4096], mode: libc::mode_t) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(MKDIRAT, dirfd, path.as_ptr(), mode) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("mkdirat", __start_ts, __end_ts);
    result
}

// https://man7.org/linux/man-pages/man2/readlinkat.2.html
// follows terminal symlink: false
#[with_ghost_var(trace: &mut Trace)]
#[requires(buf.len() >= cnt)]
#[trusted]
#[ensures(old(raw_ptr(buf)) == raw_ptr(buf))]
#[ensures(effects!(old(trace), trace, effect!(FdAccess), path_effect!(PathAccessAt, fd, p, false) if fd == dirfd && p == old(path), effect!(WriteN, addr, count) if addr == old(raw_ptr(buf)) && count == cnt))]
pub fn os_readlinkat(dirfd: usize, path: [u8; 4096], buf: &mut [u8], cnt: usize) -> isize {
    let __start_ts = start_timer();
    let result =
        unsafe { syscall!(READLINKAT, dirfd, path.as_ptr(), buf.as_mut_ptr(), cnt) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("readlinkat", __start_ts, __end_ts);
    result
}

// https://man7.org/linux/man-pages/man2/unlinkat.2.html
// follows terminal symlink: false
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess), path_effect!(PathAccessAt, fd, p, false) if fd == dirfd && p == old(path)))]
pub fn os_unlinkat(dirfd: usize, path: [u8; 4096], flags: libc::c_int) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(UNLINKAT, dirfd, path.as_ptr(), flags) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("unlinkat", __start_ts, __end_ts);
    result
}
//https://man7.org/linux/man-pages/man2/renameat.2.html
// follows terminal symlinks: false
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess), path_effect!(PathAccessAt, fd1, old_p, false) if fd1 == old_dir_fd && old_p == old(old_path), effect!(FdAccess), path_effect!(PathAccessAt, fd2, new_p, false) if fd2 == new_dir_fd && new_p == old(new_path)))]
pub fn os_renameat(
    old_dir_fd: usize,
    old_path: [u8; 4096],
    new_dir_fd: usize,
    new_path: [u8; 4096],
) -> isize {
    let __start_ts = start_timer();
    let result = unsafe {
        syscall!(
            RENAMEAT,
            old_dir_fd,
            old_path.as_ptr(),
            new_dir_fd,
            new_path.as_ptr()
        ) as isize
    };
    let __end_ts = stop_timer();
    push_syscall_result("renameat", __start_ts, __end_ts);
    result
}

// https://man7.org/linux/man-pages/man2/symlinkat.2.html
// From the spec: The string pointed to by path1 shall be treated only as a string and shall not be validated as a pathname.
// follows terminal symlinks: true (although it might fail)
// TODO: do we actually need to check the second path or can we just let the resolver do its thing?
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(effects!(old(trace), trace, path_effect!(PathAccessAt, fd, p, true) if fd == dirfd && p == old(path2), effect!(FdAccess)))]
pub fn os_symlinkat(path1: [u8; 4096], dirfd: usize, path2: [u8; 4096]) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(SYMLINKAT, path1.as_ptr(), dirfd, path2.as_ptr()) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("symlinkat", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/recvfrom.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(buf.len() >= cnt)]
#[trusted]
#[ensures(old(raw_ptr(buf)) == raw_ptr(buf))]
#[ensures(effects!(old(trace), trace, effect!(FdAccess), effect!(WriteN, addr, count) if addr == old(raw_ptr(buf)) && count == cnt))]
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
#[ensures(effects!(old(trace), trace, effect!(FdAccess), effect!(ReadN, addr, count) if addr == old(raw_ptr(buf)) && count == cnt))]
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
