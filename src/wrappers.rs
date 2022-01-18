use crate::os::*;
use crate::runtime::*;
use crate::tcb::misc::{bitwise_or, first_null, fresh_stat, push_dirent_name};
#[cfg(feature = "verify")]
use crate::tcb::verifier::external_specs::result::*;
#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::types::*;
use crate::{effect, no_effect, one_effect};
use extra_args::{external_calls, external_methods, with_ghost_var};
use prusti_contracts::*;
use std::convert::{TryFrom, TryInto};
use std::mem;
use RuntimeError::*;

// Note: Prusti can't really handle iterators, so we need to use while loops

// Modifies: fdmap
// TODO: fdmap trace fix
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(resolve_path, create, to_posix, to_os_flags)]
#[external_calls(from, bitwise_or)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
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

    if !ctx.fits_in_lin_mem(pathname, path_len) {
        return Err(Eoverflow);
    }

    if v_dir_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_dir_fd as usize]?;

    let host_buffer = ctx.copy_buf_from_sandbox(pathname, path_len);
    let host_pathname = ctx.resolve_path(host_buffer)?;
    let dirflags_posix = dirflags.to_posix();
    let oflags_posix = oflags.to_posix();
    let fdflags_posix = fdflags.to_os_flags();
    let flags = bitwise_or(
        bitwise_or(dirflags.to_posix(), oflags.to_posix()),
        fdflags.to_os_flags(),
    );

    let fd = trace_openat(ctx, fd, host_pathname, flags)?;
    ctx.fdmap.create(fd.into())
}

// modifies: fdmap
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(delete)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
// if args are not valid, nothing happens
#[ensures(v_fd >= MAX_SBOX_FDS ==> no_effect!(old(trace), trace))]
// #[ensures(v_fd < MAX_SBOX_FDS && old(!ctx.fdmap.contains(v_fd)) ==> no_effect!(old(trace), trace))]
// if args are valid, we do invoke an effect
//#[ensures( (v_fd < MAX_SBOX_FDS && ctx.fdmap.contains(v_fd)) ==> one_effect!(trace, old(trace), Effect::FdAccess) )]
pub fn wasi_fd_close(ctx: &mut VmCtx, v_fd: u32) -> RuntimeResult<u32> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }
    let fd = ctx.fdmap.m[v_fd as usize]?;
    ctx.fdmap.delete(v_fd);
    let result = trace_close(ctx, fd)?;
    Ok(result as u32)
}

// modifies: mem
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(reserve_exact, map_err)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
pub fn wasi_fd_read(ctx: &mut VmCtx, v_fd: u32, iovs: u32, iovcnt: u32) -> RuntimeResult<u32> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    let mut num: u32 = 0;
    let mut i = 0;
    while i < iovcnt {
        body_invariant!(trace_safe(trace, ctx.memlen) && ctx_safe(ctx));

        let start = (iovs + i * 8) as usize;
        if !ctx.fits_in_lin_mem_usize(start, 8) {
            return Err(Eoverflow);
        }
        let ptr = ctx.read_u32(start);
        let len = ctx.read_u32(start + 4);
        if !ctx.fits_in_lin_mem(ptr, len) {
            return Err(Efault);
        }

        let result = trace_read(ctx, fd, ptr, len as usize)?;
        num += result as u32;
        i += 1;
    }
    Ok(num)
}

// modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
pub fn wasi_fd_write(ctx: &mut VmCtx, v_fd: u32, iovs: u32, iovcnt: u32) -> RuntimeResult<u32> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    let mut num: u32 = 0;
    let mut i = 0;
    while i < iovcnt {
        body_invariant!(trace_safe(trace, ctx.memlen) && ctx_safe(ctx));

        let start = (iovs + i * 8) as usize;
        if !ctx.fits_in_lin_mem_usize(start, 8) {
            return Err(Eoverflow);
        }
        let ptr = ctx.read_u32(start);
        let len = ctx.read_u32(start + 4);
        if !ctx.fits_in_lin_mem(ptr, len) {
            return Err(Efault);
        }

        let result = trace_write(ctx, fd, ptr, len as usize)?;
        num += result as u32;
        i += 1;
    }
    Ok(num)
}

// modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[external_calls(from_u32)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
// #[ensures(v_fd < MAX_SBOX_FDS && ctx.fdmap.contains(v_fd) ==> one_effect!(old(trace), trace, Effect::FdAccess))]
// #[ensures(v_fd >= MAX_SBOX_FDS ==> no_effect!(old(trace), trace))]
pub fn wasi_fd_seek(ctx: &VmCtx, v_fd: u32, v_filedelta: i64, v_whence: u32) -> RuntimeResult<u32> {
    let whence = Whence::from_u32(v_whence).ok_or(Einval)?;

    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    let ret = trace_seek(ctx, fd, v_filedelta, whence.into())?;
    Ok(ret as u32)
}

// modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
pub fn wasi_fd_tell(ctx: &VmCtx, v_fd: u32) -> RuntimeResult<u32> {
    wasi_fd_seek(ctx, v_fd, 0, 1) // Whence::Cur
}

// modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[external_calls(try_from)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
pub fn wasi_fd_advise(
    ctx: &VmCtx,
    v_fd: u32,
    offset: u64,
    len: u64,
    v_advice: u32,
) -> RuntimeResult<u32> {
    let advice = Advice::try_from(v_advice as i32)?;

    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    // these casts could cause offset and len to become negative
    // I don't think this will be an issue as os_advise will throw an EINVAL error
    let ret = trace_advise(ctx, fd, offset as i64, len as i64, advice.into())?;
    Ok(ret as u32)
}

// modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
pub fn wasi_fd_allocate(ctx: &VmCtx, v_fd: u32, offset: u64, len: u64) -> RuntimeResult<u32> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    // these casts could cause offset and len to become negative
    // I don't think this will be an issue as os_advise will throw an EINVAL error
    let ret = trace_allocate(ctx, fd, offset as i64, len as i64)?;
    Ok(ret as u32)
}

// modifies: none
// TODO: should not return u32 at all?
#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
pub fn wasi_fd_sync(ctx: &VmCtx, v_fd: u32) -> RuntimeResult<u32> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    let ret = trace_sync(ctx, fd)?;
    Ok(ret as u32)
}

// // modifies: None
#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
pub fn wasi_fd_datasync(ctx: &VmCtx, v_fd: u32) -> RuntimeResult<u32> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    let ret = trace_datasync(ctx, fd)?;
    Ok(ret as u32)
}

//modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[external_calls(fresh_stat, from_posix, from_os_flags)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
pub fn wasi_fd_fdstat_get(ctx: &VmCtx, v_fd: u32) -> RuntimeResult<FdStat> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;

    let mut stat = fresh_stat();
    // TODO: double check, should this be fstat or fstat64?
    let result = trace_fstat(ctx, fd, &mut stat)?;
    let filetype = stat.st_mode;

    let mode_flags = trace_fgetfl(ctx, fd)?;

    // TODO: put rights in once those are implemented
    let result = FdStat {
        fs_filetype: (filetype as libc::mode_t).into(),
        fs_flags: FdFlags::from_os_flags(mode_flags as i32), //(mode_flags as libc::c_int).into(),
        fs_rights_base: 0, // TODO: convert read and write from mode flags to the proper masks?
        fs_rights_inheriting: u64::MAX, //TODO: we should pass in homedir rights
    };
    Ok(result)
}

//TODO: need wasm layout for FdFlags to read from ptr
#[with_ghost_var(trace: &mut Trace)]
#[external_calls(from)]
#[external_methods(to_os_flags)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
// can only adjust Fdflags using set_flags, not O_flags or any other flags
pub fn wasi_fd_fdstat_set_flags(ctx: &mut VmCtx, v_fd: u32, v_flags: u32) -> RuntimeResult<()> {
    let flags = FdFlags::from(v_flags as i32);

    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    let posix_flags = flags.to_os_flags();

    let ret = trace_fsetfl(ctx, fd, posix_flags)?;
    Ok(())
}

// // modifies: None
#[with_ghost_var(trace: &mut Trace)]
#[external_calls(fresh_stat)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
pub fn wasi_fd_filestat_get(ctx: &VmCtx, v_fd: u32) -> RuntimeResult<FileStat> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    let mut stat = fresh_stat();

    let filetype = trace_fstat(ctx, fd, &mut stat)?;
    Ok(stat.into())
}

// modifies: none
// Note: WASI API says that size should be u64.
// but ftruncate, which filestat_set_size is supposed to call used i64
// I'm keeping this as i64 since this does not cause any truncation
#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
pub fn wasi_fd_filestat_set_size(ctx: &VmCtx, v_fd: u32, size: i64) -> RuntimeResult<()> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    let ret = trace_ftruncate(ctx, fd, size)?;
    Ok(())
}

#[with_ghost_var(trace: &mut Trace)]
#[external_methods(atim_now, atim, mtim, mtim_now, nsec)] // clock methods
#[external_methods(reserve_exact, push)] // Vec methods
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
pub fn wasi_fd_filestat_set_times(
    ctx: &mut VmCtx,
    v_fd: u32,
    v_atim: u64,
    v_mtim: u64,
    v_fst_flags: u32,
) -> RuntimeResult<()> {
    let atim = Timestamp::new(v_atim);
    let mtim = Timestamp::new(v_mtim);
    let fst_flags = FstFlags::new(v_fst_flags as u16);

    // if fst_flags.atim() && fst_flags.atim_now() || fst_flags.mtim() && fst_flags.mtim_now() {
    //     return Err(Einval);
    // }

    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    // TODO: should inval clock be handled in higher level, or have Unkown ClockId variant
    //       and handle here?
    // TODO: how to handle `precision` arg? Looks like some runtimes ignore it...

    let mut specs: Vec<libc::timespec> = Vec::new();
    specs.reserve_exact(2);
    let atim_nsec = if fst_flags.atim() {
        atim.nsec() as i64
    } else {
        if fst_flags.atim_now() {
            libc::UTIME_NOW
        } else {
            libc::UTIME_OMIT
        }
    };
    let atim_spec = libc::timespec {
        tv_sec: 0,
        tv_nsec: atim_nsec,
    };

    let mtim_nsec = if fst_flags.mtim() {
        mtim.nsec() as i64
    } else if fst_flags.mtim_now() {
        libc::UTIME_NOW
    } else {
        libc::UTIME_OMIT
    };
    let mtim_spec = libc::timespec {
        tv_sec: 0,
        tv_nsec: mtim_nsec,
    };

    specs.push(atim_spec);
    specs.push(mtim_spec);

    let res = trace_futimens(ctx, fd, &specs)?;

    Ok(())
}

