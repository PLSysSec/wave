use crate::tcb::os_specs::linux::*;
#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
#[cfg(feature = "verify")]
use crate::tcb::path::path_safe;
use crate::types::*;
use crate::{effect, four_effects, no_effect, one_effect, three_effects, two_effects};
use prusti_contracts::*;
use syscall::syscall;
use wave_macros::with_ghost_var;
use crate::tcb::misc::{flag_set};


// #[with_ghost_var(trace: &mut Trace)]
// // #[requires(path_safe)]
// #[requires(path_safe(&path, !flag_set(flags, libc::O_NOFOLLOW) ))] // path_safe is parameterized by `should_follow`, so we need to reverse it
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[requires(dir_fd.to_raw() == ctx.homedir_host_fd.to_raw())]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// #[ensures(one_effect!(old(trace), trace, effect!(PathAccessAt, os_fd, p)))]
// pub fn trace_openat(
//     ctx: &VmCtx,
//     dir_fd: HostFd,
//     path: Vec<u8>, // SanndboxedPath
//     flags: i32,
// ) -> RuntimeResult<usize> {
//     // #[requires(dir_fd.to_raw() == ctx.homedir_host_fd.to_raw())]
//     // assert!(dir_fd.to_raw() == ctx.homedir_host_fd.to_raw());
//     let os_fd: usize = dir_fd.to_raw();
//     // assert!(os_fd == ctx.homedir_host_fd.to_raw());
//     // let os_path: Vec<u8> = path.into();

//     // assert!(os_path.is_relative());
//     let r = os_openat(os_fd, path, flags);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// #[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
// pub fn trace_close(ctx: &VmCtx, fd: HostFd) -> RuntimeResult<usize> {
//     let os_fd: usize = fd.to_raw();
//     let r = os_close(os_fd);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx.fits_in_lin_mem(ptr, cnt as u32, trace))]
// #[requires(cnt < ctx.memlen)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// #[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(WriteN, addr, count)))]
// /// read writes `cnt` bytes to sandbox memory
// pub fn trace_read(ctx: &mut VmCtx, fd: HostFd, ptr: SboxPtr, cnt: usize) -> RuntimeResult<usize> {
//     let slice = ctx.slice_mem_mut(ptr, cnt as u32);
//     let os_fd: usize = fd.to_raw();
//     let r = os_read(os_fd, slice, cnt);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx.fits_in_lin_mem(ptr, cnt as u32, trace))]
// //#[requires(buf.capacity() >= cnt)]
// #[requires(cnt < ctx.memlen)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// //#[ensures(buf.len() == result)]
// //#[ensures(buf.capacity() >= cnt)]
// /// pread writes `cnt` bytes to sandbox memory
// #[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(WriteN, addr, count)))]
// pub fn trace_pread(
//     ctx: &mut VmCtx,
//     fd: HostFd,
//     ptr: SboxPtr,
//     cnt: usize,
//     offset: usize,
// ) -> RuntimeResult<usize> {
//     let slice = ctx.slice_mem_mut(ptr, cnt as u32);
//     let os_fd: usize = fd.to_raw();
//     let r = os_pread(os_fd, slice, cnt, offset);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx.fits_in_lin_mem(ptr, cnt as u32, trace))]
// #[requires(cnt < ctx.memlen)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// // write reads `cnt` bytes to the sandbox
// #[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(ReadN, addr, count)))]
// pub fn trace_write(ctx: &mut VmCtx, fd: HostFd, ptr: SboxPtr, cnt: usize) -> RuntimeResult<usize> {
//     let slice = ctx.slice_mem_mut(ptr, cnt as u32);
//     let os_fd: usize = fd.to_raw();
//     let r = os_write(os_fd, slice, cnt);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx.fits_in_lin_mem(ptr, cnt as u32, trace))]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[requires(cnt < ctx.memlen)]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// // pwrite writes `cnt` bytes to the sandbox
// #[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(ReadN, addr, count)))]
// pub fn trace_pwrite(
//     ctx: &mut VmCtx,
//     fd: HostFd,
//     ptr: SboxPtr,
//     cnt: usize,
//     offset: usize,
// ) -> RuntimeResult<usize> {
//     let slice = ctx.slice_mem_mut(ptr, cnt as u32);
//     //let start = ptr as usize;
//     //let end = ptr as usize + cnt as usize;
//     //let slice = &ctx.mem[start..end];
//     //Ok(1)
//     let os_fd: usize = fd.to_raw();
//     let r = os_pwrite(os_fd, slice, cnt, offset);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// #[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
// pub fn trace_seek(ctx: &VmCtx, fd: HostFd, offset: i64, whence: i32) -> RuntimeResult<usize> {
//     let os_fd: usize = fd.to_raw();
//     let r = os_seek(os_fd, offset, whence);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// #[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
// pub fn trace_advise(
//     ctx: &VmCtx,
//     fd: HostFd,
//     offset: i64,
//     len: i64,
//     advice: i32,
// ) -> RuntimeResult<usize> {
//     let os_fd: usize = fd.to_raw();
//     let r = os_advise(os_fd, offset, len, advice);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// #[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
// pub fn trace_allocate(ctx: &VmCtx, fd: HostFd, offset: i64, len: i64) -> RuntimeResult<usize> {
//     let os_fd: usize = fd.to_raw();
//     let r = os_allocate(os_fd, offset, len);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// #[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
// pub fn trace_sync(ctx: &VmCtx, fd: HostFd) -> RuntimeResult<usize> {
//     let os_fd: usize = fd.to_raw();
//     let r = os_sync(os_fd);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// #[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
// pub fn trace_datasync(ctx: &VmCtx, fd: HostFd) -> RuntimeResult<usize> {
//     let os_fd: usize = fd.to_raw();
//     let r = os_datasync(os_fd);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// #[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
// pub fn trace_fstat(ctx: &VmCtx, fd: HostFd, stat: &mut libc::stat) -> RuntimeResult<usize> {
//     let os_fd: usize = fd.to_raw();
//     let r = os_fstat(os_fd, stat);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(path_safe(&path, (flags == 0) ))] // flags == 0 means that O_NOFOLLOW is not set and therefore that should_follow is true
// #[requires(fd.to_raw() == ctx.homedir_host_fd.to_raw())]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// #[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccessAt, os_fd)))]
// pub fn trace_fstatat(
//     ctx: &VmCtx,
//     fd: HostFd,
//     path: Vec<u8>,
//     stat: &mut libc::stat,
//     flags: i32,
// ) -> RuntimeResult<usize> {
//     let os_fd: usize = fd.to_raw();
//     let os_path: Vec<u8> = path.into();
//     let r = os_fstatat(os_fd, os_path, stat, flags);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// #[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
// pub fn trace_fgetfl(ctx: &VmCtx, fd: HostFd) -> RuntimeResult<usize> {
//     let os_fd: usize = fd.to_raw();
//     let r = os_fgetfl(os_fd);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// #[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
// pub fn trace_fsetfl(ctx: &VmCtx, fd: HostFd, flags: libc::c_int) -> RuntimeResult<usize> {
//     let os_fd: usize = fd.to_raw();
//     let r = os_fsetfl(os_fd, flags);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// #[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
// pub fn trace_ftruncate(ctx: &VmCtx, fd: HostFd, length: libc::off_t) -> RuntimeResult<usize> {
//     let os_fd: usize = fd.to_raw();
//     let r = os_ftruncate(os_fd, length);
//     RuntimeError::from_syscall_ret(r)
// }

