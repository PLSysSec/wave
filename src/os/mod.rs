use crate::tcb::misc::flag_set;
use crate::tcb::os_specs::*;
#[cfg(feature = "verify")]
use crate::tcb::path::path_safe;
#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::types::*;
use crate::{effect, effects};
use prusti_contracts::*;
use syscall::syscall;
use wave_macros::with_ghost_var;
use crate::tcb::sbox_mem::{valid_linmem, raw_ptr};

#[cfg_attr(target_os = "linux", path = "platform/linux.rs")]
#[cfg_attr(target_os = "macos", path = "platform/mac.rs")]
mod platform;
pub use platform::*;


// Common implementations between operating systems

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(path_safe(&path, !flag_set(flags, libc::O_NOFOLLOW) ))] // path_safe is parameterized by `should_follow`, so we need to reverse it
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[requires(dir_fd.to_raw() == ctx.homedir_host_fd.to_raw())]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// // #[ensures(one_effect!(old(trace), trace, effect!(PathAccessAt, os_fd, p)))]
// pub fn trace_openat(
//     ctx: &VmCtx,
//     dir_fd: HostFd,
//     path: HostPath,
//     flags: i32,
// ) -> RuntimeResult<usize> {
//     let os_fd: usize = dir_fd.to_raw();
//     let r = os_openat(os_fd, path, flags);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// #[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
// pub fn trace_close(ctx: &VmCtx, fd: HostFd) -> RuntimeResult<usize> {
//     let os_fd: usize = fd.to_raw();
//     let r = os_close(os_fd);
//     RuntimeError::from_syscall_ret(r)
// }

#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx.fits_in_lin_mem(ptr, cnt as u32, trace))]
#[requires(cnt < ctx.memlen)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
// #[trusted]
// read writes `cnt` bytes to sandbox memory
// #[ensures(effects!(old(trace), trace, effect!(FdAccess), effect!(WriteN, addr, count)))]
pub fn trace_read(ctx: &mut VmCtx, fd: HostFd, ptr: SboxPtr, cnt: usize) -> RuntimeResult<usize> {
    let slice = ctx.slice_mem_mut(ptr, cnt as u32);
    let os_fd: usize = fd.to_raw();
    let r = os_read(os_fd, slice, cnt);

    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(
    iovs.len() >= 0 &&
    forall(|idx: usize|  (idx < iovs.len() && idx >= 0) ==> {
        let iov = iovs.lookup(idx);
        let buf = iov.iov_base;
        let cnt = iov.iov_len;
        // ctx.fits_in_lin_mem(buf, cnt, trace)
        (buf >= 0) && (cnt >= 0) && 
        (buf as usize) + (cnt as usize) < LINEAR_MEM_SIZE &&
        (buf <= buf + cnt)
    })
)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn trace_readv(ctx: &mut VmCtx, fd: HostFd, iovs: &WasmIoVecs, iovcnt: usize) -> RuntimeResult<usize> {
    //let slice = ctx.slice_mem_mut(ptr, cnt as u32);
    // let mut native_iovs = ctx.translate_iovs(iovs, iovcnt);
    // native_iovs
    // let os_fd: usize = fd.to_raw();
    // let r = os_readv(os_fd, &mut native_iovs, iovcnt);
    // RuntimeError::from_syscall_ret(r)

    let mut native_iovs = ctx.translate_iovs(iovs, iovcnt);
    // native_iovs
    let os_fd: usize = fd.to_raw();
    let r = os_readv(os_fd, &mut native_iovs, iovcnt);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx.fits_in_lin_mem(ptr, cnt as u32, trace))]
#[requires(cnt < ctx.memlen)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
// pread writes `cnt` bytes to sandbox memory
// #[ensures(effects!(old(trace), trace, effect!(FdAccess), effect!(WriteN, addr, count)))]
pub fn trace_pread(
    ctx: &mut VmCtx,
    fd: HostFd,
    ptr: SboxPtr,
    cnt: usize,
    offset: usize,
) -> RuntimeResult<usize> {
    let slice = ctx.slice_mem_mut(ptr, cnt as u32);
    let os_fd: usize = fd.to_raw();
    let r = os_pread(os_fd, slice, cnt, offset);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(
    iovs.len() >= 0 &&
    forall(|idx: usize|  (idx < iovs.len() && idx >= 0) ==> {
        let iov = iovs.lookup(idx);
        let buf = iov.iov_base;
        let cnt = iov.iov_len;
        // ctx.fits_in_lin_mem(buf, cnt, trace)
        (buf >= 0) && (cnt >= 0) && 
        (buf as usize) + (cnt as usize) < LINEAR_MEM_SIZE &&
        (buf <= buf + cnt)
    })
)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn trace_preadv(ctx: &mut VmCtx, fd: HostFd, iovs: &WasmIoVecs, iovcnt: usize, offset: usize) -> RuntimeResult<usize> {
    let mut native_iovs = ctx.translate_iovs(iovs, iovcnt);
    // native_iovs
    let os_fd: usize = fd.to_raw();
    let r = os_preadv(os_fd, &mut native_iovs, iovcnt, offset);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx.fits_in_lin_mem(ptr, cnt as u32, trace))]
