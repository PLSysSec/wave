//! Contains call implementations that are specific to MacOs
//! See src/tcb/os_specs for the raw system calls.

use crate::tcb::os_specs::*;
#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::tcb::misc::fresh_rusage;
use crate::types::*;
use crate::{effect, four_effects, no_effect, one_effect, three_effects, two_effects};
use extra_args::with_ghost_var;
use prusti_contracts::*;
use syscall::syscall;

// Call does not exist on Mac. Just do nothing...
#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(no_effect!(old(trace), trace))]
pub fn trace_advise(
    ctx: &VmCtx,
    fd: HostFd,
    offset: i64,
    len: i64,
    advice: i32,
) -> RuntimeResult<usize> {
    Ok(0)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(specs.capacity() >= 2)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
pub fn trace_futimens(
    ctx: &VmCtx,
    fd: HostFd,
    specs: &Vec<libc::timespec>,
) -> RuntimeResult<usize> {
    // TODO
    Ok(0)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(specs.capacity() >= 2)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccess)))]
pub fn trace_utimensat(
    ctx: &VmCtx,
    fd: HostFd,
    pathname: SandboxedPath,
    specs: &Vec<libc::timespec>,
    flags: libc::c_int,
) -> RuntimeResult<usize> {
    let os_fd: usize = fd.into();
    let os_path: Vec<u8> = pathname.into();
    let r = os_utimensat(os_fd, os_path, specs, flags);
    RuntimeError::from_syscall_ret(r)
}

// Inspired from https://opensource.apple.com/source/Libc/Libc-1158.1.2/gen/clock_gettime.c.auto.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(no_effect!(old(trace), trace))]
pub fn trace_clock_get_time(
    ctx: &VmCtx,
    clock_id: libc::clockid_t,
    spec: &mut libc::timespec,
) -> RuntimeResult<usize> {
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
                return ret;
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
                return ret;
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
                return -1;
            }
            spec.tv_sec = ret / 1_000_000_000;
            spec.tv_nsec = ret & 1_000_000_000;
            0
        },
        _ => {
            return Err(RuntimeError::EINVAL);
        }
    };
    RuntimeError::from_syscall_ret(r)
}

////From: https://opensource.apple.com/source/Libc/Libc-1158.1.2/gen/clock_gettime.c.auto.html
#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(no_effect!(old(trace), trace))]
pub fn trace_clock_get_res(
    ctx: &VmCtx,
    clock_id: libc::clockid_t,
    spec: &mut libc::timespec,
) -> RuntimeResult<usize> {
    // all this has brought me is sadness
    match clock_id {
        libc::CLOCK_REALTIME | libc::CLOCK_MONOTONIC | libc::CLOCK_PROCESS_CPUTIME_ID | libc::CLOCK_THREAD_CPUTIME_ID => {
            spec.tv_nsec = 1_000;
            spec.tv_sec = 0;
            Ok(0)
        },
        _ => {
            Err(RuntimeError::EINVAL)
        }
    }
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx.fits_in_lin_mem(ptr, cnt as u32, trace))]
#[requires(cnt < ctx.memlen)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(one_effect!(old(trace), trace, effect!(WriteN, addr, count)))]
pub fn trace_getrandom(
    ctx: &mut VmCtx,
    ptr: SboxPtr,
    cnt: usize,
    flags: u32,
) -> RuntimeResult<usize> {
    // TODO
    Ok(0)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(no_effect!(old(trace), trace))]
pub fn trace_nanosleep(
    ctx: &VmCtx,
    req: &libc::timespec,
    rem: &mut libc::timespec,
) -> RuntimeResult<usize> {
    let r = os_nanosleep(req, rem);
    RuntimeError::from_syscall_ret(r)
}
