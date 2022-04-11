use crate::os::*;
use crate::runtime::*;
use crate::tcb::misc::{bitwise_or, first_null, flag_set, fresh_stat, push_dirent_name};
#[cfg(feature = "verify")]
use crate::tcb::verifier::external_specs::result::*;
#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
// use crate::tcb::path::{arr_depth, arr_is_relative, arr_is_symlink};
use crate::poll::{parse_subscriptions, writeback_timeouts, writeback_fds};
use crate::types::*;
use crate::{effect, effects};
use prusti_contracts::*;
use std::convert::{TryFrom, TryInto};
use std::mem;
use std::str;
use wave_macros::{external_calls, external_methods, with_ghost_var};
use RuntimeError::*;


// TODO: support arbitrary *at calls
// Currently they can only be based off our preopened dir
// (I've never seen a call that has attempted otherwise)
// We also don't need to handle the magic AT_FDCWD constant, because wasi-libc
// auto adjusts it to our home directory

// manual implementation of the `?` operator because it is currently
// broken in prusti
macro_rules! unwrap_result {
    ($p:ident) => {
        let $p = match $p {
            Ok(oc) => oc,
            Err(e) => {
                return Err(e);
            }
        };
    };
}

// TODO: eliminate as many external_methods and external_calls as possible

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#path_open
// Modifies: fdmap
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(create, to_posix, should_follow, to_openat_posix)]
#[external_calls(from, bitwise_or, flag_set)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_path_open(
    ctx: &mut VmCtx,
    v_dir_fd: u32,
    dirflags: u32,
    pathname: u32,
    path_len: u32,
    oflags: u32,
    fdflags: i32,
) -> RuntimeResult<u32> {
    let dirflags = LookupFlags::new(dirflags);
    let oflags = OFlags::new(oflags);
    let fdflags = FdFlags::from(fdflags);
    let should_follow = dirflags.should_follow();

    if v_dir_fd != HOMEDIR_FD {
        return Err(Enotcapable);
    }
    let fd = ctx.homedir_host_fd;

    let host_pathname = ctx.translate_path(pathname, path_len, should_follow, fd);
    unwrap_result!(host_pathname);

    let dflags = dirflags.to_openat_posix();
    let flags = bitwise_or(bitwise_or(dflags, oflags.to_posix()), fdflags.to_posix());

    if flag_set(flags, libc::O_NOFOLLOW) == should_follow {
        // this should never happen, but adding an extra dynamic check let me
        // avoid reasoning about bitwise operation math (which prusti does not support well)
        // in the proof
        return Err(Einval);
    }

    let fd = trace_openat(ctx, fd, host_pathname, flags)?;
    ctx.fdmap.create(HostFd::from_raw(fd))
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#fd_close
// modifies: fdmap
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(delete)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_fd_close(ctx: &mut VmCtx, v_fd: u32) -> RuntimeResult<u32> {
    // can't replace with fd_to_native until we fix question mark operator
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }
    let fd = ctx.fdmap.m[v_fd as usize]?;

    ctx.fdmap.delete(v_fd);
    let result = trace_close(ctx, fd)?;
    Ok(result as u32)
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#fd_read
// modifies: mem
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(reserve_exact, map_err)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_fd_read(ctx: &mut VmCtx, v_fd: u32, iovs: u32, iovcnt: u32) -> RuntimeResult<u32> {
    let fd = ctx.fdmap.fd_to_native(v_fd)?;

    let mut num: u32 = 0;
    let mut i = 0;
    while i < iovcnt {
        body_invariant!(ctx_safe(ctx));
        body_invariant!(trace_safe(trace, ctx));

        let start = (iovs + i * 8) as usize;
        let (ptr, len) = ctx.read_u32_pair(start)?;

        if !ctx.fits_in_lin_mem(ptr, len) {
            return Err(Efault);
        }

        let result = trace_read(ctx, fd, ptr, len as usize)?;
        num += result as u32;
        i += 1;
    }
    Ok(num)
}

// alternative implementation of https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#fd_read
// This one uses actual readv
// ssize_t readv(int fildes, const struct iovec *iov, int iovcnt);
//
// Posix representation
// =================================
// pub struct iovec {
//     pub iov_base: *mut c_void,
//     pub iov_len: usize,
// }
//
// Wasm representation
// =================================
// pub struct iovec {
//     pub iov_base: u32,
//     pub iov_len: u32,
// }
// 
// 

#[with_ghost_var(trace: &mut Trace)]
#[external_methods(push)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_fd_readv(ctx: &mut VmCtx, v_fd: u32, iovs: u32, iovcnt: u32) -> RuntimeResult<u32> {
    let fd = ctx.fdmap.fd_to_native(v_fd)?;
    // 1. Marshal IoVec to userspace
    let mut i = 0;
    let mut wasm_iov = Vec::new(); 
    while i < iovcnt {
        body_invariant!(ctx_safe(ctx));
        body_invariant!(trace_safe(trace, ctx));

        let start = (iovs + i * 8) as usize;
        let (ptr, len) = ctx.read_u32_pair(start)?;

        if !ctx.fits_in_lin_mem(ptr, len) {
            return Err(Efault);
        }

        wasm_iov.push(WasmIoVec{iov_base: ptr, iov_len: len});
        i += 1;
    }

    let result = trace_readv(ctx, fd, &wasm_iov, iovcnt as usize)?;
    Ok(result as u32)
    // forall(|i: usize| (i < wasm_iov.len() ==> 
    //    let (ptr,len) = wasm_iov.lookup(i);
    //    ctx.in_lin_mem(ptr, len)

    // Actually perform the syscall here

    //Ok(0) // temp
    //Ok(num)
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#fd_write
// modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_fd_write(ctx: &mut VmCtx, v_fd: u32, iovs: u32, iovcnt: u32) -> RuntimeResult<u32> {
    let fd = ctx.fdmap.fd_to_native(v_fd)?;

    let mut num: u32 = 0;
    let mut i = 0;
    while i < iovcnt {
        body_invariant!(ctx_safe(ctx));
        body_invariant!(trace_safe(trace, ctx));

        let start = (iovs + i * 8) as usize;

        let (ptr, len) = ctx.read_u32_pair(start)?;
        if !ctx.fits_in_lin_mem(ptr, len) {
            return Err(Efault);
        }

        let result = trace_write(ctx, fd, ptr, len as usize)?;
        num += result as u32;
        i += 1;
    }
    Ok(num)
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#fd_seek
// modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[external_calls(from_u32)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
// #[ensures(v_fd < MAX_SBOX_FDS && ctx.fdmap.contains(v_fd) ==> effects!(old(trace), trace, Effect::FdAccess))]
// #[ensures(v_fd >= MAX_SBOX_FDS ==> effects!(old(trace), trace))]
pub fn wasi_fd_seek(ctx: &VmCtx, v_fd: u32, v_filedelta: i64, v_whence: u32) -> RuntimeResult<u64> {
    let whence = Whence::from_u32(v_whence).ok_or(Einval)?;
    let fd = ctx.fdmap.fd_to_native(v_fd)?;
    let ret = trace_seek(ctx, fd, v_filedelta, whence.into())?;
    Ok(ret as u64)
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#fd_tell
// modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_fd_tell(ctx: &VmCtx, v_fd: u32) -> RuntimeResult<u64> {
    wasi_fd_seek(ctx, v_fd, 0, 1) // Whence::Cur
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#fd_advise
// modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[external_calls(try_from)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_fd_advise(
    ctx: &VmCtx,
    v_fd: u32,
    offset: u64,
    len: u64,
    v_advice: u32,
) -> RuntimeResult<u32> {
    let advice = Advice::try_from(v_advice as i32)?;
    let fd = ctx.fdmap.fd_to_native(v_fd)?;

    let ret = trace_advise(ctx, fd, offset as i64, len as i64, advice.into())?;
    Ok(ret as u32)
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#fd_allocate
// modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_fd_allocate(ctx: &VmCtx, v_fd: u32, offset: u64, len: u64) -> RuntimeResult<u32> {
    let fd = ctx.fdmap.fd_to_native(v_fd)?;

    let ret = trace_allocate(ctx, fd, offset as i64, len as i64)?;
    Ok(ret as u32)
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#fd_sync
// modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_fd_sync(ctx: &VmCtx, v_fd: u32) -> RuntimeResult<()> {
    let fd = ctx.fdmap.fd_to_native(v_fd)?;
    let ret = trace_sync(ctx, fd)?;
    Ok(())
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#fd_datasync
// modifies: None
#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_fd_datasync(ctx: &VmCtx, v_fd: u32) -> RuntimeResult<u32> {
    let fd = ctx.fdmap.fd_to_native(v_fd)?;
    let ret = trace_datasync(ctx, fd)?;
    Ok(ret as u32)
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#fd_fdstat_get
//modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[external_calls(fresh_stat, from_posix)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_fd_fdstat_get(ctx: &VmCtx, v_fd: u32) -> RuntimeResult<FdStat> {
    let fd = ctx.fdmap.fd_to_native(v_fd)?;
    let mut stat = fresh_stat();
    let result = trace_fstat(ctx, fd, &mut stat)?;
    let filetype = stat.st_mode;

    let mode_flags = trace_fgetfl(ctx, fd)?;

    let result = FdStat {
        fs_filetype: (filetype as libc::mode_t).into(),
        fs_flags: FdFlags::from_posix(mode_flags as i32),
        fs_rights_base: 0, // We don't bother to implement rights
        fs_rights_inheriting: u64::MAX,
    };
    Ok(result)
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#fd_fdstat_set_flags
#[with_ghost_var(trace: &mut Trace)]
#[external_calls(from)]
#[external_methods(to_posix)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
// can only adjust Fdflags using set_flags, not O_flags or any other flags
pub fn wasi_fd_fdstat_set_flags(ctx: &mut VmCtx, v_fd: u32, v_flags: u32) -> RuntimeResult<()> {
    let flags = FdFlags::from(v_flags as i32);
    let fd = ctx.fdmap.fd_to_native(v_fd)?;

    let posix_flags = flags.to_posix();
    let ret = trace_fsetfl(ctx, fd, posix_flags)?;
    Ok(())
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#fd_filestat_get
// modifies: None
#[with_ghost_var(trace: &mut Trace)]
#[external_calls(fresh_stat)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_fd_filestat_get(ctx: &VmCtx, v_fd: u32) -> RuntimeResult<FileStat> {
    let fd = ctx.fdmap.fd_to_native(v_fd)?;
    let mut stat = fresh_stat();
    let filetype = trace_fstat(ctx, fd, &mut stat)?;
    Ok(stat.into())
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#fd_filestat_set_size
// modifies: none
// Note: WASI API says that size should be u64.
// but ftruncate, which filestat_set_size is supposed to call used i64
// I'm keeping this as i64 since this does not cause any truncation
#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_fd_filestat_set_size(ctx: &VmCtx, v_fd: u32, size: i64) -> RuntimeResult<()> {
    let fd = ctx.fdmap.fd_to_native(v_fd)?;
    let ret = trace_ftruncate(ctx, fd, size)?;
    Ok(())
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#fd_filestat_set_times
#[with_ghost_var(trace: &mut Trace)]
#[external_calls(from)]
#[external_methods(atim_now, atim, mtim, mtim_now, nsec)] // clock methods
#[external_methods(reserve_exact, push)] // Vec methods
#[external_calls(try_from)] // FstFlags methods
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_fd_filestat_set_times(
    ctx: &mut VmCtx,
    v_fd: u32,
    v_atim: u64,
    v_mtim: u64,
    v_fst_flags: u32,
) -> RuntimeResult<()> {
    let atim = Timestamp::new(v_atim);
    let mtim = Timestamp::new(v_mtim);
    let fst_flags = FstFlags::try_from(v_fst_flags as u16)?;
    let fd = ctx.fdmap.fd_to_native(v_fd)?;

    let mut specs: Vec<libc::timespec> = Vec::new();
    let atim_spec = atim.ts_to_native(fst_flags.atim(), fst_flags.atim_now());
    let mtim_spec = mtim.ts_to_native(fst_flags.mtim(), fst_flags.mtim_now());
    specs.push(atim_spec);
    specs.push(mtim_spec);

    let res = trace_futimens(ctx, fd, &specs)?;
    Ok(())
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#fd_pread
// modifies: mem
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(reserve_exact, push)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_fd_pread(
    ctx: &mut VmCtx,
    v_fd: u32,
    iovs: u32,
    iovcnt: u32,
    offset: u64,
) -> RuntimeResult<u32> {
    let fd = ctx.fdmap.fd_to_native(v_fd)?;

    let mut num: u32 = 0;
    let mut i = 0;
    while i < iovcnt {
        body_invariant!(ctx_safe(ctx));
        body_invariant!(trace_safe(trace, ctx));

        let start = (iovs + i * 8) as usize;

        let (ptr, len) = ctx.read_u32_pair(start)?;
        if !ctx.fits_in_lin_mem(ptr, len) {
            return Err(Efault);
        }
        let result = trace_pread(ctx, fd, ptr, len as usize, offset as usize)?;
        num += result as u32;
        i += 1;
    }
    Ok(num)
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#prestat_dirname
// modifies: mem
// If v_fd refers to a preopened directory (fd == 3), write the name to path
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(get_homedir)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_prestat_dirname(
    ctx: &mut VmCtx,
    v_fd: u32,
    path: u32,
    path_len: u32,
) -> RuntimeResult<()> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let dirname = ctx.get_homedir();
    let dirname_len = dirname.len() as u32;
    if !ctx.fits_in_lin_mem(path, dirname_len) {
        return Err(Efault);
    }

    ctx.copy_buf_to_sandbox(path, &dirname, dirname_len)?;
    Ok(())
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#fd_prestat_get
/// Must return ebadf if the file doesn't exist:
/// https://github.com/WebAssembly/wasi-libc/blob/ad5133410f66b93a2381db5b542aad5e0964db96/libc-bottom-half/sources/preopens.c#L212
#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_fd_prestat_get(ctx: &mut VmCtx, v_fd: u32) -> RuntimeResult<u32> {
    if v_fd == HOMEDIR_FD {
        return Ok(ctx.homedir.len() as u32);
    }
    Err(Ebadf)
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#fd_pwrite
// modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(push)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_fd_pwrite(
    ctx: &mut VmCtx,
    v_fd: u32,
    iovs: u32,
    iovcnt: u32,
    offset: u64,
) -> RuntimeResult<u32> {
    let fd = ctx.fdmap.fd_to_native(v_fd)?;

    let mut num: u32 = 0;
    let mut i = 0;
    while i < iovcnt {
        body_invariant!(ctx_safe(ctx));
        body_invariant!(trace_safe(trace, ctx));

        let start = (iovs + i * 8) as usize;

        let (ptr, len) = ctx.read_u32_pair(start)?;
        if !ctx.fits_in_lin_mem(ptr, len) {
            return Err(Efault);
        }
        let result = trace_pwrite(ctx, fd, ptr, len as usize, offset as usize)?;
        num += result as u32;
        i += 1;
    }
    Ok(num)
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#path_create_directory
// modifies: None
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(push)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_path_create_directory(
    ctx: &mut VmCtx,
    v_fd: u32,
    pathname: u32,
    path_len: u32,
) -> RuntimeResult<()> {
    // let fd = ctx.fdmap.fd_to_native(v_fd)?;

    if v_fd != HOMEDIR_FD {
        return Err(Enotcapable);
    }
    assert!(v_fd == HOMEDIR_FD);
    let fd = ctx.homedir_host_fd;

    // create directory follows symlinks
    let host_pathname = ctx.translate_path(pathname, path_len, true, fd);
    unwrap_result!(host_pathname);
    // wasi doesn't specify what permissions should be
    // We use rw------- cause it seems sane.
    let res = trace_mkdirat(ctx, fd, host_pathname, 0o766)?;
    Ok(())
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#path_filestat_get
// modifies: None
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(push, to_posix, should_follow, to_stat_posix)]
#[external_calls(fresh_stat, flag_set)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_path_filestat_get(
    ctx: &VmCtx,
    v_fd: u32,
    flags: u32,
    pathname: u32,
    path_len: u32,
) -> RuntimeResult<FileStat> {
    let flags = LookupFlags::new(flags);
    let fd = ctx.fdmap.fd_to_native(v_fd)?;
    if v_fd != HOMEDIR_FD {
        return Err(Enotcapable);
    }
    assert!(v_fd == HOMEDIR_FD);
    let fd = ctx.homedir_host_fd;

    let should_follow = flags.should_follow();

    let host_pathname = ctx.translate_path(pathname, path_len, should_follow, fd);
    unwrap_result!(host_pathname);

    let mut stat = fresh_stat();

    let n_flags = flags.to_stat_posix();

    if flag_set(n_flags, libc::AT_SYMLINK_NOFOLLOW) == should_follow {
        // this should never happen, but adding an extra dynamic check let me
        // avoid reasoning about bitwise operation math (which prusti does not support well)
        // in the proof
        return Err(Einval);
    }

    let res = trace_fstatat(ctx, fd, host_pathname, &mut stat, n_flags)?;
    Ok(stat.into())
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#path_filestat_set_times
// modifies: None
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(atim_now, atim, mtim, mtim_now, nsec)]
#[external_methods(reserve_exact, push, should_follow, to_stat_posix)]
#[external_calls(try_from, flag_set)] // FstFlags methods
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_path_filestat_set_times(
    ctx: &VmCtx,
    v_fd: u32,
    flags: u32,
    pathname: u32,
    path_len: u32,
    atim: u64,
    mtim: u64,
    v_fst_flags: u32,
) -> RuntimeResult<()> {
    let fst_flags = FstFlags::try_from(v_fst_flags as u16)?;
    let atim = Timestamp::new(atim);
    let mtim = Timestamp::new(mtim);
    let flags = LookupFlags::new(flags);

    // TODO: should be bundled into fstflags conversion
    if fst_flags.atim() && fst_flags.atim_now() || fst_flags.mtim() && fst_flags.mtim_now() {
        return Err(Einval);
    }

    // let fd = ctx.fdmap.fd_to_native(v_fd)?;
    if v_fd != HOMEDIR_FD {
        return Err(Enotcapable);
    }
    assert!(v_fd == HOMEDIR_FD);
    let fd = ctx.homedir_host_fd;

    let should_follow = flags.should_follow();
    let host_pathname = ctx.translate_path(pathname, path_len, should_follow, fd);
    unwrap_result!(host_pathname);

    let mut specs: Vec<libc::timespec> = Vec::new();
    let atim_spec = atim.ts_to_native(fst_flags.atim(), fst_flags.atim_now());
    let mtim_spec = mtim.ts_to_native(fst_flags.mtim(), fst_flags.mtim_now());
    specs.push(atim_spec);
    specs.push(mtim_spec);

    let n_flags = flags.to_stat_posix();

    if flag_set(n_flags, libc::AT_SYMLINK_NOFOLLOW) == should_follow {
        // this should never happen, but adding an extra dynamic check let me
        // avoid reasoning about bitwise operation math (which prusti does not support well)
        // in the proof
        return Err(Einval);
    }

    let res = trace_utimensat(ctx, fd, host_pathname, &specs, n_flags)?;

    Ok(())
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#path_link
// modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(to_linkat_posix, should_follow)]
#[external_calls(flag_set)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_path_link(
    ctx: &VmCtx,
    v_old_fd: u32,
    flags: u32,
    old_pathname: u32,
    old_path_len: u32,
    v_new_fd: u32,
    new_pathname: u32,
    new_path_len: u32,
) -> RuntimeResult<()> {
    let flags = LookupFlags::new(flags);

    // let old_fd = ctx.fdmap.fd_to_native(v_old_fd)?;
    // let new_fd = ctx.fdmap.fd_to_native(v_new_fd)?;

    if v_old_fd != HOMEDIR_FD {
        return Err(Enotcapable);
    }
    assert!(v_old_fd == HOMEDIR_FD);
    let old_fd = ctx.homedir_host_fd;

    if v_new_fd != HOMEDIR_FD {
        return Err(Enotcapable);
    }
    assert!(v_new_fd == HOMEDIR_FD);
    let new_fd = ctx.homedir_host_fd;

    let should_follow = flags.should_follow();

    // when resolveing paths for path_link, we resolve the final symlink
    let old_host_pathname = ctx.translate_path(old_pathname, old_path_len, should_follow, old_fd);
    unwrap_result!(old_host_pathname);
    let new_host_pathname = ctx.translate_path(new_pathname, new_path_len, should_follow, new_fd);
    unwrap_result!(new_host_pathname);

    let n_flags = flags.to_linkat_posix();

    if flag_set(n_flags, libc::AT_SYMLINK_FOLLOW) != should_follow {
        // this should never happen, but adding an extra dynamic check let me
        // avoid reasoning about bitwise operation math (which prusti does not support well)
        // in the proof
        return Err(Einval);
    }

    let res = trace_linkat(
        ctx,
        old_fd,
        old_host_pathname,
        new_fd,
        new_host_pathname,
        n_flags,
    )?;
    Ok(())
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#path_readlink
// modifies: mem
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(reserve_exact, push)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_path_readlink(
    ctx: &mut VmCtx,
    v_fd: u32,
    pathname: u32,
    path_len: u32,
    ptr: u32,
    len: u32,
) -> RuntimeResult<u32> {
    // let fd = ctx.fdmap.fd_to_native(v_fd)?;
    if v_fd != HOMEDIR_FD {
        return Err(Enotcapable);
    }
    assert!(v_fd == HOMEDIR_FD);
    let fd = ctx.homedir_host_fd;

    let should_follow = false; // readlink never follows symlink (it reads it!)
                               // TODO: replace once we can support the ? again
    let host_pathname = ctx.translate_path(pathname, path_len, should_follow, fd);
    unwrap_result!(host_pathname);

    if !ctx.fits_in_lin_mem(ptr, len) {
        return Err(Efault);
    }

    let res = trace_readlinkat(ctx, fd, host_pathname, ptr, len as usize)?;
    let res = res as u32;
    Ok(res)
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#path_remove_directory
// modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_path_remove_directory(
    ctx: &mut VmCtx,
    v_fd: u32,
    pathname: u32,
    path_len: u32,
) -> RuntimeResult<()> {
    // let fd = ctx.fdmap.fd_to_native(v_fd)?;
    if v_fd != HOMEDIR_FD {
        return Err(Enotcapable);
    }
    assert!(v_fd == HOMEDIR_FD);
    let fd = ctx.homedir_host_fd;

    // unlinkat operates on symlinks
    let host_pathname = ctx.translate_path(pathname, path_len, false, fd);
    unwrap_result!(host_pathname);

    let res = trace_unlinkat(ctx, fd, host_pathname, libc::AT_REMOVEDIR);
    // posix spec allows unlinkat to return EEXIST for a non-empty directory
    // however, the wasi spec requires that ENOTEMPTY is returned
    // see: https://man7.org/linux/man-pages/man2/rmdir.2.html
    if let Err(Eexist) = res {
        return Err(RuntimeError::Enotempty);
    }
    res?;
    Ok(())
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#path_rename
// modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_path_rename(
    ctx: &VmCtx,
    v_old_fd: u32,
    old_pathname: u32,
    old_path_len: u32,
    v_new_fd: u32,
    new_pathname: u32,
    new_path_len: u32,
) -> RuntimeResult<()> {
    if v_old_fd != HOMEDIR_FD {
        return Err(Enotcapable);
    }
    let old_fd = ctx.homedir_host_fd;

    if v_new_fd != HOMEDIR_FD {
        return Err(Enotcapable);
    }
    let new_fd = ctx.homedir_host_fd;
    // let old_fd = ctx.fdmap.fd_to_native(v_old_fd)?;
    // let new_fd = ctx.fdmap.fd_to_native(v_old_fd)?;
    // rename does not follow terminal symlinks - it operates on symlinks directly
    let old_host_pathname = ctx.translate_path(old_pathname, old_path_len, false, old_fd);
    unwrap_result!(old_host_pathname);
    let new_host_pathname = ctx.translate_path(new_pathname, new_path_len, false, new_fd);
    unwrap_result!(new_host_pathname);

    let res = trace_renameat(ctx, old_fd, old_host_pathname, new_fd, new_host_pathname)?;
    Ok(())
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#path_symlink
//modifies: none
// TODO: we should not actually translate path2 I believe
#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_path_symlink(
    ctx: &VmCtx,
    old_pathname: u32,
    old_path_len: u32,
    v_fd: u32,
    new_pathname: u32,
    new_path_len: u32,
) -> RuntimeResult<()> {
    //let fd = ctx.fdmap.fd_to_native(v_fd)?;
    if v_fd != HOMEDIR_FD {
        return Err(Enotcapable);
    }
    assert!(v_fd == HOMEDIR_FD);
    let fd = ctx.homedir_host_fd;

    // when evaluating paths for path_symlink, we follow symlinks
    let old_host_pathname = ctx.translate_path(old_pathname, old_path_len, true, fd);
    unwrap_result!(old_host_pathname);
    let new_host_pathname = ctx.translate_path(new_pathname, new_path_len, true, fd);
    unwrap_result!(new_host_pathname);

    let res = trace_symlinkat(ctx, old_host_pathname, fd, new_host_pathname)?;
    Ok(())
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#path_unlink_file
#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_path_unlink_file(
    ctx: &mut VmCtx,
    v_fd: u32,
    pathname: u32,
    path_len: u32,
) -> RuntimeResult<()> {
    // let fd = ctx.fdmap.fd_to_native(v_fd)?;
    if v_fd != HOMEDIR_FD {
        return Err(Enotcapable);
    }
    assert!(v_fd == HOMEDIR_FD);
    let fd = ctx.homedir_host_fd;

    // unlink operates on symlinks (it is in fact the main way to delete symlinks)
    let host_pathname = ctx.translate_path(pathname, path_len, false, fd);
    unwrap_result!(host_pathname);

    let res = trace_unlinkat(ctx, fd, host_pathname, 0)?;
    Ok(())
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#clock_res_get
// modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[external_calls(from_u32, try_from)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_clock_res_get(ctx: &VmCtx, clock_id: u32) -> RuntimeResult<Timestamp> {
    let id = ClockId::try_from(clock_id)?;
    let mut spec = fresh_libc_timespec();

    let ret = trace_clock_get_res(ctx, id.into(), &mut spec)?;
    Ok(spec.into())
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#clock_time_get
// modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[external_calls(from_u32, try_from)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_clock_time_get(
    ctx: &VmCtx,
    clock_id: u32,
    _precision: u64, // ignored
) -> RuntimeResult<Timestamp> {
    let id = ClockId::try_from(clock_id)?;
    let mut spec = fresh_libc_timespec();

    let ret = trace_clock_get_time(ctx, id.into(), &mut spec)?;
    Ok(spec.into())
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#proc_exit
#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
#[ensures(effects!(old(trace), trace))]
pub fn wasi_proc_exit(ctx: &VmCtx, rval: u32) -> RuntimeResult<()> {
    Ok(())
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#proc_raise
#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
#[ensures(effects!(old(trace), trace))]
pub fn wasi_proc_raise(ctx: &VmCtx, signal: u32) -> RuntimeResult<()> {
    Ok(())
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#sched_yield
#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
#[ensures(effects!(old(trace), trace))]
pub fn wasi_sched_yield(ctx: &VmCtx) -> RuntimeResult<()> {
    Ok(())
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#random_get
// modifies: memory
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(reserve_exact)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_random_get(ctx: &mut VmCtx, ptr: u32, len: u32) -> RuntimeResult<()> {
    if !ctx.fits_in_lin_mem(ptr, len) {
        return Err(Efault);
    }

    let res = trace_getrandom(ctx, ptr, len as usize, 0)?;
    Ok(())
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#fd_renumber
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(shift)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
#[ensures(effects!(old(trace), trace))]
pub fn wasi_fd_renumber(ctx: &mut VmCtx, v_from: u32, v_to: u32) -> RuntimeResult<()> {
    if v_from >= MAX_SBOX_FDS || v_to >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }
    ctx.fdmap.shift(v_from, v_to);
    Ok(())
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#args_get
#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
// #[ensures(effects!(old(trace), trace))]
pub fn wasi_args_get(ctx: &mut VmCtx, argv: u32, argv_buf: u32) -> RuntimeResult<()> {
    // 1. copy argv_buffer
    let argv_buf_len = ctx.arg_buffer.len() as u32;
    ctx.copy_arg_buffer_to_sandbox(argv_buf, argv_buf_len)?;
    // 2. copy in argv
    let mut idx: usize = 0;
    let mut start: u32 = 0;
    let mut cursor: usize = 0;
    while idx < ctx.arg_buffer.len() {
        body_invariant!(ctx_safe(ctx));
        body_invariant!(trace_safe(trace, ctx));
        // We have found an argument either when we find a trailing space, or if we started an arg
        // and ran out of space
        if !ctx.fits_in_lin_mem_usize((argv as usize) + cursor, 8) {
            return Err(Eoverflow);
        }

        if ctx.arg_buffer[idx] == b'\0' {
            while idx < ctx.arg_buffer.len() && ctx.arg_buffer[idx] == b'\0' {
                idx += 1;
            } // scan past multiple spaces
            ctx.write_u32((argv as usize) + cursor, argv_buf + start);
            cursor += 4;
            start = idx as u32;
        }
        idx += 1;

        // we reached the end, so record the final arg
        if idx >= ctx.arg_buffer.len() {
            ctx.write_u32((argv as usize) + cursor, argv_buf + start);
        }
    }

    let argc = ctx.argc;
    // ensure the last entry is null
    if !ctx.fits_in_lin_mem_usize((argv as usize) + argc * 4, 8) {
        return Err(Eoverflow);
    }
    ctx.write_u32((argv as usize) + argc * 4, 0);
    Ok(())
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#environ_get
#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
// #[ensures(effects!(old(trace), trace))]
pub fn wasi_environ_get(ctx: &mut VmCtx, env: u32, env_buf: u32) -> RuntimeResult<()> {
    // 1. copy argv_buffer
    let env_buf_len = ctx.env_buffer.len() as u32;
    ctx.copy_environ_buffer_to_sandbox(env_buf, env_buf_len)?;
    // 2. copy in argv
    let mut idx: usize = 0;
    let mut start: u32 = 0;
    let mut cursor: usize = 0;
    while idx < ctx.env_buffer.len() {
        body_invariant!(ctx_safe(ctx));
        body_invariant!(trace_safe(trace, ctx));
        if !ctx.fits_in_lin_mem_usize((env as usize) + cursor, 8) {
            return Err(Eoverflow);
        }
        if ctx.env_buffer[idx] == b'\0' {
            while idx < ctx.env_buffer.len() && ctx.env_buffer[idx] == b'\0' {
                idx += 1;
            } // scan past multiple spaces

            ctx.write_u32((env as usize) + cursor, env_buf + start);
            cursor += 4;
            start = idx as u32;
        }
        idx += 1;
        if idx >= ctx.arg_buffer.len() {
            ctx.write_u32((env as usize) + cursor, env_buf + start);
        }
    }

    let envc = ctx.envc;
    // ensure the last entry is null
    if !ctx.fits_in_lin_mem_usize((env as usize) + envc * 4, 8) {
        return Err(Eoverflow);
    }
    ctx.write_u32((env as usize) + envc * 4, 0);
    Ok(())
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#arg_sizes_get
// modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
#[ensures(effects!(old(trace), trace))]
pub fn wasi_args_sizes_get(ctx: &VmCtx) -> RuntimeResult<(u32, u32)> {
    Ok((ctx.argc as u32, ctx.arg_buffer.len() as u32))
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#environ_sizes_get
// modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
#[ensures(effects!(old(trace), trace))]
pub fn wasi_environ_sizes_get(ctx: &VmCtx) -> RuntimeResult<(u32, u32)> {
    Ok((ctx.envc as u32, ctx.env_buffer.len() as u32))
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#sock_recv
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(reserve_exact)]
#[external_calls(reserve_exact, try_from)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_sock_recv(
    ctx: &mut VmCtx,
    v_fd: u32,
    ri_data: u32,
    ri_data_count: u32,
    ri_flags: u32,
) -> RuntimeResult<(u32, u32)> {
    let fd = ctx.fdmap.fd_to_native(v_fd)?;
    let ri_flags = RiFlags::try_from(ri_flags)?;

    let mut num: u32 = 0;
    let mut i = 0;
    while i < ri_data_count {
        body_invariant!(ctx_safe(ctx));
        body_invariant!(trace_safe(trace, ctx));

        let start = (ri_data + i * 8) as usize;

        let (ptr, len) = ctx.read_u32_pair(start)?;
        if !ctx.fits_in_lin_mem(ptr, len) {
            return Err(Efault);
        }

        let result = trace_recv(ctx, fd, ptr, len as usize, ri_flags.to_posix())?;
        let result = result as u32;

        num += result;
        i += 1;
    }
    // TODO: handle ro_flags
    //       It doesn't look like there is a good way to handle them other than to use linux
    //       recvmsg instead of recv, which is more complex
    Ok((num, 0))
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#sock_send
#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_sock_send(
    ctx: &mut VmCtx,
    v_fd: u32,
    si_data: u32,
    si_data_count: u32,
    si_flags: u32,
) -> RuntimeResult<u32> {
    let fd = ctx.fdmap.fd_to_native(v_fd)?;

    let mut num: u32 = 0;
    let mut i = 0;
    while i < si_data_count {
        body_invariant!(ctx_safe(ctx));
        body_invariant!(trace_safe(trace, ctx));

        let start = (si_data + i * 8) as usize;

        let (ptr, len) = ctx.read_u32_pair(start)?;
        if !ctx.fits_in_lin_mem(ptr, len) {
            return Err(Efault);
        }
        // Currently the flags field of trace_send must be set to 0:
        // https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md/#siflags
        let flags = 0;
        let result = trace_send(ctx, fd, ptr, len as usize, flags)?;
        num += result as u32;
        i += 1;
    }
    Ok(num)
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#sock_shutdown
// ensures: valid(v_fd) => trace = old(shutdown :: trace)
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(try_into)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]

pub fn wasi_sock_shutdown(ctx: &VmCtx, v_fd: u32, v_how: u32) -> RuntimeResult<()> {
    let fd = ctx.fdmap.fd_to_native(v_fd)?;
    let how = SdFlags::new(v_how);
    let posix_how = how.try_into()?;

    let res = trace_shutdown(ctx, fd, posix_how)?;
    Ok(())
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#poll_oneoff
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(
    subscription_clock_abstime,
    try_into,
    checked_sub,
    ok_or_else,
    push,
    map_err,
    map,
    unwrap_or
)]
#[external_calls(from_posix, from_poll_revents, Some)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_poll_oneoff(
    ctx: &mut VmCtx,
    in_ptr: u32,
    out_ptr: u32,
    nsubscriptions: u32,
) -> RuntimeResult<u32> {
    // list of clock subscription (userdata, timeout) pairs
    let mut timeouts = Vec::new();

    // Parallel vectors for fd subscriptions.
    // (userdata, typ) pairs are stored in fd_data, while the pollfds themselves are stored
    // in pollfds.
    let mut fd_data = Vec::new();
    let mut pollfds = Vec::new();

    // minimum timeout we found, for setting the poll syscall timeout
    let mut min_timeout = None;
    let precision = 0;

    parse_subscriptions(ctx, in_ptr, nsubscriptions, precision, &mut min_timeout, &mut timeouts, &mut pollfds, &mut fd_data)?;
    // Special case: If we only got Clock subscriptions, we have no pollfds to poll on, so it will
    //               immediatly return. Instead, we use nanosleep on the min timeout.
    let res = if pollfds.len() == 0 {
        let mut rem = fresh_libc_timespec();
        let timespec: libc::timespec = min_timeout.map(|t| t.into()).ok_or(Einval)?;
        trace_nanosleep(ctx, &timespec, &mut rem)?
    } else {
        let poll_timeout: i32 = min_timeout
            .map(|t| t.to_millis().try_into().map_err(|e| Eoverflow))
            .unwrap_or(Ok(-1))?;
        trace_poll(ctx, pollfds.as_mut(), poll_timeout)?
    };

    if res == 0 {
        // if res == 0, no pollfd events ocurred. Therefore, the timeout must have triggered,
        // meaning we only trigger clock events with the min_timeout
        writeback_timeouts(ctx, out_ptr, &timeouts, &min_timeout)
    } else {
        writeback_fds(ctx, out_ptr, &pollfds, &fd_data)
    }
}

// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#fd_readdir
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(extend_from_slice, reserve_exact)]
#[external_methods(to_le_bytes, to_wasi)]
#[external_calls(from_le_bytes, from, first_null, push_dirent_name, parse)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
// TODO: I'm not confident this works for multiple consecutive readdir calls to the same dir
// Correct behavior: truncate final entry
pub fn wasi_fd_readdir(
    ctx: &mut VmCtx,
    v_fd: SboxFd,
    buf: SboxFd,
    buf_len: usize,
    cookie: u64,
) -> RuntimeResult<u32> {
    let fd = ctx.fdmap.fd_to_native(v_fd)?;

    let mut host_buf: Vec<u8> = Vec::new();
    host_buf.reserve_exact(buf_len as usize);

    let res = trace_getdents64(ctx, fd, &mut host_buf, buf_len)?;

    // the number of entries we have read so far. If less than cookie, don't output the directory
    let mut entry_idx = 0;
    let mut in_idx = 0;
    let mut out_idx = 0;

    let mut out_buf: Vec<u8> = Vec::new();

    while in_idx < host_buf.len() && out_idx < buf_len {
        body_invariant!(ctx_safe(ctx));
        body_invariant!(trace_safe(trace, ctx));
        body_invariant!(in_idx < host_buf.len());

        let dirent = Dirent::parse(&host_buf, in_idx)?;

        // if we haven't hit the cookie entry, skip
        if entry_idx < cookie {
            in_idx += dirent.reclen as usize;
            entry_idx += 1;
            continue;
        }

        let out_next = in_idx + 24 + dirent.out_namlen;

        // If we would overflow - don't :)
        if out_next > buf_len {
            break;
        }

        // Copy in next offset verbatim
        let out_next_bytes: [u8; 8] = out_next.to_le_bytes();
        out_buf.extend_from_slice(&out_next_bytes);

        // Copy in Inode verbatim
        let d_ino_bytes: [u8; 8] = dirent.ino.to_le_bytes();
        out_buf.extend_from_slice(&d_ino_bytes);

        // Copy namlen
        let out_namlen_bytes: [u8; 4] = (dirent.out_namlen as u32).to_le_bytes();
        out_buf.extend_from_slice(&out_namlen_bytes);

        // Copy type
        let d_type = Filetype::from(dirent.typ as libc::mode_t);
        let out_type_bytes: [u8; 4] = (d_type.to_wasi() as u32).to_le_bytes();
        out_buf.extend_from_slice(&out_type_bytes);

        // Copy name
        push_dirent_name(
            &mut out_buf,
            &host_buf,
            in_idx + dirent.name_start,
            dirent.out_namlen,
        );

        in_idx += dirent.reclen as usize;
        out_idx += (24 + dirent.out_namlen) as usize
    }

    ctx.copy_buf_to_sandbox(buf, &out_buf, out_buf.len() as u32)?;

    Ok(out_buf.len() as u32)
}

// No spec for this one since we added it
#[with_ghost_var(trace: &mut Trace)]
#[external_calls(sock_domain_to_posix, sock_type_to_posix)]
#[external_methods(create_sock)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_socket(ctx: &mut VmCtx, domain: u32, ty: u32, protocol: u32) -> RuntimeResult<u32> {
    // We only allow TCP and UDP, which can both be identified using protocol=0 when
    // domain.ty are (AF_INET,SOCK_STREAM) or (AF_INET,SOCK_DGRAM) respectively
    if protocol != 0 {
        return Err(Einval);
    }

    let protocol = protocol as i32;
    // convert from wasi constants to posix constants
    let domain = sock_domain_to_posix(domain)?;
    let ty = sock_type_to_posix(ty)?;

    let wasi_proto = WasiProto::new(domain, ty, protocol);
    if matches!(wasi_proto, WasiProto::Unknown) {
        return Err(Einval);
    }
    if !(domain == libc::AF_INET && (ty == libc::SOCK_STREAM || ty == libc::SOCK_DGRAM)) {
        return Err(Einval);
    }

    let res = trace_socket(ctx, domain, ty, protocol)?;

    ctx.fdmap.create_sock(HostFd::from_raw(res), wasi_proto)
    // ctx.fdmap.create(res.into())
}

// No spec for this one since we added it
#[with_ghost_var(trace: &mut Trace)]
#[external_calls(sock_domain_to_posix, from, addr_in_netlist)]
//#[external_calls(addr_in_netlist)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
pub fn wasi_sock_connect(
    ctx: &mut VmCtx,
    sockfd: u32,
    addr: u32,
    addrlen: u32,
) -> RuntimeResult<()> {
    let fd = ctx.fdmap.fd_to_native(sockfd)?;

    if addrlen != 16 {
        return Err(Einval);
    }

    // since we only support inet4, addrlen will always be 16
    if addrlen != 16 {
        return Err(Einval);
    }

    if !ctx.fits_in_lin_mem(addr, addrlen) {
        return Err(Eoverflow);
    }

    let sin_family = ctx.read_u16(addr as usize);
    let sin_port = ctx.read_u16(addr as usize + 2);
    let sin_addr_in = ctx.read_u32(addr as usize + 4);
    let sin_family = sock_domain_to_posix(sin_family as u32)? as libc::sa_family_t;
    // We can directly use sockaddr_in since we already know all socks are inet
    let sin_addr = libc::in_addr {
        s_addr: sin_addr_in,
    };
    let saddr = libc::sockaddr_in {
        // i'll be lazy, should refactor to os-specific code...
        #[cfg(target_os = "macos")]
        sin_len: 0,
        sin_family,
        sin_port,
        sin_addr,
        sin_zero: [0; 8],
    };

    let protocol = ctx.fdmap.sockinfo[fd.to_raw()]?;
    if matches!(protocol, WasiProto::Unknown) {
        return Err(Enotcapable);
    }

    if !addr_in_netlist(&ctx.netlist, sin_addr_in, sin_port as u32) {
        return Err(Enotcapable);
    }

    let res = trace_connect(ctx, fd, &saddr, addrlen)?;
    Ok(())
}
