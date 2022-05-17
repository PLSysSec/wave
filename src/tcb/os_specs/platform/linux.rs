#[cfg(feature = "time_syscalls")]
use crate::stats::timing::{push_syscall_result, start_timer, stop_timer};
use crate::tcb::misc::flag_set;
use crate::tcb::sbox_mem::raw_ptr;
use crate::types::NativeIoVecs;
use crate::iov::*;
#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
#[cfg(not(feature = "time_syscalls"))]
use crate::verifier_interface::{push_syscall_result, start_timer, stop_timer};
use crate::{effect, effects, path_effect};
use prusti_contracts::*;
use syscall::syscall;
use wave_macros::{external_call, external_method, with_ghost_var};

//https://man7.org/linux/man-pages/man2/pread.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(buf.len() >= cnt)]
#[trusted]
#[ensures(old(raw_ptr(buf)) == raw_ptr(buf))]
#[ensures(effects!(old(trace), trace, effect!(FdAccess), effect!(WriteN, addr, count) if addr == old(raw_ptr(buf)) && count == cnt))]
pub fn os_pread(fd: usize, buf: &mut [u8], cnt: usize, offset: usize) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(PREAD64, fd, buf.as_mut_ptr(), cnt, offset) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("pread", __start_ts, __end_ts);
    result
}


#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(
    trace.len() == old(trace.len() + buf.len()) &&
    forall(|i: usize| (i < trace.len()) ==> 
    {
        if i < old(trace.len()) 
            { trace.lookup(i) == old(trace.lookup(i)) }
        else
        {
            let this = buf.lookup(i - old(trace.len())); 
            let ev = trace.lookup(i);
            iov_eq_write(ev, &this)
        }
    }
)
)]
pub fn os_preadv(fd: usize, buf: &NativeIoVecs, iovcnt: usize, offset: usize) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(PREADV, fd, buf.iovs.as_ptr(), iovcnt, offset) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("preadv", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/pwrite.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(buf.len() >= cnt)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess), effect!(ReadN, addr, count) if addr == old(raw_ptr(buf)) && count == cnt))]
pub fn os_pwrite(fd: usize, buf: &[u8], cnt: usize, offset: usize) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(PWRITE64, fd, buf.as_ptr(), cnt, offset) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("pwrite", __start_ts, __end_ts);
    result
}


//man7.org/linux/man-pages/man2/pwritev.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(
    trace.len() == old(trace.len() + buf.len()) &&
    forall(|i: usize| (i < trace.len()) ==> 
    {
        if i < old(trace.len()) 
            { trace.lookup(i) == old(trace.lookup(i)) }
        else
        {
            let this = buf.lookup(i - old(trace.len())); 
            let ev = trace.lookup(i);
            iov_eq_read(ev, &this)
        }
    }
)
)]
pub fn os_pwritev(fd: usize, buf: &NativeIoVecs, iovcnt: usize, offset: usize) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(PWRITEV, fd, buf.iovs.as_ptr(), iovcnt, offset) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("pwritev", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/fadvise64.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
pub fn os_advise(fd: usize, offset: i64, len: i64, advice: i32) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(FADVISE64, fd, offset, len, advice) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("advise", __start_ts, __end_ts);
    result
}

// https://man7.org/linux/man-pages/man2/fallocate.2.html
// hardcode mode to 0 to behave more like posix_fallocate
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
pub fn os_allocate(fd: usize, offset: i64, len: i64) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(FALLOCATE, fd, 0, offset, len) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("allocate", __start_ts, __end_ts);
    result
}

