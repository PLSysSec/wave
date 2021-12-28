use crate::tcb::sbox_mem::as_sbox_ptr;
#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::verifier_interface::{push_syscall_result, start_timer, stop_timer};
use crate::{effect, four_effects, no_effect, one_effect, three_effects, two_effects};
use extra_args::{external_call, external_method, with_ghost_var};
use prusti_contracts::*;
use syscall::syscall;

use security_framework_sys::random::{SecRandomCopyBytes, kSecRandomDefault};
use mach2::mach_time::{mach_wait_until, mach_timebase_info, mach_timebase_info_data_t};

//https://man7.org/linux/man-pages/man2/pread.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(buf.len() >= cnt)]
#[ensures(result >= 0 ==> buf.len() >= result as usize)]
#[ensures(result >= 0 ==> result as usize <= cnt)]
#[trusted]
#[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(WriteN, addr, count) if addr == old(as_sbox_ptr(buf)) && count == cnt))]
pub fn os_pread(fd: usize, buf: &mut [u8], cnt: usize, offset: usize) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(PREAD, fd, buf.as_mut_ptr(), cnt, offset) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("pread", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/pwrite.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(buf.len() >= cnt)]
#[trusted]
#[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(ReadN, addr, count) if addr == old(as_sbox_ptr(buf)) && count == cnt))]
pub fn os_pwrite(fd: usize, buf: &[u8], cnt: usize, offset: usize) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(PWRITE, fd, buf.as_ptr(), cnt, offset) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("pwrite", __start_ts, __end_ts);
    result
}

// FROM: https://lists.apple.com/archives/darwin-dev/2007/Dec/msg00040.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
pub fn os_allocate(fd: usize, offset: i64, len: i64) -> isize {
    // Mac uses fcntl for preallocation
    // TODO URGENT(Evan): Are there major differences between this and posix version
    /* mac docs: Preallocate file storage space.
                        Note: upon success, the space
                        that is allocated can be the
                        size requested, larger than the
                        size requested, or (if the
                        F_ALLOCATEALL flag is not
                        provided) smaller than the
                        space requested.*/
    let fstore = libc::fstore_t {
        // we want to allocate contiguous space, and we want to allocate all space or none (TODO: CHECK THIS)
        fst_flags: libc::F_ALLOCATECONTIG | libc::F_ALLOCATEALL,
        // .. there are only two modes F_PEOFPOSMODE and F_VOLPOSMODE
        // neither of them seem correct but unsure...
        fst_posmode: libc::F_PEOFPOSMODE,
        fst_offset: offset,
        fst_length: len,
        fst_bytesalloc: 0,
    };
    let __start_ts = start_timer();
    let result = unsafe { syscall!(FCNTL, fd, libc::F_PREALLOCATE, &fstore as *const libc::fstore_t) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("allocate", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/fstatat.2.html
// Inspired by: https://opensource.apple.com/source/cvs/cvs-42/cvs/lib/openat.c.auto.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccess)))]
pub fn os_fstatat(fd: usize, path: Vec<u8>, stat: &mut libc::stat, flags: i32) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(FSTATAT64, fd, path.as_ptr(), stat as *mut libc::stat, flags) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("fstatat", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/utimensat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(specs.capacity() >= 2)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
pub fn os_futimes(fd: usize, specs: &Vec<libc::timeval>) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(FUTIMES, fd, specs.as_ptr()) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("futimens", __start_ts, __end_ts);
    result
}

// TODO: no utimesat syscall, will need to do cwd trick...
////https://man7.org/linux/man-pages/man2/utimensat.2.html
///*#[with_ghost_var(trace: &mut Trace)]
//#[requires(specs.capacity() >= 2)]
//#[trusted]
//#[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccess)))]
//pub fn os_utimensat(
//    fd: usize,
//    pathname: Vec<u8>,
//    specs: &Vec<libc::timespec>,
//    flags: libc::c_int,
//) -> isize {
//    let __start_ts = start_timer();
//    let cwd = save_and_change_cwd!(fd);
//    let result =
//        unsafe { syscall!(UTIMES, pathname.as_ptr(), specs.as_ptr(), flags) as isize };
//    restore_cwd!(cwd);
//    let __end_ts = stop_timer();
//    push_syscall_result("utimensat", __start_ts, __end_ts);
//    result
//}*/

// TODO: no utimesat syscall, will need to do cwd trick...
//https://man7.org/linux/man-pages/man2/utimensat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(specs.capacity() >= 2)]
#[trusted]
#[ensures(two_effects!(old(trace), trace, effect!(PathAccess)))]
pub fn os_utimes(
    pathname: Vec<u8>,
    specs: &Vec<libc::timeval>,
) -> isize {
    // TODO: no path resultion flags here, not sure if that will be an issue
    let result =
        unsafe { syscall!(UTIMES, pathname.as_ptr(), specs.as_ptr()) as isize };
    result
}


#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(no_effect!(old(trace), trace))]
pub fn os_gettimeofday(timeval: &mut libc::timeval) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(GETTIMEOFDAY, timeval as *mut libc::timeval, 0) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("gettimeofday", __start_ts, __end_ts);
    result
}

#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(no_effect!(old(trace), trace))]
pub fn os_getboottime(timeval: &mut libc::timeval) -> isize {
    let __start_ts = start_timer();
    let sysctl_name = vec![libc::CTL_KERN, libc::KERN_BOOTTIME];
    let sysctl_len: libc::size_t = sysctl_name.len().into();
    let tv_size: libc::size_t = std::mem::size_of::<libc::timeval>().into();
    // 	T_ASSERT_POSIX_SUCCESS(sysctlbyname("kern.boottime", &bt_tv, &len, NULL, 0), NULL);
    let result = unsafe { syscall!(__SYSCTL, sysctl_name.as_ptr(), &sysctl_len as *const libc::size_t, timeval as *mut libc::timeval, &tv_size as *const usize, 0, 0) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("getboottime", __start_ts, __end_ts);
    result
}

#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(no_effect!(old(trace), trace))]
pub fn os_rusageself(rusage: &mut libc::rusage) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(GETRUSAGE, libc::RUSAGE_SELF, rusage as *mut libc::rusage) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("getrusage", __start_ts, __end_ts);
    result
}

// TODO: unclear to me that the raw syscall! will handle return values correctly.
//       e.g. from https://opensource.apple.com/source/xnu/xnu-792.25.20/bsd/kern/syscalls.master
//       it seems that this directly returns the value as ret val.
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(no_effect!(old(trace), trace))]
pub fn os_thread_selfusage() -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(THREAD_SELFUSAGE) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("thread_selfusage", __start_ts, __end_ts);
    result
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(buf.len() >= cnt)]
#[ensures(result >= 0 ==> buf.len() >= result as usize)]
#[ensures(result >= 0 ==> result as usize <= cnt)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, effect!(WriteN, addr, count) if addr == old(as_sbox_ptr(buf)) && count == cnt))]
pub fn os_getrandom(buf: &mut [u8], cnt: usize, flags: u32) -> isize {
    // no native syscall, use mac's secure random framework.
    // May also just read from /dev/random, but then its subject to File Descriptor exhaustion.

    // TODO: handle return value
    unsafe { SecRandomCopyBytes(kSecRandomDefault, cnt, bytes.as_mut_ptr()) }
    0
}