#[requires(cnt < ctx.memlen)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
// write reads `cnt` bytes to the sandbox
// #[ensures(effects!(old(trace), trace, effect!(FdAccess), effect!(ReadN, addr, count)))]
pub fn trace_write(ctx: &mut VmCtx, fd: HostFd, ptr: SboxPtr, cnt: usize) -> RuntimeResult<usize> {
    let slice = ctx.slice_mem_mut(ptr, cnt as u32);
    let os_fd: usize = fd.to_raw();
    let r = os_write(os_fd, slice, cnt);
    RuntimeError::from_syscall_ret(r)
}


#[with_ghost_var(trace: &mut Trace)]
#[requires(
    iovs.len() >= 0 &&
    forall(|idx: usize|  (idx < iovs.len() && idx >= 0) ==> {
        let iov = iovs.lookup(idx);
        let buf = iov.iov_base;
        let cnt = iov.iov_len;
        // ctx.fits_in_lin_mem(buf, cnt, trace)
        (buf >= 0) && (cnt >= 0) && 
        (buf as usize) + (cnt as usize) < LINEAR_MEM_SIZE &&
        (buf <= buf + cnt)
    })
)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn trace_writev(ctx: &mut VmCtx, fd: HostFd, iovs: &WasmIoVecs, iovcnt: usize) -> RuntimeResult<usize> {
    let native_iovs = ctx.translate_iovs(iovs, iovcnt);
    // native_iovs
    let os_fd: usize = fd.to_raw();
    let r = os_writev(os_fd, &native_iovs, iovcnt);
    RuntimeError::from_syscall_ret(r)
}


