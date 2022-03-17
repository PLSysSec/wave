use crate::tcb::sbox_mem::as_sbox_ptr;
#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::verifier_interface::{push_syscall_result, start_timer, stop_timer};
use crate::{effect, effects};
use wave_macros::{external_calls, external_methods, with_ghost_var};
use prusti_contracts::*;
use syscall::syscall;
use core::ffi::c_void;
use libc::{utimensat, __error};
use syscall::platform::SyscallReturn;

use security_framework_sys::random::{SecRandomCopyBytes, kSecRandomDefault};
use mach2::mach_time::{mach_wait_until, mach_timebase_info, mach_timebase_info_t, mach_timebase_info_data_t};
use mach2::kern_return::KERN_SUCCESS;

//https://man7.org/linux/man-pages/man2/pread.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(buf.len() >= cnt)]
#[ensures(result >= 0 ==> buf.len() >= result as usize)]
#[ensures(result >= 0 ==> result as usize <= cnt)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess), effect!(WriteN, addr, count) if addr == old(as_sbox_ptr(buf)) && count == cnt))]
pub fn os_pread(fd: usize, buf: &mut [u8], cnt: usize, offset: usize) -> SyscallReturn {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(PREAD, fd, buf.as_mut_ptr(), cnt, offset)  };
    let __end_ts = stop_timer();
    push_syscall_result("pread", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/pwrite.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(buf.len() >= cnt)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess), effect!(ReadN, addr, count) if addr == old(as_sbox_ptr(buf)) && count == cnt))]
pub fn os_pwrite(fd: usize, buf: &[u8], cnt: usize, offset: usize) -> SyscallReturn {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(PWRITE, fd, buf.as_ptr(), cnt, offset)  };
    let __end_ts = stop_timer();
    push_syscall_result("pwrite", __start_ts, __end_ts);
    result
}

#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
pub fn os_allocate(fd: usize, fstore: &libc::fstore_t) -> SyscallReturn {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(FCNTL, fd, libc::F_PREALLOCATE, fstore as *const libc::fstore_t)  };
    let __end_ts = stop_timer();
    push_syscall_result("allocate", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/fstatat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccessAt, fd)))]
pub fn os_fstatat(fd: usize, path: Vec<u8>, stat: &mut libc::stat, flags: i32) -> SyscallReturn {
    let __start_ts = start_timer();
    let result = unsafe {
        syscall!(
            FSTATAT64,
            fd,
            path.as_ptr(),
            stat as *mut libc::stat,
            flags
        ) 
    };
    let __end_ts = stop_timer();
    push_syscall_result("fstatat", __start_ts, __end_ts);
    result
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(specs.len() >= 2)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
pub fn os_futimens(fd: usize, specs: &Vec<libc::timespec>) -> SyscallReturn {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(FUTIMES, fd, specs.as_ptr())  };
    let __end_ts = stop_timer();
    push_syscall_result("futimens", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/utimensat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[external_calls(utimensat, __error)]
#[requires(specs.len() >= 2)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccessAt, fd)))]
pub fn os_utimensat(
    fd: usize,
    pathname: Vec<u8>,
    specs: &Vec<libc::timespec>,
    flags: libc::c_int,
) -> SyscallReturn {
    // TODO: There is no direct utimensat syscall on osx. Instead, we will just call the
    //       libc wrapper
    let res = unsafe { utimensat(fd as i32, pathname.as_ptr() as *const i8, specs.as_ptr(), flags)  };
    if res == -1 {
       	// convert errno to -errno to conform to our expected syscall api 
	(unsafe { *__error() } as isize, true)
    } else {
        (res as isize, false)
    }
}

#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(effects!(old(trace), trace))]
pub fn os_gettimeofday(timeval: &mut libc::timeval) -> SyscallReturn {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(GETTIMEOFDAY, timeval as *mut libc::timeval, 0)  };
    let __end_ts = stop_timer();
    push_syscall_result("gettimeofday", __start_ts, __end_ts);
    result
}

#[with_ghost_var(trace: &mut Trace)]
#[external_calls(size_of)]
#[external_methods(into)]
#[trusted]
#[ensures(effects!(old(trace), trace))]
pub fn os_getboottime(timeval: &mut libc::timeval) -> SyscallReturn {
    let __start_ts = start_timer();
    // boot time is available through sysctl
    // should these conversions happen in trace_clock_get_time instead?
    let sysctl_name = vec![libc::CTL_KERN, libc::KERN_BOOTTIME];
    let sysctl_len: libc::size_t = sysctl_name.len().into();
    let tv_size: libc::size_t = std::mem::size_of::<libc::timeval>().into();
    // 	T_ASSERT_POSIX_SUCCESS(sysctlbyname("kern.boottime", &bt_tv, &len, NULL, 0), NULL);
    let result = unsafe { syscall!(__SYSCTL, sysctl_name.as_ptr(), &sysctl_len as *const libc::size_t, timeval as *mut libc::timeval, &tv_size as *const usize, 0, 0)  };
    let __end_ts = stop_timer();
    push_syscall_result("getboottime", __start_ts, __end_ts);
    result
}

#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(effects!(old(trace), trace))]
pub fn os_rusageself(rusage: &mut libc::rusage) -> SyscallReturn {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(GETRUSAGE, libc::RUSAGE_SELF, rusage as *mut libc::rusage)  };
    let __end_ts = stop_timer();
    push_syscall_result("getrusage", __start_ts, __end_ts);
    result
}