#[with_ghost_var(trace: &mut Trace)]
#[requires(path_safe(&old_path, true))]
#[requires(path_safe(&new_path, true))]
#[requires(old_fd.to_raw() == ctx.homedir_host_fd.to_raw())]
#[requires(new_fd.to_raw() == ctx.homedir_host_fd.to_raw())]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
// #[ensures(four_effects!(old(trace), trace, effect!(FdAccess), effect!(FdAccess), effect!(PathAccessAt, os_old_fd), effect!(PathAccessAt, os_new_fd)))]
pub fn trace_linkat(
    ctx: &VmCtx,
    old_fd: HostFd,
    old_path: GuestPath,
    new_fd: HostFd,
    new_path: GuestPath,
    flags: i32,
) -> RuntimeResult<usize> {
    let os_old_fd: usize = old_fd.to_raw();
    let os_new_fd: usize = new_fd.to_raw();
    // let os_old_path: Vec<u8> = old_path.into();
    // let os_new_path: Vec<u8> = new_path.into();
    let r = os_linkat(os_old_fd, old_path, os_new_fd, new_path, flags);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(path_safe(&path, true))]
#[requires(dir_fd.to_raw() == ctx.homedir_host_fd.to_raw())]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
// #[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccessAt, os_fd)))]
pub fn trace_mkdirat(
    ctx: &VmCtx,
    dir_fd: HostFd,
    path: GuestPath,
    mode: libc::mode_t,
) -> RuntimeResult<usize> {
    let os_fd: usize = dir_fd.to_raw();
    // let os_path: Vec<u8> = pathname.into();
    let r = os_mkdirat(os_fd, path, mode);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(path_safe(&pathname, false))]