#[with_ghost_var(trace: &mut Trace)]
#[requires(
    iovs.len() >= 0 &&
    forall(|idx: usize|  (idx < iovs.len() && idx >= 0) ==> {
        let iov = iovs.lookup(idx);
        let buf = iov.iov_base;
        let cnt = iov.iov_len;
        // ctx.fits_in_lin_mem(buf, cnt, trace)
        (buf >= 0) && (cnt >= 0) && 
        (buf as usize) + (cnt as usize) < LINEAR_MEM_SIZE &&
        (buf <= buf + cnt)
    })
)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn trace_pwritev(ctx: &mut VmCtx, fd: HostFd, iovs: &WasmIoVecs, iovcnt: usize, offset: usize) -> RuntimeResult<usize> {
    let native_iovs = ctx.translate_iovs(iovs, iovcnt);
    // native_iovs
    let os_fd: usize = fd.to_raw();
    let r = os_pwritev(os_fd, &native_iovs, iovcnt, offset);
    RuntimeError::from_syscall_ret(r)
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx.fits_in_lin_mem(ptr, cnt as u32, trace))]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[requires(cnt < ctx.memlen)]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
// pwrite writes `cnt` bytes to the sandbox
// #[ensures(effects!(old(trace), trace, effect!(FdAccess), effect!(ReadN, addr, count)))]
pub fn trace_pwrite(
    ctx: &mut VmCtx,
    fd: HostFd,
    ptr: SboxPtr,
    cnt: usize,
    offset: usize,
) -> RuntimeResult<usize> {
    let slice = ctx.slice_mem_mut(ptr, cnt as u32);
    let os_fd: usize = fd.to_raw();
    let r = os_pwrite(os_fd, slice, cnt, offset);
    RuntimeError::from_syscall_ret(r)
}

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// #[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
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
// #[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
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
// #[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
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
// #[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
// pub fn trace_fstat(ctx: &VmCtx, fd: HostFd, stat: &mut libc::stat) -> RuntimeResult<usize> {
//     let os_fd: usize = fd.to_raw();
//     let r = os_fstat(os_fd, stat);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(path_safe(&path, !flag_set(flags, libc::AT_SYMLINK_NOFOLLOW) ))] // flags == 0 means that O_NOFOLLOW is not set and therefore that should_follow is true
// #[requires(fd.to_raw() == ctx.homedir_host_fd.to_raw())]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// // #[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccessAt, os_fd)))]
// pub fn trace_fstatat(
//     ctx: &VmCtx,
//     fd: HostFd,
//     path: HostPath,
//     stat: &mut libc::stat,
//     flags: i32,
// ) -> RuntimeResult<usize> {
//     let os_fd: usize = fd.to_raw();
//     let r = os_fstatat(os_fd, path, stat, flags);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// #[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
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
// #[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
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
// #[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
// pub fn trace_ftruncate(ctx: &VmCtx, fd: HostFd, length: libc::off_t) -> RuntimeResult<usize> {
//     let os_fd: usize = fd.to_raw();
//     let r = os_ftruncate(os_fd, length);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(path_safe(&old_path, flag_set(flags, libc::AT_SYMLINK_FOLLOW)))]
// #[requires(path_safe(&new_path, flag_set(flags, libc::AT_SYMLINK_FOLLOW)))]
// #[requires(old_fd.to_raw() == ctx.homedir_host_fd.to_raw())]
// #[requires(new_fd.to_raw() == ctx.homedir_host_fd.to_raw())]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// // #[ensures(four_effects!(old(trace), trace, effect!(FdAccess), effect!(FdAccess), effect!(PathAccessAt, os_old_fd), effect!(PathAccessAt, os_new_fd)))]
// pub fn trace_linkat(
//     ctx: &VmCtx,
//     old_fd: HostFd,
//     old_path: HostPath,
//     new_fd: HostFd,
//     new_path: HostPath,
//     flags: i32,
// ) -> RuntimeResult<usize> {
//     let os_old_fd: usize = old_fd.to_raw();
//     let os_new_fd: usize = new_fd.to_raw();
//     // let os_old_path: Vec<u8> = old_path.into();
//     // let os_new_path: Vec<u8> = new_path.into();
//     let r = os_linkat(os_old_fd, old_path, os_new_fd, new_path, flags);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(path_safe(&path, true))]
// #[requires(dir_fd.to_raw() == ctx.homedir_host_fd.to_raw())]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// // #[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccessAt, os_fd)))]
// pub fn trace_mkdirat(
//     ctx: &VmCtx,
//     dir_fd: HostFd,
//     path: HostPath,
//     mode: libc::mode_t,
// ) -> RuntimeResult<usize> {
//     let os_fd: usize = dir_fd.to_raw();
//     // let os_path: Vec<u8> = pathname.into();
//     let r = os_mkdirat(os_fd, path, mode);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(path_safe(&pathname, false))]
// #[requires(dir_fd.to_raw() == ctx.homedir_host_fd.to_raw())]
// #[requires(ctx.fits_in_lin_mem(ptr, cnt as u32, trace))]
// #[requires(cnt < ctx.memlen)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// // #[ensures(three_effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccessAt, os_fd), effect!(WriteN, addr, count)))]
// pub fn trace_readlinkat(
//     ctx: &mut VmCtx,
//     dir_fd: HostFd,
//     pathname: HostPath,
//     ptr: SboxPtr,
//     cnt: usize,
// ) -> RuntimeResult<usize> {
//     let slice = ctx.slice_mem_mut(ptr, cnt as u32);
//     let os_fd: usize = dir_fd.to_raw();
//     // let os_path: Vec<u8> = pathname.into();
//     let r = os_readlinkat(os_fd, pathname, slice, cnt);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(path_safe(&path, false))]
// #[requires(dir_fd.to_raw() == ctx.homedir_host_fd.to_raw())]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// // #[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccessAt, os_fd)))]
// pub fn trace_unlinkat(
//     ctx: &VmCtx,
//     dir_fd: HostFd,
//     path: HostPath,
//     flags: libc::c_int,
// ) -> RuntimeResult<usize> {
//     let os_fd: usize = dir_fd.to_raw();
//     // let os_path: Vec<u8> = pathname.into();
//     let r = os_unlinkat(os_fd, path, flags);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(path_safe(&old_path, false))]
// #[requires(path_safe(&new_path, false))]
// #[requires(old_dir_fd.to_raw() == ctx.homedir_host_fd.to_raw())]
// #[requires(new_dir_fd.to_raw() == ctx.homedir_host_fd.to_raw())]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// // #[ensures(four_effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccessAt, os_old_fd), effect!(FdAccess), effect!(PathAccessAt, os_new_fd)))]
// pub fn trace_renameat(
//     ctx: &VmCtx,
//     old_dir_fd: HostFd,
//     old_path: HostPath,
//     new_dir_fd: HostFd,
//     new_path: HostPath,
// ) -> RuntimeResult<usize> {
//     let os_old_fd: usize = old_dir_fd.to_raw();
//     // let os_old_path: Vec<u8> = old_pathname.into();
//     let os_new_fd: usize = new_dir_fd.to_raw();
//     // let os_new_path: Vec<u8> = new_pathname.into();
//     let r = os_renameat(os_old_fd, old_path, os_new_fd, new_path);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(path_safe(&old_pathname, true))]
// #[requires(path_safe(&new_pathname, true))]
// #[requires(dir_fd.to_raw() == ctx.homedir_host_fd.to_raw())]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// // #[ensures(two_effects!(old(trace), trace,  effect!(PathAccessAt, os_fd), effect!(FdAccess)))]
// pub fn trace_symlinkat(
//     ctx: &VmCtx,
//     old_pathname: HostPath,
//     dir_fd: HostFd,
//     new_pathname: HostPath,
// ) -> RuntimeResult<usize> {
//     let os_fd: usize = dir_fd.to_raw();
//     // let os_old_path: Vec<u8> = old_pathname.into();
//     // let os_new_path: Vec<u8> = new_pathname.into();
//     let r = os_symlinkat(old_pathname, os_fd, new_pathname);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(specs.len() >= 2)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// #[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
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
// #[requires(path_safe(&path, !flag_set(flags, libc::AT_SYMLINK_NOFOLLOW) ))] // flags == 0 means that O_NOFOLLOW is not set and therefore that should_follow is true
// #[requires(dir_fd.to_raw() == ctx.homedir_host_fd.to_raw())]
// #[requires(specs.len() >= 2)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// // #[ensures(two_effects!(old(trace), trace, effect!(FdAccess), effect!(PathAccessAt, os_fd)))]
// pub fn trace_utimensat(
//     ctx: &VmCtx,
//     dir_fd: HostFd,
//     path: HostPath,
//     specs: &Vec<libc::timespec>,
//     flags: libc::c_int,
// ) -> RuntimeResult<usize> {
//     let os_fd: usize = dir_fd.to_raw();
//     // let os_path: Vec<u8> = pathname.into();
//     let r = os_utimensat(os_fd, path, specs, flags);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx.fits_in_lin_mem(ptr, cnt as u32, trace))]
// #[requires(cnt < ctx.memlen)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// // #[ensures(effects!(old(trace), trace, effect!(WriteN, addr, count)))]
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
// // #[ensures(effects!(old(trace), trace, effect!(FdAccess), effect!(WriteN, addr, count)))]
// pub fn trace_recv(
//     ctx: &mut VmCtx,
//     fd: HostFd,
//     ptr: SboxPtr,
//     cnt: usize,
//     flags: i32,
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
// // #[ensures(effects!(old(trace), trace, effect!(FdAccess), effect!(ReadN, addr, count)))]
// pub fn trace_send(
//     ctx: &mut VmCtx,
//     fd: HostFd,
//     ptr: SboxPtr,
//     cnt: usize,
//     flags: i32,
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
// #[ensures(effects!(old(trace), trace, effect!(Shutdown), effect!(FdAccess)))]
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
// #[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
// pub fn trace_poll(
//     ctx: &VmCtx,
//     pollfds: &mut [libc::pollfd],
//     timeout: libc::c_int,
// ) -> RuntimeResult<usize> {
//     let r = os_poll(pollfds, timeout);
//     RuntimeError::from_syscall_ret(r)
// }

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(dirp.capacity() >= count)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// #[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
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
// #[ensures(effects!(old(trace), trace, effect!(SockCreation, d, t) if d == domain as usize && t == ty as usize ))]
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
// #[ensures(effects!(old(trace), trace, effect!(FdAccess), effect!(NetAccess, protocol, ip, port)))]
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

// // TODO: I am not positive whether this returns the output value in its return
// //       or in its argument
// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
// #[ensures(effects!(old(trace), trace, effect!(FdAccess)))]
// pub fn trace_fionread(ctx: &VmCtx, sockfd: HostFd) -> RuntimeResult<usize> {
//     let os_fd: usize = sockfd.to_raw();
//     let r = os_fionread(os_fd);
//     RuntimeError::from_syscall_ret(r)
// }
