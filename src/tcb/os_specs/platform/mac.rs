use crate::tcb::sbox_mem::as_sbox_ptr;
#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::verifier_interface::{push_syscall_result, start_timer, stop_timer};
use crate::{effect, four_effects, no_effect, one_effect, three_effects, two_effects};
use extra_args::{external_call, external_method, with_ghost_var};
use prusti_contracts::*;
use syscall::syscall;

// TODO: MOVE THIS STUFF TO UNTRUSTED
// make this suppppppper thing wrapper

macro_rules! save_and_change_cwd {
    ($fd:ident) => {
        {
            // TODO: better error handling
            let cwd = vec![b'.'];
            // TODO: check flags
            let cwd_fd = unsafe { syscall!(OPEN, cwd.as_ptr(), 0) };
            assert!(cwd_fd > 0); // REPLACE LATER
            let res = unsafe { syscall!(FCHDIR, $fd) };
            assert!(res == 0); // REPLACE LATER
            cwd_fd
        }
    }
}

macro_rules! restore_cwd {
    ($cwd_fd:ident) => {
        {
            // restore working directory
            let res = unsafe { syscall!(FCHDIR, $cwd_fd) };
            assert!(res == 0); // REPLACE LATER
        }
    }
}

//https://man7.org/linux/man-pages/man2/open.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, effect!(PathAccess)))]
pub fn os_openat(dirfd: usize, pathname: Vec<u8>, flags: i32) -> isize {
    let __start_ts = start_timer();
    let cwd = save_and_change_cwd!(dirfd);
    let result = unsafe { syscall!(OPEN, dirfd, pathname.as_ptr(), flags) as isize };
    restore_cwd!(cwd);
    let __end_ts = stop_timer();
    push_syscall_result("openat", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/close.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
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
#[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(WriteN, addr, count) if addr == old(as_sbox_ptr(buf)) && count == cnt))]
pub fn os_read(fd: usize, buf: &mut [u8], cnt: usize) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(READ, fd, buf.as_mut_ptr(), cnt) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("read", __start_ts, __end_ts);
    result
}

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

//https://man7.org/linux/man-pages/man2/write.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(buf.len() >= cnt)]
#[trusted]
#[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(ReadN, addr, count) if addr == old(as_sbox_ptr(buf)) && count == cnt))]
pub fn os_write(fd: usize, buf: &[u8], cnt: usize) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(WRITE, fd, buf.as_ptr(), cnt) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("write", __start_ts, __end_ts);
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

//https://man7.org/linux/man-pages/man2/lseek.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
pub fn os_seek(fd: usize, offset: i64, whence: i32) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(LSEEK, fd, offset, whence) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("seek", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/fadvise64.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
pub fn os_advise(fd: usize, offset: i64, len: i64, advice: i32) -> isize {
    // Not implmented on Mac...
    // TODO URGENT(Evan): do we just say trace did fdaccess then? Or should
    //                    I remove cause it didn't really do it...
    0
}

// https://man7.org/linux/man-pages/man2/fallocate.2.html
// hardcode mode to 0 to behave more like posix_fallocate
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
pub fn os_allocate(fd: usize, offset: i64, len: i64) -> isize {
    // FROM: https://lists.apple.com/archives/darwin-dev/2007/Dec/msg00040.html
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

//https://man7.org/linux/man-pages/man2/fsync.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
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
#[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
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
#[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
pub fn os_fstat(fd: usize, stat: &mut libc::stat) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(FSTAT, fd, stat as *mut libc::stat) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("fstat", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/fstatat.2.html
// Inspired by: https://opensource.apple.com/source/cvs/cvs-42/cvs/lib/openat.c.auto.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccess)))]
pub fn os_fstatat(fd: usize, path: Vec<u8>, stat: &mut libc::stat, flags: i32) -> isize {
    let __start_ts = start_timer();
    let cwd = save_and_change_cwd!(fd);
    // TODO: 64-bit or not?
    // TODO CHECK FLAG FOR SYMLINK FOLLOW OR NOT
    let result = unsafe { syscall!(STAT64, path.as_ptr(), stat as *mut libc::stat) as isize };
    restore_cwd!(cwd);
    let __end_ts = stop_timer();
    push_syscall_result("fstatat", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/fcntl.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
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
#[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
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
#[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
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
#[ensures(four_effects!(old(trace), trace, effect!(FdAccess), effect!(FdAccess), effect!(PathAccess), effect!(PathAccess)))]
pub fn os_linkat(
    old_fd: usize,
    old_path: Vec<u8>,
    new_fd: usize,
    new_path: Vec<u8>,
    flags: i32,
) -> isize {
    // TODO: This is annoying, cant do cwd trick cause
    // new_path needs to be relative to new_fd .............................
    
    /*let __start_ts = start_timer();
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
    result*/
    0
}

//https://man7.org/linux/man-pages/man2/mkdirat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccess)))]
pub fn os_mkdirat(dir_fd: usize, pathname: Vec<u8>, mode: libc::mode_t) -> isize {
    let __start_ts = start_timer();
    let cwd = save_and_change_cwd!(dir_fd);
    let result = unsafe { syscall!(MKDIR, pathname.as_ptr(), mode) as isize };
    restore_cwd!(cwd);
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
#[ensures(three_effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccess), effect!(WriteN, addr, count) if addr == old(as_sbox_ptr(buf)) && count == cnt))]
pub fn os_readlinkat(dir_fd: usize, pathname: Vec<u8>, buf: &mut [u8], cnt: usize) -> isize {
    let __start_ts = start_timer();
    let cwd = save_and_change_cwd!(dir_fd);
    let result =
        unsafe { syscall!(READLINK, pathname.as_ptr(), buf.as_mut_ptr(), cnt) as isize };
    restore_cwd!(cwd);
    let __end_ts = stop_timer();
    push_syscall_result("readlinkat", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/unlinkat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccess)))]
pub fn os_unlinkat(dir_fd: usize, pathname: Vec<u8>, flags: libc::c_int) -> isize {
    let __start_ts = start_timer();
    let cwd = save_and_change_cwd!(dir_fd);
    let result = unsafe { syscall!(UNLINK, pathname.as_ptr(), flags) as isize };
    restore_cwd!(cwd);
    let __end_ts = stop_timer();
    push_syscall_result("unlinkat", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/renameat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(four_effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccess), effect!(FdAccess), effect!(PathAccess)))]
pub fn os_renameat(
    old_dir_fd: usize,
    old_pathname: Vec<u8>,
    new_dir_fd: usize,
    new_pathname: Vec<u8>,
) -> isize {
    // TODO: same as linkat....
    /*let __start_ts = start_timer();
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
    result*/
    0
}

//https://man7.org/linux/man-pages/man2/symlinkat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(three_effects!(old(trace), trace,  effect!(PathAccess), effect!(FdAccess), effect!(PathAccess)))]
pub fn os_symlinkat(old_pathname: Vec<u8>, dir_fd: usize, new_pathname: Vec<u8>) -> isize {
    let __start_ts = start_timer();
    // TODO: same as linkat....
    /*let result = unsafe {
        syscall!(
            SYMLINKAT,
            old_pathname.as_ptr(),
            dir_fd,
            new_pathname.as_ptr()
        ) as isize
    };
    let __end_ts = stop_timer();
    push_syscall_result("symlinkat", __start_ts, __end_ts);
    result*/
    0
}

//https://man7.org/linux/man-pages/man2/utimensat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(specs.capacity() >= 2)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
pub fn os_futimens(fd: usize, specs: &Vec<libc::timespec>) -> isize {
    let __start_ts = start_timer();
    // TODO: check ret
    let result = unsafe { syscall!(FUTIMES, fd, specs.as_ptr()) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("futimens", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/utimensat.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(specs.capacity() >= 2)]
#[trusted]
#[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccess)))]
pub fn os_utimensat(
    fd: usize,
    pathname: Vec<u8>,
    specs: &Vec<libc::timespec>,
    flags: libc::c_int,
) -> isize {
    let __start_ts = start_timer();
    let cwd = save_and_change_cwd!(fd);
    let result =
        unsafe { syscall!(UTIMES, pathname.as_ptr(), specs.as_ptr(), flags) as isize };
    restore_cwd!(cwd);
    let __end_ts = stop_timer();
    push_syscall_result("utimensat", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/clock_gettime.2.html
//Mac: from https://opensource.apple.com/source/Libc/Libc-1158.1.2/gen/clock_gettime.c.auto.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(no_effect!(old(trace), trace))]
pub fn os_clock_get_time(clock_id: libc::clockid_t, spec: &mut libc::timespec) -> isize {
    let __start_ts = start_timer();
    let result = match clock_id {
        libc::CLOCK_REALTIME => {
            let mut tv = libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            };
            let ret = unsafe { syscall!(GETTIMEOFDAY, &mut tv as *mut libc::timeval, 0) as isize };
            // TODO: refactor -> timeval_to_timespec function or macro...
            spec.tv_sec = tv.tv_sec;
            spec.tv_nsec = (tv.tv_usec * 1000) as i64;
            ret
        },
        libc::CLOCK_MONOTONIC => {
            // Computes a monotonic clock by subtracting the real_time with the boot_time
            // from https://opensource.apple.com/source/xnu/xnu-3789.41.3/tools/tests/darwintests/mach_boottime_usec.c.auto.html
            let mut boot_tv = libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            };
            let mut tv_size = std::mem::size_of::<libc::timeval>() as libc::size_t;
            let sysctl_name = vec![libc::CTL_KERN, libc::KERN_BOOTTIME];
            // 	T_ASSERT_POSIX_SUCCESS(sysctlbyname("kern.boottime", &bt_tv, &len, NULL, 0), NULL);
            let ret = unsafe { syscall!(__SYSCTL, sysctl_name.as_ptr(), sysctl_name.len(), &mut boot_tv as *mut libc::timeval, &tv_size as *const usize, 0, 0) as isize };
            assert!(ret == 0); // TODO: ERROR HANDLING
            let mut real_tv = libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            };
            let ret = unsafe { syscall!(GETTIMEOFDAY, &mut real_tv as *mut libc::timeval, 0) as isize };
            // from https://www.google.com/url?sa=t&rct=j&q=&esrc=s&source=web&cd=&cad=rja&uact=8&ved=2ahUKEwj-rZepot_0AhVtFjQIHasdDq4QFnoECAMQAQ&url=https%3A%2F%2Fopensource.apple.com%2Fsource%2Fxnu%2Fxnu-344%2Fbsd%2Fsys%2Ftime.h&usg=AOvVaw3WH-hjCN8NBpw9CTx_3Eer
            let mut diff_sec = real_tv.tv_sec - boot_tv.tv_sec;
            let mut diff_usec = real_tv.tv_usec - boot_tv.tv_usec;
            if diff_usec < 0 {
                diff_sec -= 1;
                diff_usec += 1_000_000;
            }
            spec.tv_sec = diff_sec;
            spec.tv_nsec = (diff_usec * 1000) as i64;
            ret
        },
        libc::CLOCK_PROCESS_CPUTIME_ID => {
            let mut ru: libc::rusage = unsafe { std::mem::zeroed() };
            let ret = unsafe { syscall!(GETRUSAGE, libc::RUSAGE_SELF, &mut ru as *mut libc::rusage) as isize };
            // from https://www.google.com/url?sa=t&rct=j&q=&esrc=s&source=web&cd=&cad=rja&uact=8&ved=2ahUKEwj-rZepot_0AhVtFjQIHasdDq4QFnoECAMQAQ&url=https%3A%2F%2Fopensource.apple.com%2Fsource%2Fxnu%2Fxnu-344%2Fbsd%2Fsys%2Ftime.h&usg=AOvVaw3WH-hjCN8NBpw9CTx_3Eer
            let mut sum_sec = ru.ru_utime.tv_sec + ru.ru_stime.tv_sec;
            let mut sum_usec = ru.ru_utime.tv_usec + ru.ru_stime.tv_usec;
            if sum_usec > 1_000_000 {
                sum_sec += 1;
                sum_usec -= 1_000_000;
            }
            spec.tv_sec = sum_sec;
            spec.tv_nsec = (sum_usec * 1000) as i64;
            ret
        },
        libc::CLOCK_THREAD_CPUTIME_ID => {
            // TODO: Annoying...
            0
        },
        _ => {
            // TODO: handle error...
            0
        }
    };
    let __end_ts = stop_timer();
    push_syscall_result("clock_get_time", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/clock_getres.2.html
//From: https://opensource.apple.com/source/Libc/Libc-1158.1.2/gen/clock_gettime.c.auto.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(no_effect!(old(trace), trace))]
pub fn os_clock_get_res(clock_id: libc::clockid_t, spec: &mut libc::timespec) -> isize {
    let __start_ts = start_timer();
    let result = match clock_id {
        libc::CLOCK_REALTIME | libc::CLOCK_MONOTONIC | libc::CLOCK_PROCESS_CPUTIME_ID => {
            spec.tv_nsec = 1000;
            spec.tv_sec = 0;
            0
        },
        libc::CLOCK_THREAD_CPUTIME_ID => {
            //TODO: annoying
            0
        },
        _ => {
            // TODO: handle
            0
        }
    };
    let __end_ts = stop_timer();
    push_syscall_result("clock_get_res", __start_ts, __end_ts);
    result
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(buf.len() >= cnt)]
#[ensures(result >= 0 ==> buf.len() >= result as usize)]
#[ensures(result >= 0 ==> result as usize <= cnt)]
#[trusted]
#[ensures(one_effect!(old(trace), trace, effect!(WriteN, addr, count) if addr == old(as_sbox_ptr(buf)) && count == cnt))]
pub fn os_getrandom(buf: &mut [u8], cnt: usize, flags: u32) -> isize {
    // no native syscall, just open /dev/random and read bytes
    // TODO...
    0
}

//https://man7.org/linux/man-pages/man2/recvfrom.2.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(buf.len() >= cnt)]
#[ensures(result >= 0 ==> buf.len() >= result as usize)]
#[ensures(result >= 0 ==> result as usize <= cnt)]
#[trusted]
#[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(WriteN, addr, count) if addr == old(as_sbox_ptr(buf)) && count == cnt))]
pub fn os_recv(fd: usize, buf: &mut [u8], cnt: usize, flags: u32) -> isize {
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
#[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(ReadN, addr, count) if addr == old(as_sbox_ptr(buf)) && count == cnt))]
pub fn os_send(fd: usize, buf: &[u8], cnt: usize, flags: u32) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(SENDTO, fd, buf.as_ptr(), cnt, flags, 0, 0) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("send", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/shutdown.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(two_effects!(old(trace), trace, effect!(Shutdown), effect!(FdAccess)))]
pub fn os_shutdown(fd: usize, how: libc::c_int) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(SHUTDOWN, fd, how) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("shutdown", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/nanosleep.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(no_effect!(old(trace), trace))]
pub fn os_nanosleep(req: &libc::timespec, rem: &mut libc::timespec) -> isize {
    // TODO: https://opensource.apple.com/source/Libc/Libc-1158.50.2/gen/nanosleep.c.auto.html
    0
}

//https://man7.org/linux/man-pages/man2/poll.2.html
// can make more efficient using slice of pollfds
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(no_effect!(old(trace), trace))]
pub fn os_poll(pollfd: &mut libc::pollfd, timeout: libc::c_int) -> isize {
    let __start_ts = start_timer();
    let result = unsafe { syscall!(POLL, pollfd as *const libc::pollfd, 1, timeout) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("poll", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/getdents64.2.html
//  long syscall(SYS_getdents, unsigned int fd, struct linux_dirent *dirp, unsigned int count);
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
    let result = unsafe {
        let result = syscall!(GETDIRENTRIES, fd, dirp.as_mut_ptr(), 0);
        dirp.set_len(result);
        result as isize
    };
    let __end_ts = stop_timer();
    push_syscall_result("getdents64", __start_ts, __end_ts);
    result
}

//https://man7.org/linux/man-pages/man2/socket.2.html
#[with_ghost_var(trace: &mut Trace)]
#[trusted]
#[ensures(no_effect!(old(trace), trace))]
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
#[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(NetAccess, protocol, ip, port) if ip == addr.sin_addr.s_addr as usize && port == addr.sin_port as usize))]
pub fn os_connect(sockfd: usize, addr: &libc::sockaddr_in, addrlen: u32) -> isize {
    let __start_ts = start_timer();
    let result =
        unsafe { syscall!(CONNECT, sockfd, addr as *const libc::sockaddr_in, addrlen) as isize };
    let __end_ts = stop_timer();
    push_syscall_result("connect", __start_ts, __end_ts);
    result
}