// modifies: mem
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(reserve_exact, push, resolve_path)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
pub fn wasi_fd_pread(
    ctx: &mut VmCtx,
    v_fd: u32,
    iovs: u32,
    iovcnt: u32,
    offset: u64,
) -> RuntimeResult<u32> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    let mut num: u32 = 0;
    let mut i = 0;
    while i < iovcnt {
        body_invariant!(trace_safe(trace, ctx.memlen) && ctx_safe(ctx));

        let start = (iovs + i * 8) as usize;
        if !ctx.fits_in_lin_mem_usize(start, 8) {
            return Err(Eoverflow);
        }
        let ptr = ctx.read_u32(start);
        let len = ctx.read_u32(start + 4);
        if !ctx.fits_in_lin_mem(ptr, len) {
            return Err(Efault);
        }
        let result = trace_pread(ctx, fd, ptr, len as usize, offset as usize)?;
        num += result as u32;
        i += 1;
    }
    Ok(num)
}

// modifies: mem
// If v_fd refers to a preopened directory (fd == 3), write the name to path
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(get_homedir)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
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

    let copy_ok = ctx
        .copy_buf_to_sandbox(path, &dirname, dirname_len)
        .ok_or(Efault)?;
    Ok(())
}

/// Must return ebadf if the file doesn't exist:
/// https://github.com/WebAssembly/wasi-libc/blob/ad5133410f66b93a2381db5b542aad5e0964db96/libc-bottom-half/sources/preopens.c#L212
#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
pub fn wasi_fd_prestat_get(ctx: &mut VmCtx, v_fd: u32) -> RuntimeResult<u32> {
    if v_fd == HOMEDIR_FD {
        return Ok(ctx.homedir.len() as u32);
    }
    return Err(Ebadf);
}

// modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(push, resolve_path)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
pub fn wasi_fd_pwrite(
    ctx: &mut VmCtx,
    v_fd: u32,
    iovs: u32,
    iovcnt: u32,
    offset: u64,
) -> RuntimeResult<u32> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    let mut num: u32 = 0;
    let mut i = 0;
    while i < iovcnt {
        body_invariant!(trace_safe(trace, ctx.memlen) && ctx_safe(ctx));

        let start = (iovs + i * 8) as usize;
        if !ctx.fits_in_lin_mem_usize(start, 8) {
            return Err(Eoverflow);
        }
        let ptr = ctx.read_u32(start);
        let len = ctx.read_u32(start + 4);
        if !ctx.fits_in_lin_mem(ptr, len) {
            return Err(Efault);
        }
        let result = trace_pwrite(ctx, fd, ptr, len as usize, offset as usize)?;
        num += result as u32;
        i += 1;
    }
    Ok(num)
}

// TODO: should create fd for directory
// modifies: adds hostfd for directory created
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(push, resolve_path)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
pub fn wasi_path_create_directory(
    ctx: &mut VmCtx,
    v_fd: u32,
    pathname: u32,
    path_len: u32,
) -> RuntimeResult<()> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    if !ctx.fits_in_lin_mem(pathname, path_len) {
        return Err(Eoverflow);
    }
    let host_buffer = ctx.copy_buf_from_sandbox(pathname, path_len);
    let host_pathname = ctx.resolve_path(host_buffer)?;
    let fd = ctx.fdmap.m[v_fd as usize]?;
    // TODO: wasi doesn't seem so specify what permissions should be?
    //       I will use rw------- cause it seems sane.
    let mode = libc::S_IRUSR + libc::S_IWUSR; // using add cause | isn't supported
    let res = trace_mkdirat(ctx, fd, host_pathname, mode)?;
    Ok(())
}

// TODO: this needs to make sure that the pathname is relative. If pathname is abosolute it won't
//       respect the fd.
// modifies: None
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(push, resolve_path)]
#[external_calls(fresh_stat)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
pub fn wasi_path_filestat_get(
    ctx: &VmCtx,
    v_fd: u32,
    flags: u32,
    pathname: u32,
    path_len: u32,
) -> RuntimeResult<FileStat> {
    let flags = LookupFlags::new(flags);
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    if !ctx.fits_in_lin_mem(pathname, path_len) {
        return Err(Eoverflow);
    }

    let host_buffer = ctx.copy_buf_from_sandbox(pathname, path_len);
    let host_pathname = ctx.resolve_path(host_buffer)?;
    let fd = ctx.fdmap.m[v_fd as usize]?;
    let mut stat = fresh_stat();

    let res = trace_fstatat(ctx, fd, host_pathname, &mut stat, flags.to_posix())?;
    Ok(stat.into())
}