#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(no_effect!(old(trace), trace))]
pub fn os_timebase_info(info: &mut mach_timebase_info_data_t) -> isize {
    // TODO: handle return value
    let result = unsafe {
        mach_timebase_info(info as mach_timebase_info_t) as isize
    };
    result
}

// https://opensource.apple.com/source/xnu/xnu-7195.81.3/osfmk/kern/clock.c.auto.html
// Waits until the deadline (in absolute time, mach ticks) has passed
// To use, you should call os_timebase_info for the conversion between mach_ticks and
// nanoseconds first.
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(no_effect!(old(trace), trace))]
pub fn os_wait_until(deadline: u64) -> isize {
    // TODO: handle return value
    let result = unsafe {
        mach_wait_until(deadline) as isize
    };
    result
}

#[with_ghost_var(trace: &mut Trace)]
#[external_method(set_len)]
#[trusted]
#[requires(dirp.capacity() >= count)]
#[ensures(no_effect!(old(trace), trace))]
// TODO: this result handling is screwed up
//#[ensures(no_effect!(old(trace), trace))]
#[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
pub fn os_getdents64(fd: usize, dirp: &mut Vec<u8>, count: usize) -> isize {
    let __start_ts = start_timer();
    // TODO: safe to put 0 in for basep? TODO...
    // TODO: ensure directory entry format is correct...
    let result = unsafe {
        let result = syscall!(GETDIRENTRIES, fd, dirp.as_mut_ptr(), 0);
        dirp.set_len(result);
        result as isize
    };
    let __end_ts = stop_timer();
    push_syscall_result("getdirentries", __start_ts, __end_ts);
    result
}

// TODO: should log the effect of changing the working directory
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(no_effect!(old(trace), trace))]
pub fn os_fchdir(fd: usize) -> isize {
    let result = unsafe {
        syscall!(FCHDIR, fd) as isize
    };
    result
}