#[requires(dir_fd.to_raw() == ctx.homedir_host_fd.to_raw())]
#[requires(ctx.fits_in_lin_mem(ptr, cnt as u32, trace))]
#[requires(cnt < ctx.memlen)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
// #[ensures(three_effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccessAt, os_fd), effect!(WriteN, addr, count)))]
pub fn trace_readlinkat(
    ctx: &mut VmCtx,
    dir_fd: HostFd,
    pathname: GuestPath,
    ptr: SboxPtr,
    cnt: usize,
) -> RuntimeResult<usize> {
    let slice = ctx.slice_mem_mut(ptr, cnt as u32);
    let os_fd: usize = dir_fd.to_raw();
    // let os_path: Vec<u8> = pathname.into();
    let r = os_readlinkat(os_fd, pathname, slice, cnt);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(path_safe(&path, false))]
#[requires(dir_fd.to_raw() == ctx.homedir_host_fd.to_raw())]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
// #[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccessAt, os_fd)))]
pub fn trace_unlinkat(
    ctx: &VmCtx,
    dir_fd: HostFd,
    path: GuestPath,
    flags: libc::c_int,
) -> RuntimeResult<usize> {
    let os_fd: usize = dir_fd.to_raw();
    // let os_path: Vec<u8> = pathname.into();
    let r = os_unlinkat(os_fd, path, flags);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(path_safe(&old_path, false))]
#[requires(path_safe(&new_path, false))]
#[requires(old_dir_fd.to_raw() == ctx.homedir_host_fd.to_raw())]
#[requires(new_dir_fd.to_raw() == ctx.homedir_host_fd.to_raw())]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
// #[ensures(four_effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccessAt, os_old_fd), effect!(FdAccess), effect!(PathAccessAt, os_new_fd)))]
pub fn trace_renameat(
    ctx: &VmCtx,
    old_dir_fd: HostFd,
    old_path: GuestPath,
    new_dir_fd: HostFd,
    new_path: GuestPath,
) -> RuntimeResult<usize> {
    let os_old_fd: usize = old_dir_fd.to_raw();
    // let os_old_path: Vec<u8> = old_pathname.into();
    let os_new_fd: usize = new_dir_fd.to_raw();
    // let os_new_path: Vec<u8> = new_pathname.into();
    let r = os_renameat(os_old_fd, old_path, os_new_fd, new_path);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(path_safe(&old_pathname, true))]
#[requires(path_safe(&new_pathname, true))]
#[requires(dir_fd.to_raw() == ctx.homedir_host_fd.to_raw())]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
// #[ensures(two_effects!(old(trace), trace,  effect!(PathAccessAt, os_fd), effect!(FdAccess)))]
pub fn trace_symlinkat(
    ctx: &VmCtx,
    old_pathname: GuestPath,
    dir_fd: HostFd,
    new_pathname: GuestPath,
) -> RuntimeResult<usize> {
    let os_fd: usize = dir_fd.to_raw();
    // let os_old_path: Vec<u8> = old_pathname.into();
    // let os_new_path: Vec<u8> = new_pathname.into();
    let r = os_symlinkat(old_pathname, os_fd, new_pathname);
    RuntimeError::from_syscall_ret(r)
}

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(specs.len() >= 2)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// #[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
// pub fn trace_futimens(
//     ctx: &VmCtx,
//     fd: HostFd,
//     specs: &Vec<libc::timespec>,
// ) -> RuntimeResult<usize> {
//     let os_fd: usize = fd.to_raw();
//     let r = os_futimens(os_fd, specs);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(path_safe(&path, (flags == 0) ))] // flags == 0 means that O_NOFOLLOW is not set and therefore that should_follow is true
// #[requires(dir_fd.to_raw() == ctx.homedir_host_fd.to_raw())]
// #[requires(specs.len() >= 2)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// #[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccessAt, os_fd)))]
// pub fn trace_utimensat(
//     ctx: &VmCtx,
//     dir_fd: HostFd,
//     path: Vec<u8>,
//     specs: &Vec<libc::timespec>,
//     flags: libc::c_int,
// ) -> RuntimeResult<usize> {
//     let os_fd: usize = dir_fd.to_raw();
//     // let os_path: Vec<u8> = pathname.into();
//     let r = os_utimensat(os_fd, path, specs, flags);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// #[ensures(no_effect!(old(trace), trace))]
// pub fn trace_clock_get_time(
//     ctx: &VmCtx,
//     clock_id: libc::clockid_t,
//     spec: &mut libc::timespec,
// ) -> RuntimeResult<usize> {
//     let r = os_clock_get_time(clock_id, spec);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// #[ensures(no_effect!(old(trace), trace))]
// pub fn trace_clock_get_res(
//     ctx: &VmCtx,
//     clock_id: libc::clockid_t,
//     spec: &mut libc::timespec,
// ) -> RuntimeResult<usize> {
//     let r = os_clock_get_res(clock_id, spec);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx.fits_in_lin_mem(ptr, cnt as u32, trace))]
// #[requires(cnt < ctx.memlen)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// #[ensures(one_effect!(old(trace), trace, effect!(WriteN, addr, count)))]
// pub fn trace_getrandom(
//     ctx: &mut VmCtx,
//     ptr: SboxPtr,
//     cnt: usize,
//     flags: u32,
// ) -> RuntimeResult<usize> {
//     let slice = ctx.slice_mem_mut(ptr, cnt as u32);
//     let r = os_getrandom(slice, cnt, flags);
//     RuntimeError::from_syscall_ret(r)
// }