// modifies: None
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(atim_now, atim, mtim, mtim_now, nsec)]
#[external_methods(reserve_exact, push)]
#[external_methods(resolve_path)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
pub fn wasi_path_filestat_set_times(
    ctx: &VmCtx,
    v_fd: u32,
    flags: u32,
    pathname: u32,
    path_len: u32,
    atim: u64,
    mtim: u64,
    fst_flags: FstFlags,
) -> RuntimeResult<()> {
    let atim = Timestamp::new(atim);
    let mtim = Timestamp::new(mtim);
    let flags = LookupFlags::new(flags);

    if fst_flags.atim() && fst_flags.atim_now() || fst_flags.mtim() && fst_flags.mtim_now() {
        return Err(Einval);
    }

    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    if !ctx.fits_in_lin_mem(pathname, path_len) {
        return Err(Eoverflow);
    }

    let host_buffer = ctx.copy_buf_from_sandbox(pathname, path_len);
    let host_pathname = ctx.resolve_path(host_buffer)?;
    let fd = ctx.fdmap.m[v_fd as usize]?;

    let mut specs: Vec<libc::timespec> = Vec::new();
    specs.reserve_exact(2);
    let atim_nsec = if fst_flags.atim() {
        atim.nsec() as i64
    } else {
        if fst_flags.atim_now() {
            libc::UTIME_NOW
        } else {
            libc::UTIME_OMIT
        }
    };
    let atim_spec = libc::timespec {
        tv_sec: 0,
        tv_nsec: atim_nsec,
    };

    let mtim_nsec = if fst_flags.mtim() {
        mtim.nsec() as i64
    } else if fst_flags.mtim_now() {
        libc::UTIME_NOW
    } else {
        libc::UTIME_OMIT
    };
    let mtim_spec = libc::timespec {
        tv_sec: 0,
        tv_nsec: mtim_nsec,
    };

    specs.push(atim_spec);
    specs.push(mtim_spec);

    let res = trace_utimensat(ctx, fd, host_pathname, &specs, flags.to_posix())?;

    Ok(())
}

// TODO: same caveat as wasi_path_filestat_get in terms of relative and absolute path.
// modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(resolve_path)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
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

    if v_old_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }
    if v_new_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }
    if !ctx.fits_in_lin_mem(old_pathname, old_path_len) {
        return Err(Eoverflow);
    }
    if !ctx.fits_in_lin_mem(new_pathname, new_path_len) {
        return Err(Eoverflow);
    }

    let old_host_buffer = ctx.copy_buf_from_sandbox(old_pathname, old_path_len);
    let old_host_pathname = ctx.resolve_path(old_host_buffer)?;
    let new_host_buffer = ctx.copy_buf_from_sandbox(new_pathname, new_path_len);
    let new_host_pathname = ctx.resolve_path(new_host_buffer)?;
    let old_fd = ctx.fdmap.m[v_old_fd as usize]?;
    let new_fd = ctx.fdmap.m[v_new_fd as usize]?;

    let res = trace_linkat(
        ctx,
        old_fd,
        old_host_pathname,
        new_fd,
        new_host_pathname,
        flags.to_posix(),
    )?;
    Ok(())
}

// modifies: mem
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(reserve_exact, push)]
#[external_methods(resolve_path)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
pub fn wasi_path_readlink(
    ctx: &mut VmCtx,
    v_fd: u32,
    pathname: u32,
    path_len: u32,
    ptr: u32,
    len: u32,
) -> RuntimeResult<u32> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }
    if !ctx.fits_in_lin_mem(pathname, path_len) {
        return Err(Eoverflow);
    }

    let host_buffer = ctx.copy_buf_from_sandbox(pathname, path_len);
    let host_pathname = ctx.resolve_path(host_buffer)?;
    let fd = ctx.fdmap.m[v_fd as usize]?;

    if !ctx.fits_in_lin_mem(ptr, len) {
        return Err(Efault);
    }

    let res = trace_readlinkat(ctx, fd, host_pathname, ptr, len as usize)?;
    let res = res as u32;
    Ok(res)
}

//TODO: should remove fd from map?
//modifies: removes directory from fdmap
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(resolve_path)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
pub fn wasi_path_remove_directory(
    ctx: &mut VmCtx,
    v_fd: u32,
    pathname: u32,
    path_len: u32,
) -> RuntimeResult<()> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }
    if !ctx.fits_in_lin_mem(pathname, path_len) {
        return Err(Eoverflow);
    }

    let host_buffer = ctx.copy_buf_from_sandbox(pathname, path_len);
    let host_pathname = ctx.resolve_path(host_buffer)?;
    let fd = ctx.fdmap.m[v_fd as usize]?;

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

// // modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(resolve_path)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
pub fn wasi_path_rename(
    ctx: &VmCtx,
    v_old_fd: u32,
    old_pathname: u32,
    old_path_len: u32,
    v_new_fd: u32,
    new_pathname: u32,
    new_path_len: u32,
) -> RuntimeResult<()> {
    if v_old_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }
    if v_new_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }
    if !ctx.fits_in_lin_mem(old_pathname, old_path_len) {
        return Err(Eoverflow);
    }
    if !ctx.fits_in_lin_mem(new_pathname, new_path_len) {
        return Err(Eoverflow);
    }

    let old_host_buffer = ctx.copy_buf_from_sandbox(old_pathname, old_path_len);
    let old_host_pathname = ctx.resolve_path(old_host_buffer)?;
    let new_host_buffer = ctx.copy_buf_from_sandbox(new_pathname, new_path_len);
    let new_host_pathname = ctx.resolve_path(new_host_buffer)?;
    let old_fd = ctx.fdmap.m[v_old_fd as usize]?;
    let new_fd = ctx.fdmap.m[v_new_fd as usize]?;

    let res = trace_renameat(ctx, old_fd, old_host_pathname, new_fd, new_host_pathname)?;
    Ok(())
}

//modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(resolve_path)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
pub fn wasi_path_symlink(
    ctx: &VmCtx,
    old_pathname: u32,
    old_path_len: u32,
    v_fd: u32,
    new_pathname: u32,
    new_path_len: u32,
) -> RuntimeResult<()> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }
    if !ctx.fits_in_lin_mem(old_pathname, old_path_len) {
        return Err(Eoverflow);
    }
    if !ctx.fits_in_lin_mem(new_pathname, new_path_len) {
        return Err(Eoverflow);
    }

    let old_host_buffer = ctx.copy_buf_from_sandbox(old_pathname, old_path_len);
    let old_host_pathname = ctx.resolve_path(old_host_buffer)?;
    let new_host_buffer = ctx.copy_buf_from_sandbox(new_pathname, new_path_len);
    let new_host_pathname = ctx.resolve_path(new_host_buffer)?;
    let fd = ctx.fdmap.m[v_fd as usize]?;

    let res = trace_symlinkat(ctx, old_host_pathname, fd, new_host_pathname)?;
    Ok(())
}

#[with_ghost_var(trace: &mut Trace)]
#[external_methods(resolve_path)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
pub fn wasi_path_unlink_file(
    ctx: &mut VmCtx,
    v_fd: u32,
    pathname: u32,
    path_len: u32,
) -> RuntimeResult<()> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }
    if !ctx.fits_in_lin_mem(pathname, path_len) {
        return Err(Eoverflow);
    }

    let host_buffer = ctx.copy_buf_from_sandbox(pathname, path_len);
    let host_pathname = ctx.resolve_path(host_buffer)?;
    let fd = ctx.fdmap.m[v_fd as usize]?;

    let res = trace_unlinkat(ctx, fd, host_pathname, 0)?;
    Ok(())
}

// modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[external_calls(from_u32)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
pub fn wasi_clock_res_get(ctx: &VmCtx, clock_id: u32) -> RuntimeResult<Timestamp> {
    let id = ClockId::from_u32(clock_id).ok_or(Einval)?;

    let mut spec = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };

    let ret = trace_clock_get_res(ctx, id.into(), &mut spec)?;
    Ok(spec.into())
}

// modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[external_calls(from_u32)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
pub fn wasi_clock_time_get(
    ctx: &VmCtx,
    clock_id: u32,
    //precision: Timestamp,
) -> RuntimeResult<Timestamp> {
    let id = ClockId::from_u32(clock_id).ok_or(Einval)?;
    // TODO: how to handle `precision` arg? Looks like some runtimes ignore it...
    let mut spec = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };

    let ret = trace_clock_get_time(ctx, id.into(), &mut spec)?;
    Ok(spec.into())
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(no_effect!(old(trace), trace))]
pub fn wasi_proc_exit(ctx: &VmCtx, rval: u32) -> RuntimeResult<()> {
    Ok(())
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(no_effect!(old(trace), trace))]
pub fn wasi_proc_raise(ctx: &VmCtx, signal: u32) -> RuntimeResult<()> {
    Ok(())
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(no_effect!(old(trace), trace))]
pub fn wasi_sched_yield(ctx: &VmCtx) -> RuntimeResult<()> {
    Ok(())
}

// modifies: memory
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(reserve_exact)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
pub fn wasi_random_get(ctx: &mut VmCtx, ptr: u32, len: u32) -> RuntimeResult<()> {
    if !ctx.fits_in_lin_mem(ptr, len) {
        return Err(Efault);
    }

    let res = trace_getrandom(ctx, ptr, len as usize, 0)?;
    Ok(())
}