// TODO: unclear to me that the raw syscall! will handle return values correctly.
//       e.g. from https://opensource.apple.com/source/xnu/xnu-792.25.20/bsd/kern/syscalls.master
//       it seems that this directly returns the value as ret val.
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(effects!(old(trace), trace))]
pub fn os_thread_selfusage() -> SyscallReturn {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(THREAD_SELFUSAGE)  };
    let __end_ts = stop_timer();
    push_syscall_result("thread_selfusage", __start_ts, __end_ts);
    result
}

#[with_ghost_var(trace: &mut Trace)]
#[external_calls(SecRandomCopyBytes)]
#[requires(buf.len() >= cnt)]
#[ensures(result >= 0 ==> buf.len() >= result as usize)]
#[ensures(result >= 0 ==> result as usize <= cnt)]
#[trusted]
#[ensures(effects!(old(trace), trace, effect!(WriteN, addr, count) if addr == old(as_sbox_ptr(buf)) && count == cnt))]
pub fn os_getrandom(buf: &mut [u8], cnt: usize, flags: u32) -> SyscallReturn {
    // no native syscall, use mac's secure random framework.
    // May also just read from /dev/random, but then its subject to File Descriptor exhaustion.

    // TODO: handle return value
    unsafe { SecRandomCopyBytes(kSecRandomDefault, cnt, buf.as_mut_ptr() as *mut c_void); }
    (0, false)
}

// https://opensource.apple.com/source/xnu/xnu-7194.81.3/osfmk/kern/clock.c.auto.html
// Waits until the deadline (in absolute time, mach ticks) has passed
// To use, you should call os_timebase_info for the conversion between mach_ticks and
// nanoseconds first.
#[with_ghost_var(trace: &mut Trace)]
#[external_calls(mach_wait_until)]
#[trusted]
#[ensures(effects!(old(trace), trace))]
pub fn os_wait_until(deadline: u64) -> SyscallReturn {
    // TODO: handle return value
    let result = unsafe {
        mach_wait_until(deadline) 
    };
    // TODO: need to convert kern_return_t to errno
    // If result isn't success, signal error
    if result != KERN_SUCCESS {
	(result as isize, true)
    } else {
        (result as isize, false)
    } 
}

#[with_ghost_var(trace: &mut Trace)]
#[external_calls(mach_timebase_info)]
#[trusted]
#[ensures(effects!(old(trace), trace))]
pub fn os_timebase_info(info: &mut mach_timebase_info_data_t) -> SyscallReturn {
    // TODO: handle return value
    let result = unsafe {
        mach_timebase_info(info as mach_timebase_info_t) 
    };
    // TODO: need to convert kern_return_t to errno
    // If result isn't success, signal error
    if result != KERN_SUCCESS {
	(result as isize, true)
    } else {
        (result as isize, false)
    } 
}

#[with_ghost_var(trace: &mut Trace)]
#[external_methods(set_len)]
#[trusted]
#[requires(dirp.capacity() >= count)]
#[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
pub fn os_getdents64(fd: usize, dirp: &mut Vec<u8>, count: usize) -> SyscallReturn {
    let __start_ts = start_timer();
    // TODO: safe to put 0 in for basep? TODO...
    // TODO: ensure directory entry format is correct...
    let result = unsafe { syscall!(GETDIRENTRIES, fd, dirp.as_mut_ptr(), 0) };
    let __end_ts = stop_timer();
    push_syscall_result("getdirentries", __start_ts, __end_ts);
    result
}
