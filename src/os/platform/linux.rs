//! Contains call implementations that are specific to Linux/Posix
//! See src/tcb/os_specs for the raw system calls.

use crate::tcb::os_specs::*;
#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::types::*;
use crate::{effect, four_effects, no_effect, one_effect, three_effects, two_effects};
use extra_args::with_ghost_var;
use prusti_contracts::*;
use syscall::syscall;

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
pub fn trace_advise(
    ctx: &VmCtx,
    fd: HostFd,
    offset: i64,
    len: i64,
    advice: i32,
) -> RuntimeResult<usize> {
    let os_fd: usize = fd.into();
    let r = os_advise(os_fd, offset, len, advice);
    RuntimeError::from_syscall_ret(r)
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
    let os_fd: usize = fd.into();
    let r = os_futimens(os_fd, specs);
    RuntimeError::from_syscall_ret(r)
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

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(no_effect!(old(trace), trace))]
pub fn trace_clock_get_time(
    ctx: &VmCtx,
    clock_id: libc::clockid_t,
    spec: &mut libc::timespec,
) -> RuntimeResult<usize> {
    let r = os_clock_get_time(clock_id, spec);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(no_effect!(old(trace), trace))]
pub fn trace_clock_get_res(
    ctx: &VmCtx,
    clock_id: libc::clockid_t,
    spec: &mut libc::timespec,
) -> RuntimeResult<usize> {
    let r = os_clock_get_res(clock_id, spec);
    RuntimeError::from_syscall_ret(r)
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
    let slice = ctx.slice_mem_mut(ptr, cnt as u32);
    let r = os_getrandom(slice, cnt, flags);
    RuntimeError::from_syscall_ret(r)
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