#[with_ghost_var(trace: &mut Trace)]
#[external_methods(shift)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(no_effect!(old(trace), trace))]
pub fn wasi_fd_renumber(ctx: &mut VmCtx, v_from: u32, v_to: u32) -> RuntimeResult<()> {
    // 1. translate from fd
    if v_from >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }
    // 2. translate to fd
    if v_to >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }
    // 3. delete from with to
    ctx.fdmap.shift(v_from, v_to);
    Ok(())
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
// #[ensures(no_effect!(old(trace), trace))]
pub fn wasi_args_get(ctx: &mut VmCtx, argv: u32, argv_buf: u32) -> RuntimeResult<()> {
    // 1. copy argv_buffer
    let argv_buf_len = ctx.arg_buffer.len() as u32;
    ctx.copy_arg_buffer_to_sandbox(argv_buf, argv_buf_len)
        .ok_or(Efault)?;
    // 2. copy in argv
    let mut idx: usize = 0;
    let mut start: u32 = 0;
    let mut cursor: usize = 0;
    while idx < ctx.arg_buffer.len() {
        body_invariant!(trace_safe(trace, ctx.memlen) && ctx_safe(ctx));
        // body_invariant!(idx < ctx.arg_buffer.len());
        //body_invariant!(idx * 4 < cursor);
        // We have found an argument either when we find a trailing space, or if we started an arg
        // and ran out of space
        if !ctx.fits_in_lin_mem_usize((argv as usize) + cursor, 8) {
            return Err(Eoverflow);
        }

        if ctx.arg_buffer[idx] == b'\0' {
            while ctx.arg_buffer[idx] == b'\0' {
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
    Ok(())
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
// #[ensures(no_effect!(old(trace), trace))]
pub fn wasi_environ_get(ctx: &mut VmCtx, env: u32, env_buf: u32) -> RuntimeResult<()> {
    // 1. copy argv_buffer
    let env_buf_len = ctx.env_buffer.len() as u32;
    ctx.copy_environ_buffer_to_sandbox(env_buf, env_buf_len)
        .ok_or(Efault)?;
    // 2. copy in argv
    let mut idx: usize = 0;
    let mut start: u32 = 0;
    let mut cursor: usize = 0;
    while idx < ctx.env_buffer.len() {
        body_invariant!(trace_safe(trace, ctx.memlen) && ctx_safe(ctx));
        if !ctx.fits_in_lin_mem_usize((env as usize) + cursor, 8) {
            return Err(Eoverflow);
        }
        if ctx.env_buffer[idx] == b'\0' {
            while ctx.env_buffer[idx] == b'\0' {
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
    Ok(())
}

// modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(no_effect!(old(trace), trace))]
pub fn wasi_args_sizes_get(ctx: &VmCtx) -> RuntimeResult<(u32, u32)> {
    Ok((ctx.argc as u32, ctx.arg_buffer.len() as u32))
}

// modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(no_effect!(old(trace), trace))]
pub fn wasi_environ_sizes_get(ctx: &VmCtx) -> RuntimeResult<(u32, u32)> {
    Ok((ctx.envc as u32, ctx.env_buffer.len() as u32))
}

#[with_ghost_var(trace: &mut Trace)]
#[external_methods(reserve_exact)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
pub fn wasi_sock_recv(
    ctx: &mut VmCtx,
    v_fd: u32,
    ri_data: u32,
    ri_data_count: u32,
    ri_flags: u32,
) -> RuntimeResult<(u32, u32)> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    let mut num: u32 = 0;
    let mut i = 0;
    while i < ri_data_count {
        body_invariant!(trace_safe(trace, ctx.memlen) && ctx_safe(ctx));

        let start = (ri_data + i * 8) as usize;
        if !ctx.fits_in_lin_mem_usize(start, 8) {
            return Err(Eoverflow);
        }
        let ptr = ctx.read_u32(start);
        let len = ctx.read_u32(start + 4);
        if !ctx.fits_in_lin_mem(ptr, len) {
            return Err(Efault);
        }

        let flags = 0;
        // TODO: handle flags
        let result = trace_recv(ctx, fd, ptr, len as usize, flags)?;
        let result = result as u32;

        num += result;
        i += 1;
    }
    // TODO: handle ro_flags
    Ok((num, 0))
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
pub fn wasi_sock_send(
    ctx: &mut VmCtx,
    v_fd: u32,
    si_data: u32,
    si_data_count: u32,
    si_flags: u32,
) -> RuntimeResult<u32> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    let mut num: u32 = 0;
    let mut i = 0;
    while i < si_data_count {
        body_invariant!(trace_safe(trace, ctx.memlen) && ctx_safe(ctx));

        let start = (si_data + i * 8) as usize;
        if !ctx.fits_in_lin_mem_usize(start, 8) {
            return Err(Eoverflow);
        }
        let ptr = ctx.read_u32(start);
        let len = ctx.read_u32(start + 4);
        if !ctx.fits_in_lin_mem(ptr, len) {
            return Err(Efault);
        }
        let flags = 0;
        // TODO: handle flags
        let result = trace_send(ctx, fd, ptr, len as usize, flags)?;
        num += result as u32;
        i += 1;
    }
    Ok(num)
}

// ensures: valid(v_fd) => trace = old(shutdown :: trace)
#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
// If sandbox does not own fd, no effects take place
//#[ensures(v_fd >= MAX_SBOX_FDS || !ctx.fdmap.m[v_fd].is_err() ==> no_effect!(old(trace), trace))]
// if args are valid, we do invoke an effect
//#[ensures( (v_fd < MAX_SBOX_FDS && ctx.fdmap.contains(v_fd)) ==> one_effect!(trace, old(trace), Effect::Shutdown) )] // we added 1 effect (add-only)
// #[ensures( (v_fd < MAX_SBOX_FDS && ctx.fdmap.contains(v_fd)) ==> matches!(trace.lookup(trace.len() - 1), Effect::Shutdown) )]
pub fn wasi_sock_shutdown(ctx: &VmCtx, v_fd: u32, v_how: u32) -> RuntimeResult<()> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let how = SdFlags::new(v_how);
    let fd = ctx.fdmap.m[v_fd as usize]?;
    let res = trace_shutdown(ctx, fd, how.into())?;
    Ok(())
}

// TODO: Do we need to check alignment on the pointers?
// TODO: clean this up, pretty gross
#[with_ghost_var(trace: &mut Trace)]
#[external_methods(subscription_clock_abstime)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
pub fn wasi_poll_oneoff(
    ctx: &mut VmCtx,
    in_ptr: u32,
    out_ptr: u32,
    nsubscriptions: u32,
) -> RuntimeResult<u32> {
    // copy events to runtime buffer
    if !ctx.fits_in_lin_mem(
        in_ptr,
        nsubscriptions * mem::size_of::<Subscription>() as u32,
    ) {
        return Err(Efault);
    }

    if !ctx.fits_in_lin_mem(out_ptr, nsubscriptions * mem::size_of::<Event>() as u32) {
        return Err(Efault);
    }

    //let mut current_byte = 0;
    let mut i = 0;
    while i < nsubscriptions {
        body_invariant!(trace_safe(trace, ctx.memlen) && ctx_safe(ctx));
        // TODO: refactor to use constants
        let sub_offset = i * 48;
        let event_offset = i * 32;
        if !ctx.fits_in_lin_mem_usize((in_ptr + sub_offset) as usize, 42) {
            return Err(Eoverflow);
        }
        if !ctx.fits_in_lin_mem_usize((out_ptr + event_offset) as usize, 12) {
            return Err(Eoverflow);
        }
        // read the subscription struct fields
        let userdata = ctx.read_u64((in_ptr + sub_offset) as usize);
        let tag = ctx.read_u64((in_ptr + sub_offset + 8) as usize);

        match tag {
            0 => {
                let clock_id = ctx.read_u32((in_ptr + sub_offset + 16) as usize);
                //let clock_id = clock_id.ok_or(Einval)?;
                let timeout_bytes = ctx.read_u64((in_ptr + sub_offset + 24) as usize);
                let timeout = Timestamp::new(timeout_bytes);
                // let _precision: Timestamp =
                //     Timestamp::new(ctx.read_u64((in_ptr + sub_offset + 32) as usize));
                let flags: SubClockFlags = ctx.read_u16((in_ptr + sub_offset + 40) as usize).into();
                // TODO: get clock time and use it to check if passed

                let now = wasi_clock_time_get(ctx, clock_id)?;
                let req: libc::timespec = if flags.subscription_clock_abstime() {
                    // is absolute, wait the diff between timeout and noew
                    // TODO: I assume we should check for underflow
                    (timeout - now).into()
                } else {
                    timeout.into()
                };

                let mut rem = libc::timespec {
                    tv_sec: 0,
                    tv_nsec: 0,
                };

                // TODO: handle multi-threaded case, in which nanosleep can be canceled by
                //       signals... we shouldn't need to handle it now though...
                let res = trace_nanosleep(ctx, &req, &mut rem)?;
                //RuntimeError::from_syscall_ret(res)?;

                // write the event output...
                ctx.write_u64((out_ptr + event_offset) as usize, userdata);
                let errno = 0;
                ctx.write_u16((out_ptr + event_offset + 8) as usize, errno); // TODO: errno
                ctx.write_u16(
                    (out_ptr + event_offset + 10) as usize,
                    EventType::Clock.into(),
                );
                // clock event ignores fd_readwrite field.
            }
            1 => {
                let v_fd = ctx.read_u32((in_ptr + sub_offset + 16) as usize);
                if v_fd >= MAX_SBOX_FDS {
                    return Err(Ebadf);
                }
                let fd = ctx.fdmap.m[v_fd as usize]?;
                let host_fd: usize = fd.into();

                let mut pollfd = libc::pollfd {
                    fd: host_fd as i32,
                    events: libc::POLLIN,
                    revents: 0,
                };
                let timeout = -1;

                let res = trace_poll(ctx, &mut pollfd, timeout)?;

                // write the event output...
                ctx.write_u64((out_ptr + event_offset) as usize, userdata);
                let errno = 0;
                ctx.write_u16((out_ptr + event_offset + 8) as usize, errno); // TODO: errno
                ctx.write_u16(
                    (out_ptr + event_offset + 10) as usize,
                    EventType::FdRead.into(),
                );
                // TODO: fd_readwrite member...need number of bytes available....
            }
            2 => {
                let v_fd = ctx.read_u32((in_ptr + sub_offset + 16) as usize);
                if v_fd >= MAX_SBOX_FDS {
                    return Err(Ebadf);
                }
                let fd = ctx.fdmap.m[v_fd as usize]?;
                let host_fd: usize = fd.into();

                let mut pollfd = libc::pollfd {
                    fd: host_fd as i32,
                    events: libc::POLLIN,
                    revents: 0,
                };
                let timeout = -1;

                let res = trace_poll(ctx, &mut pollfd, timeout)?;

                // write the event output...
                ctx.write_u64((out_ptr + event_offset) as usize, userdata);
                let errno = 0;
                ctx.write_u16((out_ptr + event_offset + 8) as usize, errno); // TODO: errno
                ctx.write_u16(
                    (out_ptr + event_offset + 10) as usize,
                    EventType::FdWrite.into(),
                );
                // TODO: fd_readwrite member...need number of bytes available....
            }
            _ => {
                return Err(Einval);
            }
        }

        i += 1;
    }

    Ok(0)
}

#[with_ghost_var(trace: &mut Trace)]
#[external_methods(extend_from_slice, reserve_exact)]
#[external_methods(to_le_bytes, to_wasi)]
#[external_calls(from_le_bytes, from, first_null, push_dirent_name)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
// TODO: currently ignoring cookie
// TODO: need to map posix filetypes to wasi filetypes
// TODO: I'm not confident this works for multiple consecutive readdir calls to the same dir
// Correct behavior: truncate final entry
pub fn wasi_fd_readdir(
    ctx: &mut VmCtx,
    v_fd: SboxFd,
    buf: SboxFd,
    buf_len: usize,
    cookie: u64,
) -> RuntimeResult<u32> {
    // TODO: use the cookie properly
    if cookie != 0 {
        return Err(Einval);
    }

    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }
    let fd = ctx.fdmap.m[v_fd as usize]?;

    let mut host_buf: Vec<u8> = Vec::new();
    host_buf.reserve_exact(buf_len as usize);

    let res = trace_getdents64(ctx, fd, &mut host_buf, buf_len)?;

    let mut in_idx = 0;
    let mut out_idx = 0;

    let mut out_buf: Vec<u8> = Vec::new();

    while in_idx < host_buf.len() && out_idx < host_buf.len() {
        body_invariant!(trace_safe(trace, ctx.memlen) && ctx_safe(ctx));
        // Inode number
        let d_ino = u64::from_le_bytes([
            host_buf[in_idx + 0],
            host_buf[in_idx + 1],
            host_buf[in_idx + 2],
            host_buf[in_idx + 3],
            host_buf[in_idx + 4],
            host_buf[in_idx + 5],
            host_buf[in_idx + 6],
            host_buf[in_idx + 7],
        ]);

        // Offset to next linux_dirent
        let d_offset = u64::from_le_bytes([
            host_buf[in_idx + 8],
            host_buf[in_idx + 9],
            host_buf[in_idx + 10],
            host_buf[in_idx + 11],
            host_buf[in_idx + 12],
            host_buf[in_idx + 13],
            host_buf[in_idx + 14],
            host_buf[in_idx + 15],
        ]);

        // Length of this linux_dirent
        let d_reclen = u16::from_le_bytes([host_buf[in_idx + 16], host_buf[in_idx + 17]]);
        // File type
        let d_type = u8::from_le_bytes([host_buf[in_idx + 18]]);

        // If we would overflow - don't :)
        if d_reclen < 19 || (in_idx + d_reclen as usize) >= host_buf.len() {
            break;
        }

        let out_namlen = first_null(&host_buf, in_idx, d_reclen as usize);
        // let out_namlen = 3;
        let out_next = in_idx + 24 + out_namlen as usize;

        // If we would overflow - don't :)
        if out_next > buf_len {
            break;
        }

        // Copy in next offset verbatim
        let out_next_bytes: [u8; 8] = out_next.to_le_bytes();
        out_buf.extend_from_slice(&out_next_bytes);

        // Copy in Inode verbatim
        let d_ino_bytes: [u8; 8] = d_ino.to_le_bytes();
        out_buf.extend_from_slice(&d_ino_bytes);

        // Copy namlen
        let out_namlen_bytes: [u8; 4] = (out_namlen as u32).to_le_bytes();
        out_buf.extend_from_slice(&out_namlen_bytes);

        // Copy type
        let d_type = Filetype::from(d_type as libc::mode_t);
        let out_type_bytes: [u8; 4] = (d_type.to_wasi() as u32).to_le_bytes();
        out_buf.extend_from_slice(&out_type_bytes);

        // Copy name
        push_dirent_name(&mut out_buf, &host_buf, in_idx, out_namlen as usize);

        in_idx += d_reclen as usize;
        out_idx += (24 + out_namlen) as usize
    }

    let copy_ok = ctx
        .copy_buf_to_sandbox(buf, &out_buf, out_buf.len() as u32)
        .ok_or(Efault)?;

    Ok(out_buf.len() as u32)
}

#[with_ghost_var(trace: &mut Trace)]
#[external_calls(sock_domain_to_posix, sock_type_to_posix)]
#[external_methods(create_sock)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
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
    let res = trace_socket(ctx, domain, ty, protocol)?;

    ctx.fdmap.create_sock(res.into(), wasi_proto)
    // ctx.fdmap.create(res.into())
}

#[with_ghost_var(trace: &mut Trace)]
#[external_calls(sock_domain_to_posix, from)]
#[external_methods(addr_in_netlist)]
#[requires(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx.memlen) && ctx_safe(ctx))]
pub fn wasi_sock_connect(
    ctx: &mut VmCtx,
    sockfd: u32,
    addr: u32,
    addrlen: u32,
) -> RuntimeResult<()> {
    if sockfd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }
    let fd = ctx.fdmap.m[sockfd as usize]?;

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

    let protocol = ctx.fdmap.sockinfo[usize::from(fd)]?;
    if matches!(protocol, WasiProto::Unknown) {
        return Err(Enotcapable);
    }
    //fd_proto(fd, protocol);// TODO: do this better?
    if !ctx.addr_in_netlist(sin_addr_in, sin_port as u32) {
        return Err(Enotcapable);
    }

    let res = trace_connect(ctx, fd, &saddr, addrlen)?;
    return Ok(());
}
