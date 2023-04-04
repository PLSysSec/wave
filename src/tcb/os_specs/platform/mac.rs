use crate::{
    syscall_spec_gen,
    tcb::{misc::flag_set, path::HostPath},
    types::VmCtx,
};
// use crate::tcb::sbox_mem::raw_ptr;
#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::verifier_interface::{push_syscall_result, start_timer, stop_timer};
// use crate::{effect, effects, path_effect};
use core::ffi::c_void;
use libc::{__error, fstore_t, rusage, stat, timeval, utimensat};
// use prusti_contracts::*;
// use syscall::syscall;
// use wave_macros::{external_calls, external_methods, with_ghost_var};

use mach2::mach_time::{
    mach_absolute_time, mach_timebase_info, mach_timebase_info_data_t, mach_timebase_info_t,
    mach_wait_until,
};
use security_framework_sys::random::{kSecRandomDefault, SecRandomCopyBytes};
// use security_framework_sys::random::{kSecRandomDefault, SecRandomCopyBytes};

use crate::rvec::{BSlice, RVec};
use libc::timespec;
pub use paste::paste;

// https://man7.org/linux/man-pages/man2/pread.2.html
syscall_spec_gen! {
    // trace;
    // requires((buf.len() >= cnt));
    // ensures((effects!(old(trace), trace, effect!(FdAccess), effect!(WriteMem, addr, count) if addr == old(raw_ptr(buf)) && count == cnt)));
    // ensures((old(raw_ptr(buf)) == raw_ptr(buf)));
    sig(flux::sig(fn(fd: usize, buf: BSlice, cnt: usize{buf.len >= cnt}, offset: usize) -> isize requires WriteMem(buf.base, buf.addr, cnt)));
    syscall(pread, fd: usize, buf: BSlice, cnt: usize, offset: usize)
}

/* FLUX-TODO

//https://man7.org/linux/man-pages/man2/pwrite.2.html
syscall_spec_gen! {
    trace;
    requires((buf.len() >= cnt));
    ensures((effects!(old(trace), trace, effect!(FdAccess), effect!(ReadMem, addr, count) if addr == old(raw_ptr(buf)) && count == cnt)));
    syscall(pwrite, fd: usize, buf: &[u8], cnt: usize, offset: usize)
}
*/

syscall_spec_gen! {
    // trace;
    // ensures((effects!(old(trace), trace, effect!(FdAccess))));
    syscall(allocate, fd: usize, fstore: (&fstore_t))
}

// https://man7.org/linux/man-pages/man2/fstatat.2.html
syscall_spec_gen! {
    // trace;
    // ensures((effects!(old(trace), trace, effect!(FdAccess), path_effect!(PathAccessAt, fd, p, f) if fd == dirfd && p == old(path) && f == (flags == 0))));
    sig(flux::sig(fn(ctx: &VmCtx[@cx], dirfd: usize, path: HostPathSafe[flags==0], stat: &mut stat, flags: i32) -> isize requires PathAccessAt(dirfd, cx.homedir_host_fd)));
    syscall_with_cx(fstatat64 ALIAS fstatat, dirfd: usize, path: HostPath, stat: (&mut stat), flags: i32)
}

syscall_spec_gen! {
    // trace;
    // requires((specs.len() >= 2));
    // ensures((effects!(old(trace), trace, effect!(FdAccess))));
    sig(flux::sig(fn(fd: usize, specs: &RVec<timespec>{len : 2 <= len}) -> isize requires FdAccess()));
    syscall(futimens, fd: usize, specs: (&RVec<timespec>))
}

