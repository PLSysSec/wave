use crate::tcb::misc::flag_set;
use crate::tcb::sbox_mem::raw_ptr;
#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::verifier_interface::{push_syscall_result, start_timer, stop_timer};
use crate::{effect, effects, path_effect};
use core::ffi::c_void;
use libc::{__error, utimensat};
use prusti_contracts::*;
use syscall::syscall;
use wave_macros::{external_calls, external_methods, with_ghost_var};
use crate::syscall_spec_gen;

use mach2::mach_time::{
    mach_absolute_time, mach_timebase_info, mach_timebase_info_data_t, mach_timebase_info_t,
    mach_wait_until,
};
use security_framework_sys::random::{kSecRandomDefault, SecRandomCopyBytes};

//https://man7.org/linux/man-pages/man2/pread.2.html
syscall_spec_gen! {
    trace;
    requires((buf.len() >= cnt));
    ensures((effects!(old(trace), trace, effect!(FdAccess), effect!(WriteMem, addr, count) if addr == old(raw_ptr(buf)) && count == cnt)));
    ensures((old(raw_ptr(buf)) == raw_ptr(buf)));
    syscall(pread, fd: usize, buf: &mut [u8], cnt: usize, offset: usize)
}

//https://man7.org/linux/man-pages/man2/pwrite.2.html
syscall_spec_gen! {
    trace;
    requires((buf.len() >= cnt));
    ensures((effects!(old(trace), trace, effect!(FdAccess), effect!(ReadMem, addr, count) if addr == old(raw_ptr(buf)) && count == cnt)));
    syscall(pwrite, fd: usize, buf: &[u8], cnt: usize, offset: usize)
}

syscall_spec_gen! {
    trace;
    ensures((effects!(old(trace), trace, effect!(FdAccess))));
    syscall(allocate, fd: usize, fstore: &libc::fstore_t)
}

// https://man7.org/linux/man-pages/man2/fstatat.2.html
syscall_spec_gen! {
    trace;
    ensures((effects!(old(trace), trace, effect!(FdAccess), path_effect!(PathAccessAt, fd, p, f) if fd == dirfd && p == old(path) && f == (flags == 0))));
    syscall(fstatat64, dirfd: usize, path: [u8; 4096], stat: &mut libc::stat, flags: i32)
}

syscall_spec_gen! {
    trace;
    requires((specs.len() >= 2));
    ensures((effects!(old(trace), trace, effect!(FdAccess))));
    syscall(futimens, fd: usize, specs: &Vec<libc::timespec>)
}

//https://man7.org/linux/man-pages/man2/utimensat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[external_calls(utimensat, __error)]
#[requires(specs.len() >= 2)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess), path_effect!(PathAccessAt, fd, p, f) if fd == dirfd && p == old(path) && f == !flag_set(flags, libc::AT_SYMLINK_NOFOLLOW)))]
pub fn os_utimensat(
    dirfd: usize,
    path: [u8; 4096],
    specs: &Vec<libc::timespec>,
    flags: libc::c_int,
) -> isize {
    // TODO: There is no direct utimensat syscall on osx. Instead, we will just call the
    //       libc wrapper
    let res = unsafe {
        utimensat(
            dirfd as i32,
            path.as_ptr() as *const i8,
            specs.as_ptr(),
            flags,
        ) as isize
    };
    if res == -1 {
        // convert errno to -errno to conform to our expected syscall api
        -1 * unsafe { *__error() } as isize
    } else {
        res
    }
}

syscall_spec_gen! {
    trace;
    ensures((effects!(old(trace), trace)));
    syscall(gettimeofday, timeval: &mut libc::timeval, zero: usize)
}

#[with_ghost_var(trace: &mut Trace)]
#[external_calls(size_of)]
#[external_methods(into)]
#[trusted]
#[ensures(effects!(old(trace), trace))]
pub fn os_getboottime(timeval: &mut libc::timeval) -> isize {
    let __start_ts = start_timer();
    // boot time is available through sysctl
    // should these conversions happen in trace_clock_get_time instead?
    let sysctl_name = vec![libc::CTL_KERN, libc::KERN_BOOTTIME];
    let sysctl_len: libc::size_t = sysctl_name.len().into();
    let tv_size: libc::size_t = std::mem::size_of::<libc::timeval>().into();
    // 	T_ASSERT_POSIX_SUCCESS(sysctlbyname("kern.boottime", &bt_tv, &len, NULL, 0), NULL);
    let result = unsafe {
        syscall!(
            __SYSCTL,
            sysctl_name.as_ptr(),
            &sysctl_len as *const libc::size_t,
            timeval as *mut libc::timeval,
            &tv_size as *const usize,
            0,
            0
        ) as isize
    };
    let __end_ts = stop_timer();
    push_syscall_result("getboottime", __start_ts, __end_ts);
    result
}

