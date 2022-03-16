//! Contains call implementations that are specific to MacOs
//! See src/tcb/os_specs for the raw system calls.

use crate::tcb::os_specs::*;
#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::tcb::misc::{fresh_rusage, bitwise_or_u32};
use crate::types::*;
use crate::{effect, effects};
use wave_macros::{with_ghost_var, external_call};
use prusti_contracts::*;
use syscall::syscall;

use mach2::mach_time::mach_timebase_info_data_t;

// Call does not exist on Mac. Do nothing...
#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
#[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
pub fn trace_advise(
    ctx: &VmCtx,
    fd: HostFd,
    offset: i64,
    len: i64,
    advice: i32,
) -> RuntimeResult<usize> {
    Ok(0)
}

//// FROM: https://lists.apple.com/archives/darwin-dev/2007/Dec/msg00040.html
#[with_ghost_var(trace: &mut Trace)]
#[external_call(bitwise_or_u32)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
#[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
pub fn trace_allocate(ctx: &VmCtx, fd: HostFd, offset: i64, len: i64) -> RuntimeResult<usize> {
    let os_fd: usize = fd.into();
    let fstore = libc::fstore_t {
        // we want to allocate contiguous space, and we want to allocate all space or none (TODO: CHECK THIS)
        fst_flags: bitwise_or_u32(libc::F_ALLOCATECONTIG, libc::F_ALLOCATEALL),
        // .. there are only two modes F_PEOFPOSMODE and F_VOLPOSMODE
        // neither of them seem correct but unsure...
        fst_posmode: libc::F_PEOFPOSMODE,
        fst_offset: offset,
        fst_length: len,
        fst_bytesalloc: 0,
    };
    let r = os_allocate(os_fd, &fstore);
    RuntimeError::from_syscall_ret(r)
}

// Inspired from https://opensource.apple.com/source/Libc/Libc-1158.1.2/gen/clock_gettime.c.auto.html
#[with_ghost_var(trace: &mut Trace)]
#[external_call(fresh_rusage)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
#[ensures(effects!(old(trace), trace))]
pub fn trace_clock_get_time(
    ctx: &VmCtx,
    clock_id: libc::clockid_t,
    spec: &mut libc::timespec,
) -> RuntimeResult<usize> {
    // TODO: redo this??
    //       look at https://opensource.apple.com/source/Libc/Libc-320.1.3/i386/mach/mach_absolute_time.c.auto.html
    let r = match clock_id {
        libc::CLOCK_REALTIME => {
            let mut tv = libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            };
            let ret = os_gettimeofday(&mut tv);
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
            let ret = os_getboottime(&mut boot_tv);
            if ret != 0 {
                return RuntimeError::from_syscall_ret(ret);
            }
            let mut real_tv = libc::timeval {
                tv_sec: 0,
                tv_usec: 0,
            };
            let ret = os_gettimeofday(&mut real_tv);
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
            let mut ru: libc::rusage = fresh_rusage();
            let ret = os_rusageself(&mut ru);
            if ret != 0 {
                return RuntimeError::from_syscall_ret(ret);
            }
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
            let ret = os_thread_selfusage();
            if ret == 0 {
                // TODO: -1 probably wrong...
                return RuntimeError::from_syscall_ret(-1);
            }
            spec.tv_sec = ret as i64 / 1_000_000_000;
            spec.tv_nsec = ret as i64 % 1_000_000_000;
            0
        },
        _ => {
            return Err(RuntimeError::Einval);
        }
    };
    RuntimeError::from_syscall_ret(r)
}

////From: https://opensource.apple.com/source/Libc/Libc-1158.1.2/gen/clock_gettime.c.auto.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
#[ensures(effects!(old(trace), trace))]
pub fn trace_clock_get_res(
    ctx: &VmCtx,
    clock_id: libc::clockid_t,
    spec: &mut libc::timespec,
) -> RuntimeResult<usize> {
    match clock_id {
        libc::CLOCK_REALTIME | libc::CLOCK_MONOTONIC | libc::CLOCK_PROCESS_CPUTIME_ID | libc::CLOCK_THREAD_CPUTIME_ID => {
            spec.tv_nsec = 1_000;
            spec.tv_sec = 0;
            Ok(0)
        },
        _ => {
            Err(RuntimeError::Einval)
        }
    }
}

// based on https://opensource.apple.com/source/Libc/Libc-1158.50.2/gen/nanosleep.c.auto.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
#[ensures(effects!(old(trace), trace))]
pub fn trace_nanosleep(
    ctx: &VmCtx,
    req: &libc::timespec,
    rem: &mut libc::timespec,
) -> RuntimeResult<usize> {
    let nanos = req.tv_sec * 1_000_000_000 + req.tv_nsec;
    let mut timebase_info = mach_timebase_info_data_t {
        numer: 0,
        denom: 0,
    };
    // TODO: handle errors
    os_timebase_info(&mut timebase_info);
    // TODO: do we need to worry about overflow?
    let mach_ticks = (nanos * timebase_info.numer as i64) * timebase_info.denom as i64;
    // TODO: handle errors and type cast
    os_wait_until(mach_ticks as u64);
    Ok(0)
}