// https://man7.org/linux/man-pages/man2/fstatat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
// follows terminal symlink if O_NOFOLLOW are not set
// this is the only lookupflags, so we just say flags == 0
#[ensures(effects!(old(trace), trace, effect!(FdAccess), 
path_effect!(PathAccessAt, fd, p, f) if fd == dirfd && p == old(path) && f == !flag_set(flags, libc::AT_SYMLINK_NOFOLLOW))
)]
// path_effect!(PathAccessAt, fd1, old_p, true) if fd1 == old_fd && old_p == old(old_path)
pub fn os_fstatat(dirfd: usize, path: [u8; 4096], stat: &mut libc::stat, flags: i32) -> isize {
    let __start_ts = start_timer();
    let result = unsafe {
        syscall!(
            NEWFSTATAT,
            dirfd,
            path.as_ptr(),
            stat as *mut libc::stat,
            flags
        ) as isize
    };
    let __end_ts = stop_timer();
    push_syscall_result("fstatat", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/utimensat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(specs.len() >= 2)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
pub fn os_futimens(fd: usize, specs: &Vec<libc::timespec>) -> isize {
    let __start_ts = start_timer();
    // Linux impls futimens as UTIMENSAT with null path
    // source: https://code.woboq.org/userspace/glibc/sysdeps/unix/sysv/linux/futimens.c.html
    let result = unsafe { syscall!(UTIMENSAT, fd, 0, specs.as_ptr(), 0) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("futimens", __start_ts, __end_ts);
    result
}

// https://man7.org/linux/man-pages/man2/utimensat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(specs.len() >= 2)]
#[trusted]
// follows terminal symlink if O_NOFOLLOW are not set
// this is the only lookupflags, so we just say flags == 0
#[ensures(effects!(old(trace), trace, effect!(FdAccess), path_effect!(PathAccessAt, fd, p, f) if fd == dirfd && p == old(path) && f == !flag_set(flags, libc::AT_SYMLINK_NOFOLLOW) ))]
pub fn os_utimensat(
    dirfd: usize,
    path: [u8; 4096],
    specs: &Vec<libc::timespec>,
    flags: libc::c_int,
) -> isize {
    let __start_ts = start_timer();
    let result =
        unsafe { syscall!(UTIMENSAT, dirfd, path.as_ptr(), specs.as_ptr(), flags) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("utimensat", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/clock_gettime.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(effects!(old(trace), trace))]
pub fn os_clock_get_time(clock_id: libc::clockid_t, spec: &mut libc::timespec) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(CLOCK_GETTIME, clock_id, spec as *mut libc::timespec) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("clock_get_time", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/clock_getres.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(effects!(old(trace), trace))]
pub fn os_clock_get_res(clock_id: libc::clockid_t, spec: &mut libc::timespec) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(CLOCK_GETRES, clock_id, spec as *mut libc::timespec) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("clock_get_res", __start_ts, __end_ts);
    result
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(buf.len() >= cnt)]
#[trusted]
#[ensures(old(raw_ptr(buf)) == raw_ptr(buf))]
#[ensures(effects!(old(trace), trace, effect!(WriteN, addr, count) if addr == old(raw_ptr(buf)) && count == cnt))]
pub fn os_getrandom(buf: &mut [u8], cnt: usize, flags: u32) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(GETRANDOM, buf.as_mut_ptr(), cnt, flags) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("getrandom", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/nanosleep.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(effects!(old(trace), trace))]
pub fn os_nanosleep(req: &libc::timespec, rem: &mut libc::timespec) -> isize {
    let __start_ts = start_timer();
    let result = unsafe {
        syscall!(
            NANOSLEEP,
            req as *const libc::timespec,
            rem as *mut libc::timespec
        ) as isize
    };
    let __end_ts = stop_timer();
    push_syscall_result("nanosleep", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/getdents64.2.html
#[with_ghost_var(trace: &mut Trace)]
#[external_method(set_len)]
#[trusted]
#[requires(dirp.capacity() >= count)]
#[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
pub fn os_getdents64(fd: usize, dirp: &mut Vec<u8>, count: usize) -> isize {
    let __start_ts = start_timer();
    let result = unsafe {
        let result = syscall!(GETDENTS64, fd, dirp.as_mut_ptr(), count);
        if (result as isize) != -1 {
            dirp.set_len(result);
        } else {
            dirp.set_len(0);
        }
        result as isize
    };
    let __end_ts = stop_timer();
    push_syscall_result("getdents64", __start_ts, __end_ts);
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