syscall_spec_gen! {
    trace;
    ensures((effects!(old(trace), trace)));
    syscall(rusageself, rusage: &mut libc::rusage)
}

// TODO: unclear to me that the raw syscall! will handle return values correctly.
//       e.g. from https://opensource.apple.com/source/xnu/xnu-792.25.20/bsd/kern/syscalls.master
//       it seems that this directly returns the value as ret val.
syscall_spec_gen! {
    trace;
    ensures((effects!(old(trace), trace)));
    syscall(thread_selfusage)
}

#[with_ghost_var(trace: &mut Trace)]
#[external_calls(SecRandomCopyBytes)]
#[requires(buf.len() >= cnt)]
#[trusted]
#[ensures(old(raw_ptr(buf)) == raw_ptr(buf))]
#[ensures(effects!(old(trace), trace, effect!(WriteMem, addr, count) if addr == old(raw_ptr(buf)) && count == cnt))]
pub fn os_getrandom(buf: &mut [u8], cnt: usize, flags: u32) -> isize {
    // no native syscall, use mac's secure random framework.
    // May also just read from /dev/random, but then its subject to File Descriptor exhaustion.

    // TODO: handle return value
    unsafe {
        SecRandomCopyBytes(kSecRandomDefault, cnt, buf.as_mut_ptr() as *mut c_void);
    }
    0
}

// https://opensource.apple.com/source/xnu/xnu-7194.81.3/osfmk/kern/clock.c.auto.html
// Waits until the deadline (in absolute time, mach ticks) has passed
// To use, you should call os_timebase_info for the conversion between mach_ticks and
// nanoseconds first.
#[with_ghost_var(trace: &mut Trace)]
#[external_calls(mach_wait_until)]
#[trusted]
#[ensures(effects!(old(trace), trace))]
pub fn os_wait_until(deadline: u64) -> isize {
    let result = unsafe { mach_wait_until(deadline) as isize };
    result
}

#[with_ghost_var(trace: &mut Trace)]
#[external_calls(mach_wait_until)]
#[trusted]
#[ensures(effects!(old(trace), trace))]
pub fn os_absolute_time() -> u64 {
    let result = unsafe { mach_absolute_time() };
    result
}

#[with_ghost_var(trace: &mut Trace)]
#[external_calls(mach_timebase_info)]
#[trusted]
#[ensures(effects!(old(trace), trace))]
pub fn os_timebase_info(info: &mut mach_timebase_info_data_t) -> isize {
    // TODO: handle return value
    let result = unsafe { mach_timebase_info(info as mach_timebase_info_t) as isize };
    result
}

#[with_ghost_var(trace: &mut Trace)]
#[external_methods(set_len)]
#[trusted]
#[requires(dirp.capacity() >= count)]
#[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
pub fn os_getdents64(fd: usize, dirp: &mut Vec<u8>, count: usize) -> isize {
    let __start_ts = start_timer();
    // TODO: safe to put 0 in for basep? TODO...
    // TODO: ensure directory entry format is correct...
    let mut basep: u64 = 0;
    let result = unsafe {
        let result = syscall!(
            GETDIRENTRIES,
            fd,
            dirp.as_mut_ptr(),
            count,
            &mut basep as *mut u64
        );
        if (result as isize) != -1 {
            dirp.set_len(result);
        } else {
            dirp.set_len(0);
        }
        result as isize
    };
    let __end_ts = stop_timer();
    push_syscall_result("getdirentries", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/fstat.2.html
syscall_spec_gen! {
    trace;
    ensures((effects!(old(trace), trace, effect!(FdAccess))));
    syscall(fstat64 ALIAS fstat, fd: usize, stat: &mut libc::stat)
}