// //https://man7.org/linux/man-pages/man2/getrandom.2.html
// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx.fits_in_lin_mem(ptr, cnt as u32, trace))]
// #[requires(cnt < ctx.memlen)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// #[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(WriteN, addr, count)))]
// pub fn trace_recv(
//     ctx: &mut VmCtx,
//     fd: HostFd,
//     ptr: SboxPtr,
//     cnt: usize,
//     flags: u32,
// ) -> RuntimeResult<usize> {
//     let slice = ctx.slice_mem_mut(ptr, cnt as u32);
//     let os_fd: usize = fd.to_raw();
//     let r = os_recv(os_fd, slice, cnt, flags);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx.fits_in_lin_mem(ptr, cnt as u32, trace))]
// #[requires(cnt < ctx.memlen)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// #[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(ReadN, addr, count)))]
// pub fn trace_send(
//     ctx: &mut VmCtx,
//     fd: HostFd,
//     ptr: SboxPtr,
//     cnt: usize,
//     flags: u32,
// ) -> RuntimeResult<usize> {
//     let slice = ctx.slice_mem_mut(ptr, cnt as u32);
//     let os_fd: usize = fd.to_raw();
//     let r = os_send(os_fd, slice, cnt, flags);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// #[ensures(two_effects!(old(trace), trace, effect!(Shutdown), effect!(FdAccess)))]
// pub fn trace_shutdown(ctx: &VmCtx, fd: HostFd, how: libc::c_int) -> RuntimeResult<usize> {
//     let os_fd: usize = fd.to_raw();
//     let r = os_shutdown(os_fd, how);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// #[ensures(no_effect!(old(trace), trace))]
// pub fn trace_nanosleep(
//     ctx: &VmCtx,
//     req: &libc::timespec,
//     rem: &mut libc::timespec,
// ) -> RuntimeResult<usize> {
//     let r = os_nanosleep(req, rem);
//     RuntimeError::from_syscall_ret(r)
// }

// //TODO: not sure what the spec for this is yet.
// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// #[ensures(no_effect!(old(trace), trace))]
// pub fn trace_poll(
//     ctx: &VmCtx,
//     pollfd: &mut libc::pollfd,
//     timeout: libc::c_int,
// ) -> RuntimeResult<usize> {
//     let r = os_poll(pollfd, timeout);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(dirp.capacity() >= count)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// #[ensures(one_effect!(old(trace), trace, effect!(FdAccess)))]
// // pub fn os_getdents64(fd: usize, dirp: &mut libc::dirent, count: usize) -> usize {
// //buf: &mut Vec<u8>
// pub fn trace_getdents64(
//     ctx: &VmCtx,
//     fd: HostFd,
//     dirp: &mut Vec<u8>,
//     count: usize,
// ) -> RuntimeResult<usize> {
//     let os_fd: usize = fd.to_raw();
//     let r = os_getdents64(os_fd, dirp, count);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(domain == libc::AF_INET && (ty == libc::SOCK_STREAM || ty == libc::SOCK_DGRAM ))]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// #[ensures(one_effect!(old(trace), trace, effect!(SockCreation, d, t) if d == domain as usize && t == ty as usize ))]
// pub fn trace_socket(ctx: &VmCtx, domain: i32, ty: i32, protocol: i32) -> RuntimeResult<usize> {
//     let r = os_socket(domain, ty, protocol);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(addr_in_netlist(&ctx.netlist, addr.sin_addr.s_addr, addr.sin_port as u32))]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// #[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(NetAccess, protocol, ip, port)))]
// pub fn trace_connect(
//     ctx: &VmCtx,
//     sockfd: HostFd,
//     addr: &libc::sockaddr_in,
//     addrlen: u32,
// ) -> RuntimeResult<usize> {
//     let os_fd: usize = sockfd.to_raw();
//     let r = os_connect(os_fd, addr, addrlen);
//     RuntimeError::from_syscall_ret(r)
// }