//https://man7.org/linux/man-pages/man2/utimensat.2.html
// #[with_ghost_var(trace: &mut Trace)]
// #[external_calls(utimensat, __error)]
// #[requires(specs.len() >= 2)]
#[flux::trusted]
// #[ensures(effects!(old(trace), trace, effect!(FdAccess), path_effect!(PathAccessAt, fd, p, f) if fd == dirfd && p == old(path) && f == !flag_set(flags, libc::AT_SYMLINK_NOFOLLOW)))]
#[flux::sig(fn (ctx: &VmCtx[@cx], dirfd: usize, path: HostPathSafe[!flag_set(flags, AT_SYMLINK_NOFOLLOW)], specs: &RVec<libc::timespec>{len : 2 <= len}, flags: libc::c_int) -> isize requires PathAccessAt(dirfd, cx.homedir_host_fd))]
pub fn os_utimensat(
    ctx: &VmCtx,
    dirfd: usize,
    path: HostPath,
    specs: &RVec<libc::timespec>,
    flags: libc::c_int,
) -> isize {
    // TODO: There is no direct utimensat syscall on osx. Instead, we will just call the
    //       libc wrapper
    let res = unsafe {
        utimensat(
            dirfd as i32,
            path.as_ptr() as *const i8,
            specs.inner.as_ptr(),
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
    // trace;
    // ensures((effects!(old(trace), trace)));
    // FLUX-TODO-ORIG extra param? syscall(gettimeofday, timeval: (&mut timeval), zero: usize)
    sig(flux::sig(fn(timeval: &strg timeval[@dummy]) -> isize ensures timeval: timeval)); // FLUX-TODO2: open-mut-ref
    syscall(gettimeofday, timeval: (&mut timeval))

}

// #[with_ghost_var(trace: &mut Trace)]
// #[external_calls(size_of)]
// #[external_methods(into)]
// #[trusted]
// #[ensures(effects!(old(trace), trace))]
// FLUX-TODO: ensures timeval business is GROSS!
#[flux::sig(fn (timeval: &strg timeval[@dummy]) -> isize ensures timeval: timeval)]
pub fn os_getboottime(timeval: &mut timeval) -> isize {
    let __start_ts = start_timer();
    // boot time is available through sysctl
    // should these conversions happen in trace_clock_get_time instead?

    // FLUX-TODO: see https://github.com/liquid-rust/flux/issues/282
    let mut sysctl_name = Vec::new();
    sysctl_name.push(libc::CTL_KERN);
    sysctl_name.push(libc::KERN_BOOTTIME);

    let sysctl_len: libc::size_t = sysctl_name.len().into();
    let tv_size: libc::size_t = std::mem::size_of::<libc::timeval>().into();
    // 	T_ASSERT_POSIX_SUCCESS(sysctlbyname("kern.boottime", &bt_tv, &len, NULL, 0), NULL);
    let result = 0; /* FLUX-TODO unsafe {
                        syscall!(
                            __SYSCTL,
                            sysctl_name.as_ptr(),
                            &sysctl_len as *const libc::size_t,
                            timeval as *mut libc::timeval,
                            &tv_size as *const usize,
                            0,
                            0
                        ) as isize
                    }; */
    let __end_ts = stop_timer();
    push_syscall_result("getboottime", __start_ts, __end_ts);
    result
}

syscall_spec_gen! {
    // trace;
    // ensures((effects!(old(trace), trace)));
    sig(flux::sig(fn(rusage: &strg rusage[@dummy]) -> isize ensures rusage: rusage)); // FLUX-TODO: ensures rusage business is GROSS!
    syscall(rusageself, rusage: (&mut rusage))
}

// TODO: unclear to me that the raw syscall! will handle return values correctly.
//       e.g. from https://opensource.apple.com/source/xnu/xnu-792.25.20/bsd/kern/syscalls.master
//       it seems that this directly returns the value as ret val.
syscall_spec_gen! {
    // trace;
    // ensures((effects!(old(trace), trace)));
    syscall(thread_selfusage,)
}

// #[with_ghost_var(trace: &mut Trace)]
// #[external_calls(SecRandomCopyBytes)]
// #[requires(buf.len() >= cnt)]
// #[trusted]
// #[ensures(old(raw_ptr(buf)) == raw_ptr(buf))]
// #[ensures(effects!(old(trace), trace, effect!(WriteMem, addr, count) if addr == old(raw_ptr(buf)) && count == cnt))]
#[flux::trusted]
#[flux::sig(fn (buf: BSlice, cnt: usize{buf.len >= cnt}, flags: u32) -> isize requires WriteMem(buf.base, buf.addr, cnt))]
pub fn os_getrandom(buf: BSlice, cnt: usize, flags: u32) -> isize {
    // no native syscall, use mac's secure random framework.
    // May also just read from /dev/random, but then its subject to File Descriptor exhaustion.

    // TODO: handle return value
    unsafe {
        SecRandomCopyBytes(
            kSecRandomDefault,
            cnt,
            buf.inner.as_mut_ptr() as *mut c_void,
        );
    }
    0
}

// https://opensource.apple.com/source/xnu/xnu-7194.81.3/osfmk/kern/clock.c.auto.html
// Waits until the deadline (in absolute time, mach ticks) has passed
// To use, you should call os_timebase_info for the conversion between mach_ticks and
// nanoseconds first.
// #[with_ghost_var(trace: &mut Trace)]
// #[external_calls(mach_wait_until)]
// #[trusted]
// #[ensures(effects!(old(trace), trace))]
pub fn os_wait_until(deadline: u64) -> isize {
    let result = unsafe { mach_wait_until(deadline) as isize };
    result
}

// #[with_ghost_var(trace: &mut Trace)]
// #[external_calls(mach_wait_until)]
// #[trusted]
// #[ensures(effects!(old(trace), trace))]
pub fn os_absolute_time() -> u64 {
    let result = unsafe { mach_absolute_time() };
    result
}

// #[with_ghost_var(trace: &mut Trace)]
// #[external_calls(mach_timebase_info)]
// #[trusted]
// #[ensures(effects!(old(trace), trace))]
// FLUX-TODO: (ASK-NICO) ensures info business is GROSS!
#[flux::trusted]
#[flux::sig(fn (info: &strg mach_timebase_info) -> isize ensures info: mach_timebase_info )]
pub fn os_timebase_info(info: &mut mach_timebase_info) -> isize {
    // TODO: handle return value
    let result = unsafe { mach_timebase_info(info as mach_timebase_info_t) as isize };
    result
}

// #[with_ghost_var(trace: &mut Trace)]
// #[external_methods(set_len)]
#[flux::trusted]
// #[requires(dirp.capacity() >= count)]
// #[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
#[flux::sig(fn(fd: usize, dirp: &mut RVec<u8>[@capacity], count: usize{capacity >= count}) -> isize)]
pub fn os_getdents64(fd: usize, dirp: &mut RVec<u8>, count: usize) -> isize {
    let __start_ts = start_timer();
    // TODO: safe to put 0 in for basep? TODO...
    // TODO: ensure directory entry format is correct...
    let mut basep: u64 = 0;
    let result = unsafe {
        let result = 0; /* FLUX-TODO: syscall!(
                            GETDIRENTRIES,
                            fd,
                            dirp.as_mut_ptr(),
                            count,
                            &mut basep as *mut u64
                        ); */
        if (result as isize) != -1 {
            dirp.inner.set_len(result);
        } else {
            dirp.inner.set_len(0);
        }
        result as isize
    };
    let __end_ts = stop_timer();
    push_syscall_result("getdirentries", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/fstat.2.html
syscall_spec_gen! {
    // trace;
    // ensures((effects!(old(trace), trace, effect!(FdAccess))));
    syscall(fstat64 ALIAS fstat, fd: usize, stat: (&mut libc::stat))
}
