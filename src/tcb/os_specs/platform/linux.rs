use crate::iov::*;
#[cfg(feature = "time_syscalls")]
use crate::stats::timing::{push_syscall_result, start_timer, stop_timer};
use crate::syscall_spec_gen;
use crate::tcb::misc::flag_set;
use crate::tcb::sbox_mem::raw_ptr;
#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::types::NativeIoVecs;
#[cfg(not(feature = "time_syscalls"))]
use crate::verifier_interface::{push_syscall_result, start_timer, stop_timer};
use crate::{effect, effects, path_effect};
use prusti_contracts::*;
use syscall::syscall;
use wave_macros::{external_call, external_method, with_ghost_var};

use paste::paste;

//https://man7.org/linux/man-pages/man2/pread.2.html
syscall_spec_gen! {
    trace;
    requires((buf.len() >= cnt));
    ensures((effects!(old(trace), trace, effect!(FdAccess), effect!(WriteN, addr, count) if addr == old(raw_ptr(buf)) && count == cnt)));
    ensures((old(raw_ptr(buf)) == raw_ptr(buf)));
    syscall(pread64 ALIAS pread, fd: usize, buf: (&mut [u8]), cnt: usize, offset: usize)
}

//https://man7.org/linux/man-pages/man2/pwrite.2.html
syscall_spec_gen! {
    trace;
    requires((buf.len() >= cnt));
    ensures((effects!(old(trace), trace, effect!(FdAccess), effect!(ReadN, addr, count) if addr == old(raw_ptr(buf)) && count == cnt)));
    syscall(pwrite64 ALIAS pwrite, fd: usize, buf: (&[u8]), cnt: usize, offset: usize)
}

//https://man7.org/linux/man-pages/man2/fadvise64.2.html
syscall_spec_gen! {
    trace;
    ensures((effects!(old(trace), trace, effect!(FdAccess))));
    syscall(fadvise64, fd: usize, offset: i64, len: i64, advice: i32)
}

// https://man7.org/linux/man-pages/man2/fallocate.2.html
// hardcode mode to 0 to behave more like posix_fallocate
syscall_spec_gen! {
    trace;
    ensures((effects!(old(trace), trace, effect!(FdAccess))));
    syscall(fallocate, fd: usize, mode: i32, offset: i64, len: i64)
}

// https://man7.org/linux/man-pages/man2/fstatat.2.html
// follows terminal symlink if O_NOFOLLOW are not set
// this is the only lookupflags, so we just say flags == 0
syscall_spec_gen! {
    trace;
    ensures((effects!(old(trace), trace, effect!(FdAccess), path_effect!(PathAccessAt, fd, p, f) if fd == dirfd && p == old(path) && f == !flag_set(flags, libc::AT_SYMLINK_NOFOLLOW))));
    syscall(newfstatat ALIAS fstatat, dirfd: usize, path: [u8; 4096], stat: (&mut libc::stat), flags: i32)
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
syscall_spec_gen! {
    trace;
    requires((specs.len() >= 2));
    ensures((effects!(old(trace), trace, effect!(FdAccess), path_effect!(PathAccessAt, fd, p, f) if fd == dirfd && p == old(path) && f == !flag_set(flags, libc::AT_SYMLINK_NOFOLLOW))));
    syscall(utimensat, dirfd: usize, path: [u8; 4096], specs: (&Vec<libc::timespec>), flags: (libc::c_int))
}

//https://man7.org/linux/man-pages/man2/clock_gettime.2.html
syscall_spec_gen! {
    trace;
    ensures((effects!(old(trace), trace)));
    syscall(clock_gettime, clock_id: (libc::clockid_t), spec: (&mut libc::timespec))
}

//https://man7.org/linux/man-pages/man2/clock_getres.2.html
syscall_spec_gen! {
    trace;
    ensures((effects!(old(trace), trace)));
    syscall(clock_getres, clock_id: (libc::clockid_t), spec: (&mut libc::timespec))
}

syscall_spec_gen! {
    trace;
    requires((buf.len() >= cnt));
    ensures((effects!(old(trace), trace, effect!(WriteN, addr, count) if addr == old(raw_ptr(buf)) && count == cnt)));
    ensures((old(raw_ptr(buf)) == raw_ptr(buf)));
    syscall(getrandom, buf: (&mut [u8]), cnt: usize, flags: u32)
}

//https://man7.org/linux/man-pages/man2/nanosleep.2.html
syscall_spec_gen! {
    trace;
    ensures((effects!(old(trace), trace)));
    syscall(nanosleep, req: (&libc::timespec), rem: (&mut libc::timespec))
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
syscall_spec_gen! {
    trace;
    ensures((effects!(old(trace), trace, effect!(FdAccess))));
    syscall(fstat, fd: usize, stat: (&mut libc::stat))
}
