use crate::os::*;
use crate::runtime::*;
use crate::types::*;
#[cfg(feature = "verify")]
use crate::verifier::external_specs::result::*;
#[cfg(feature = "verify")]
use crate::verifier::*;
use crate::{effect, no_effect, one_effect};
use extra_args::{external_call, external_method, with_ghost_var};
use prusti_contracts::*;
use std::convert::{TryFrom, TryInto};
use std::mem;
use RuntimeError::*;

// Note: Prusti can't really handle iterators, so we need to use while loops

// Modifies: fdmap
// TODO: fdmap trace fix
#[with_ghost_var(trace: &mut Trace)]
#[external_call(Err)]
#[external_method(resolve_path)]
#[external_method(create)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
pub fn wasi_path_open(ctx: &mut VmCtx, pathname: u32, flags: i32) -> RuntimeResult<u32> {
    if !ctx.fits_in_lin_mem(pathname, PATH_MAX) {
        return Err(Ebadf);
    }

    let host_buffer = ctx.copy_buf_from_sandbox(pathname, PATH_MAX);
    let host_pathname = ctx.resolve_path(host_buffer)?;
    let fd = trace_open(ctx, host_pathname, flags);
    ctx.fdmap.create(fd.into())
}

// // modifies: fdmap
#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[external_call(Err)]
#[external_method(delete)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
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
    let result = trace_close(ctx, fd);
    Ok(result as u32)
}

// // modifies: mem
// #[with_ghost_var(trace: &mut Trace)]
// #[external_call(Ok)]
// #[external_call(Err)]
// #[requires(trace_safe(ctx, trace))]
// #[ensures(trace_safe(ctx, trace))]
// // TODO: fold-unfold error
// pub fn wasi_fd_read(
//     ctx: &mut VmCtx,
//     v_fd: u32,
//     iovs: u32,
//     iovcnt: u32,
// ) -> RuntimeResult<u32> {
//     if v_fd >= MAX_SBOX_FDS {
//         return Err(Ebadf);
//     }

//     let fd = ctx.fdmap.m[v_fd as usize]?;
//     let mut num: u32 = 0;
//     let mut i = 0;
//     while i < iovcnt {
//         body_invariant!(trace_safe(ctx, trace));

//         let start = (iovs + i * 8) as usize;
//         let ptr = ctx.read_u32(start);
//         let len = ctx.read_u32(start + 4);
//         if !ctx.fits_in_lin_mem(ptr, len) {
//             return Err(Efault);
//         }
//         let slice = ctx.slice_mem_mut(ptr, len);
//         let result = trace_read(ctx, fd, slice, len as usize);
//         RuntimeError::from_syscall_ret(result as usize)?;
//         let result = result as u32;
//         num += result;
//         i += 1;
//     }
//     Ok(num)
// }

// // modifies: none
// #[with_ghost_var(trace: &mut Trace)]
// #[external_call(Ok)]
// #[external_call(Err)]
// #[requires(trace_safe(ctx, trace))]
// #[ensures(trace_safe(ctx, trace))]
// // TODO: fold-unfold error
// pub fn wasi_fd_write(
//    ctx: &mut VmCtx,
//    v_fd: u32,
//    iovs: u32,
//    iovcnt: u32,
// ) -> RuntimeResult<u32> {
//    if v_fd >= MAX_SBOX_FDS {
//        return Err(Ebadf);
//    }

//    let fd = ctx.fdmap.m[v_fd as usize]?;
//    let mut num: u32 = 0;
//    let mut i = 0;
//    while i < iovcnt {
//        body_invariant!(trace_safe(ctx, trace));

//        let start = (iovs + i * 8) as usize;
//        let ptr = ctx.read_u32(start);
//        let len = ctx.read_u32(start + 4);
//        if !ctx.fits_in_lin_mem(ptr, len) {
//            return Err(Efault);
//        }
//        //let slice = ctx.slice_mem(ptr, len);
//        let start = ptr as usize;
//        let end = (ptr + len) as usize;
//        let slice = &ctx.mem[start..end];
//        let result = trace_write(ctx, fd, slice, len as usize);
//        RuntimeError::from_syscall_ret(result)?;
//        num += result as u32;
//        i += 1;
//    }
//    Ok(num)
// }

// // modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[external_method(ok_or)]
#[external_call(from_u32)]
#[external_call(Ok)]
#[external_call(Err)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
// #[ensures(v_fd < MAX_SBOX_FDS && ctx.fdmap.contains(v_fd) ==> one_effect!(old(trace), trace, Effect::FdAccess))]
// #[ensures(v_fd >= MAX_SBOX_FDS ==> no_effect!(old(trace), trace))]
pub fn wasi_fd_seek(ctx: &VmCtx, v_fd: u32, v_filedelta: i64, v_whence: u32) -> RuntimeResult<u32> {
    let whence = Whence::from_u32(v_whence).ok_or(Einval)?;

    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    let ret = trace_seek(ctx, fd, v_filedelta, whence.into());
    RuntimeError::from_syscall_ret(ret)?;
    Ok(ret as u32)
}

// modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
pub fn wasi_fd_tell(ctx: &VmCtx, v_fd: u32) -> RuntimeResult<u32> {
    wasi_fd_seek(ctx, v_fd, 0, 1) // Whence::Cur
}

// // modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[external_call(try_from)]
#[external_call(Ok)]
#[external_call(Err)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
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
    let ret = trace_advise(ctx, fd, offset as i64, len as i64, advice.into());
    RuntimeError::from_syscall_ret(ret)?;
    Ok(ret as u32)
}

// // modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[external_call(Err)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
pub fn wasi_fd_allocate(ctx: &VmCtx, v_fd: u32, offset: u64, len: u64) -> RuntimeResult<u32> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    // these casts could cause offset and len to become negative
    // I don't think this will be an issue as os_advise will throw an EINVAL error
    let ret = trace_allocate(ctx, fd, offset as i64, len as i64);
    RuntimeError::from_syscall_ret(ret)?;
    Ok(ret as u32)
}

// // modifies: none
// // TODO: should not return u32 at all?
#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[external_call(Err)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
pub fn wasi_fd_sync(ctx: &VmCtx, v_fd: u32) -> RuntimeResult<u32> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    let ret = trace_sync(ctx, fd);
    RuntimeError::from_syscall_ret(ret)?;
    Ok(ret as u32)
}

// // modifies: None
#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[external_call(Err)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
pub fn wasi_fd_datasync(ctx: &VmCtx, v_fd: u32) -> RuntimeResult<u32> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    let ret = trace_datasync(ctx, fd);
    RuntimeError::from_syscall_ret(ret)?;
    Ok(ret as u32)
}

// // //modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[external_call(Err)]
#[external_call(zeroed)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
pub fn wasi_fd_fdstat_get(ctx: &VmCtx, v_fd: u32) -> RuntimeResult<FdStat> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;

    // Unsafe necessary as libc::stat is opaque. It is safe but we can replace it by implementing
    // the struct ourselves if we want to avoid as much unsafe as possible.
    // Safety: Safe as libc::stat is valid with an all-zero byte-pattern (i.e. it is not a
    // reference)
    let mut stat: libc::stat = unsafe { std::mem::zeroed() };
    // TODO: double check, should this be fstat or fstat64?
    let filetype = trace_fstat(ctx, fd, &mut stat);
    RuntimeError::from_syscall_ret(filetype)?;

    let mode_flags = trace_fgetfl(ctx, fd);
    RuntimeError::from_syscall_ret(mode_flags)?;

    // TODO: put rights in once those are implemented
    let result = FdStat {
        fs_filetype: (filetype as libc::mode_t).into(),
        fs_flags: (mode_flags as libc::c_int).into(),
        fs_rights_base: 0,
        fs_rights_inheriting: 0,
    };
    Ok(result)
}

//TODO: need wasm layout for FdFlags to read from ptr
#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[external_call(Err)]
#[external_call(from)]
#[external_method(to_posix)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
pub fn wasi_fd_fdstat_set_flags(ctx: &mut VmCtx, v_fd: u32, v_flags: u32) -> RuntimeResult<()> {
    let flags = FdFlags::from(v_flags as i32);

    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    let posix_flags = flags.to_posix();

    let ret = trace_fsetfl(ctx, fd, posix_flags);
    RuntimeError::from_syscall_ret(ret)?;
    Ok(())
}

// // modifies: None
#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[external_call(Err)]
#[external_call(zeroed)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
pub fn wasi_fd_filestat_get(ctx: &VmCtx, v_fd: u32) -> RuntimeResult<FileStat> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    // Unsafe necessary as libc::stat is opaque. It is safe but we can replace it by implementing
    // the struct ourselves if we want to avoid as much unsafe as possible.
    // Safety: Safe as libc::stat is valid with an all-zero byte-pattern (i.e. it is not a
    // reference)
    let mut stat: libc::stat = unsafe { std::mem::zeroed() };
    let filetype = trace_fstat(ctx, fd, &mut stat);
    RuntimeError::from_syscall_ret(filetype)?;
    Ok(stat.into())
}

// modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[external_call(Err)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
pub fn wasi_fd_filestat_set_size(ctx: &VmCtx, v_fd: u32, size: u64) -> RuntimeResult<()> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    let ret = trace_ftruncate(ctx, fd, size as i64);
    RuntimeError::from_syscall_ret(ret)?;
    Ok(())
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[external_call(Err)]
#[external_call(new)]
#[external_method(atim_now)]
#[external_method(atim)]
#[external_method(mtim)]
#[external_method(mtim_now)]
#[external_method(reserve_exact)]
#[external_method(nsec)]
#[external_method(push)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
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

    if fst_flags.atim() && fst_flags.atim_now() || fst_flags.mtim() && fst_flags.mtim_now() {
        return Err(Einval);
    }

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

    let res = trace_futimens(ctx, fd, &specs);
    RuntimeError::from_syscall_ret(res)?;

    Ok(())
}

// TODO: refactor read and pread into common impl
// modifies: mem
// #[with_ghost_var(trace: &mut Trace)]
// #[external_call(Ok)]
// #[external_call(Err)]
// #[external_call(new)]
// #[external_method(ok_or)]
// #[external_method(reserve_exact)]
// #[external_method(push)]
// #[external_method(resolve_path)]
// #[requires(trace_safe(ctx, trace))]
// #[ensures(trace_safe(ctx, trace))]
// pub fn wasi_fd_pread(ctx: &mut VmCtx, v_fd: u32, iovs: u32, iovcnt: u32) -> RuntimeResult<u32> {
//    if v_fd >= MAX_SBOX_FDS {
//        return Err(Ebadf);
//    }

//    let fd = ctx.fdmap.m[v_fd as usize]?;
//    let mut num: u32 = 0;
//    let mut i = 0;
//    while i < iovcnt {
//        let start = (iovs + i * 8) as usize;
//        let ptr = ctx.read_u32(start);
//        let len = ctx.read_u32(start + 4);
//        if !ctx.fits_in_lin_mem(ptr, len) {
//            return Err(Efault);
//        }
//        let mut buf: Vec<u8> = Vec::new();
//        buf.reserve_exact(len as usize);
//        let result = trace_read(ctx, fd, &mut buf, len as usize);
//        RuntimeError::from_syscall_ret(result)?;
//        let result = result as u32;
//        let copy_ok = ctx
//            .copy_buf_to_sandbox(ptr, &buf, result as u32)
//            .ok_or(Efault)?;
//        num += result;
//        i += 1;
//    }
//    Ok(num)
// }

// modifies: ????
#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[external_call(Err)]
#[external_call(new)]
#[external_method(ok_or)]
#[external_method(push)]
#[external_method(len)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
pub fn wasi_prestat_dirname(
    ctx: &mut VmCtx,
    v_fd: u32,
    path: u32,
    path_len: u32,
) -> RuntimeResult<()> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let mut dirname: Vec<u8> = Vec::new();
    dirname.push(b'/');
    let dirname_len = dirname.len() as u32;
    if !ctx.fits_in_lin_mem(path, dirname_len) {
        return Err(Efault);
    }

    let copy_ok = ctx
        .copy_buf_to_sandbox(path, &dirname, dirname_len)
        .ok_or(Efault)?;
    Ok(())
}

/// Currently we use the same implementation as wasm2c, which is to not do very mucb at all
/// TODO: real implementation for this, most likely following wasi-common's implementation
#[with_ghost_var(trace: &mut Trace)]
#[external_call(Err)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
pub fn wasi_fd_prestat_get(ctx: &mut VmCtx, v_fd: u32) -> RuntimeResult<()> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }
    return Err(Emfile);
}

// TODO: refactor write and pwrite into common impl
// modifies: none
// #[with_ghost_var(trace: &mut Trace)]
// #[external_call(Ok)]
// #[external_call(Err)]
// #[external_call(new)]
// #[external_method(ok_or)]
// #[external_method(push)]
// #[external_method(resolve_path)]
// #[requires(trace_safe(ctx, trace))]
// #[ensures(trace_safe(ctx, trace))]
// pub fn wasi_fd_pwrite(ctx: &VmCtx, v_fd: u32, iovs: u32, iovcnt: u32) -> RuntimeResult<u32> {
//    if v_fd >= MAX_SBOX_FDS {
//        return Err(Ebadf);
//    }

//    let fd = ctx.fdmap.m[v_fd as usize]?;
//    let mut num: u32 = 0;
//    let mut i = 0;
//    while i < iovcnt {
//        let start = (iovs + i * 8) as usize;
//        let ptr = ctx.read_u32(start);
//        let len = ctx.read_u32(start + 4);
//        if !ctx.fits_in_lin_mem(ptr, len) {
//            return Err(Efault);
//        }
//        let host_buffer = ctx.copy_buf_from_sandbox(ptr, len);
//        let result = trace_write(ctx, fd, &host_buffer, len as usize);
//        RuntimeError::from_syscall_ret(result)?;
//        num += result as u32;
//        i += 1;
//    }
//    Ok(num)
// }

// // //TODO: should create fd for directory
// // // modifies: adds hostfd for directory created
#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[external_call(Err)]
#[external_call(new)]
#[external_method(ok_or)]
#[external_method(push)]
#[external_method(resolve_path)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
pub fn wasi_path_create_directory(ctx: &mut VmCtx, v_fd: u32, pathname: u32) -> RuntimeResult<()> {
    if !ctx.fits_in_lin_mem(pathname, PATH_MAX) {
        return Err(Ebadf);
    }

    let host_buffer = ctx.copy_buf_from_sandbox(pathname, PATH_MAX);
    let host_pathname = ctx.resolve_path(host_buffer)?;
    let fd = ctx.fdmap.m[v_fd as usize]?;
    // TODO: wasi doesn't seem so specify what permissions should be?
    //       I will use rw------- cause it seems sane.
    let mode = libc::S_IRUSR + libc::S_IWUSR; // using add cause | isn't supported
    let res = trace_mkdirat(ctx, fd, host_pathname, mode);
    RuntimeError::from_syscall_ret(res)?;
    Ok(())
}

// // // TODO: handle lookup flags
// // // TODO: this needs to make sure that the pathname is relative. If pathname is abosolute it won't
// // //       respect the fd.
// // // modifies: None
#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[external_call(Err)]
#[external_method(ok_or)]
#[external_method(push)]
#[external_method(resolve_path)]
#[external_call(zeroed)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
pub fn wasi_path_filestat_get(
    ctx: &VmCtx,
    v_fd: u32,
    flags: LookupFlags,
    pathname: u32,
) -> RuntimeResult<FileStat> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    if !ctx.fits_in_lin_mem(pathname, PATH_MAX) {
        return Err(Ebadf);
    }

    let host_buffer = ctx.copy_buf_from_sandbox(pathname, PATH_MAX);
    let host_pathname = ctx.resolve_path(host_buffer)?;
    let fd = ctx.fdmap.m[v_fd as usize]?;
    // Unsafe necessary as libc::stat is opaque. It is safe but we can replace it by implementing
    // the struct ourselves if we want to avoid as much unsafe as possible.
    // Safety: Safe as libc::stat is valid with an all-zero byte-pattern (i.e. it is not a
    //         reference)
    let mut stat: libc::stat = unsafe { std::mem::zeroed() };
    let res = trace_fstatat(ctx, fd, host_pathname, &mut stat, 0);
    RuntimeError::from_syscall_ret(res)?;
    Ok(stat.into())
}

// // // modifies: None
#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[external_call(Err)]
#[external_call(new)]
#[external_method(atim_now)]
#[external_method(atim)]
#[external_method(mtim)]
#[external_method(mtim_now)]
#[external_method(reserve_exact)]
#[external_method(nsec)]
#[external_method(push)]
#[external_method(resolve_path)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
pub fn wasi_path_filestat_set_times(
    ctx: &VmCtx,
    v_fd: u32,
    flags: LookupFlags,
    pathname: u32,
    atim: Timestamp,
    mtim: Timestamp,
    fst_flags: FstFlags,
) -> RuntimeResult<()> {
    if fst_flags.atim() && fst_flags.atim_now() || fst_flags.mtim() && fst_flags.mtim_now() {
        return Err(Einval);
    }

    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    if !ctx.fits_in_lin_mem(pathname, PATH_MAX) {
        return Err(Ebadf);
    }

    let host_buffer = ctx.copy_buf_from_sandbox(pathname, PATH_MAX);
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

    // TODO: path flags
    let res = trace_utimensat(ctx, fd, host_pathname, &specs, 0);
    RuntimeError::from_syscall_ret(res)?;

    Ok(())
}

// // TODO: Pass through the path lengths
// // TODO: handle LookupFlags
// // TODO: same caveat as wasi_path_filestat_get in terms of relative and absolute path.
// // modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[external_call(Err)]
#[external_call(new)]
#[external_method(resolve_path)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
pub fn wasi_path_link(
    ctx: &VmCtx,
    v_old_fd: u32,
    old_flags: LookupFlags,
    old_pathname: u32,
    v_new_fd: u32,
    new_pathname: u32,
) -> RuntimeResult<()> {
    if v_old_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }
    if v_new_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }
    if !ctx.fits_in_lin_mem(old_pathname, PATH_MAX) {
        return Err(Ebadf);
    }
    if !ctx.fits_in_lin_mem(new_pathname, PATH_MAX) {
        return Err(Ebadf);
    }

    let old_host_buffer = ctx.copy_buf_from_sandbox(old_pathname, PATH_MAX);
    let old_host_pathname = ctx.resolve_path(old_host_buffer)?;
    let new_host_buffer = ctx.copy_buf_from_sandbox(new_pathname, PATH_MAX);
    let new_host_pathname = ctx.resolve_path(new_host_buffer)?;
    let old_fd = ctx.fdmap.m[v_old_fd as usize]?;
    let new_fd = ctx.fdmap.m[v_new_fd as usize]?;

    let res = trace_linkat(ctx, old_fd, old_host_pathname, new_fd, new_host_pathname, 0);
    RuntimeError::from_syscall_ret(res)?;
    Ok(())
}

// // modifies: mem
#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[external_call(Err)]
#[external_call(new)]
#[external_method(reserve_exact)]
#[external_method(push)]
#[external_method(resolve_path)]
#[external_method(ok_or)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
pub fn wasi_path_readlink(
    ctx: &mut VmCtx,
    v_fd: u32,
    pathname: u32,
    ptr: u32,
    len: u32,
) -> RuntimeResult<u32> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }
    if !ctx.fits_in_lin_mem(pathname, PATH_MAX) {
        return Err(Ebadf);
    }

    let host_buffer = ctx.copy_buf_from_sandbox(pathname, PATH_MAX);
    let host_pathname = ctx.resolve_path(host_buffer)?;
    let fd = ctx.fdmap.m[v_fd as usize]?;

    if !ctx.fits_in_lin_mem(ptr, len) {
        return Err(Efault);
    }
    let mut buf: Vec<u8> = Vec::new();
    buf.reserve_exact(len as usize);

    let res = trace_readlinkat(ctx, fd, host_pathname, &mut buf, len as usize);
    RuntimeError::from_syscall_ret(res)?;
    let res = res as u32;
    let copy_ok = ctx.copy_buf_to_sandbox(ptr, &buf, res).ok_or(Efault)?;
    Ok(res)
}

//TODO: should remove fd from map?
//modifies: removes directory from fdmap
#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[external_call(Err)]
#[external_method(resolve_path)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
pub fn wasi_path_remove_directory(ctx: &mut VmCtx, v_fd: u32, pathname: u32) -> RuntimeResult<()> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }
    if !ctx.fits_in_lin_mem(pathname, PATH_MAX) {
        return Err(Ebadf);
    }

    let host_buffer = ctx.copy_buf_from_sandbox(pathname, PATH_MAX);
    let host_pathname = ctx.resolve_path(host_buffer)?;
    let fd = ctx.fdmap.m[v_fd as usize]?;

    let res = trace_unlinkat(ctx, fd, host_pathname, libc::AT_REMOVEDIR);
    let err = RuntimeError::from_syscall_ret(res);
    // posix spec allows unlinkat to return EEXIST for a non-empty directory
    // however, the wasi spec requires that ENOTEMPTY is returned
    // see: https://man7.org/linux/man-pages/man2/rmdir.2.html
    if let Err(Eexist) = err {
        return Err(RuntimeError::Enotempty);
    }
    err
}

// // modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[external_call(Err)]
#[external_method(resolve_path)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
pub fn wasi_path_rename(
    ctx: &VmCtx,
    v_old_fd: u32,
    old_pathname: u32,
    v_new_fd: u32,
    new_pathname: u32,
) -> RuntimeResult<()> {
    if v_old_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }
    if v_new_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }
    if !ctx.fits_in_lin_mem(old_pathname, PATH_MAX) {
        return Err(Ebadf);
    }
    if !ctx.fits_in_lin_mem(new_pathname, PATH_MAX) {
        return Err(Ebadf);
    }

    let old_host_buffer = ctx.copy_buf_from_sandbox(old_pathname, PATH_MAX);
    let old_host_pathname = ctx.resolve_path(old_host_buffer)?;
    let new_host_buffer = ctx.copy_buf_from_sandbox(new_pathname, PATH_MAX);
    let new_host_pathname = ctx.resolve_path(new_host_buffer)?;
    let old_fd = ctx.fdmap.m[v_old_fd as usize]?;
    let new_fd = ctx.fdmap.m[v_new_fd as usize]?;

    let res = trace_renameat(ctx, old_fd, old_host_pathname, new_fd, new_host_pathname);
    RuntimeError::from_syscall_ret(res)?;
    Ok(())
}

//modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[external_call(Err)]
#[external_method(resolve_path)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
pub fn wasi_path_symlink(
    ctx: &VmCtx,
    old_pathname: u32,
    v_fd: u32,
    new_pathname: u32,
) -> RuntimeResult<()> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }
    if !ctx.fits_in_lin_mem(old_pathname, PATH_MAX) {
        return Err(Ebadf);
    }
    if !ctx.fits_in_lin_mem(new_pathname, PATH_MAX) {
        return Err(Ebadf);
    }

    let old_host_buffer = ctx.copy_buf_from_sandbox(old_pathname, PATH_MAX);
    let old_host_pathname = ctx.resolve_path(old_host_buffer)?;
    let new_host_buffer = ctx.copy_buf_from_sandbox(new_pathname, PATH_MAX);
    let new_host_pathname = ctx.resolve_path(new_host_buffer)?;
    let fd = ctx.fdmap.m[v_fd as usize]?;

    let res = trace_symlinkat(ctx, old_host_pathname, fd, new_host_pathname);
    RuntimeError::from_syscall_ret(res)?;
    Ok(())
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[external_call(Err)]
#[external_method(resolve_path)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
pub fn wasi_path_unlink_file(ctx: &mut VmCtx, v_fd: u32, pathname: u32) -> RuntimeResult<()> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }
    if !ctx.fits_in_lin_mem(pathname, PATH_MAX) {
        return Err(Ebadf);
    }

    let host_buffer = ctx.copy_buf_from_sandbox(pathname, PATH_MAX);
    let host_pathname = ctx.resolve_path(host_buffer)?;
    let fd = ctx.fdmap.m[v_fd as usize]?;

    let res = trace_unlinkat(ctx, fd, host_pathname, 0);
    RuntimeError::from_syscall_ret(res)?;
    Ok(())
}

// modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[external_call(from_u32)]
#[external_method(ok_or)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
pub fn wasi_clock_res_get(ctx: &VmCtx, clock_id: u32) -> RuntimeResult<Timestamp> {
    let id = ClockId::from_u32(clock_id).ok_or(Einval)?;

    let mut spec = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };

    let ret = trace_clock_get_res(ctx, id.into(), &mut spec);
    RuntimeError::from_syscall_ret(ret)?;
    Ok(spec.into())
}

// modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[external_call(from_u32)]
#[external_method(ok_or)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
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

    let ret = trace_clock_get_time(ctx, id.into(), &mut spec);
    RuntimeError::from_syscall_ret(ret)?;
    Ok(spec.into())
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(no_effect!(old(trace), trace))]
pub fn wasi_proc_exit(ctx: &VmCtx, rval: u32) -> RuntimeResult<()> {
    Ok(())
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(no_effect!(old(trace), trace))]
pub fn wasi_proc_raise(ctx: &VmCtx, signal: u32) -> RuntimeResult<()> {
    Ok(())
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(no_effect!(old(trace), trace))]
pub fn wasi_sched_yield(ctx: &VmCtx) -> RuntimeResult<()> {
    Ok(())
}

// modifies: memory
#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[external_call(Err)]
#[external_call(new)]
#[external_method(reserve_exact)]
#[external_method(ok_or)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
pub fn wasi_random_get(ctx: &mut VmCtx, ptr: u32, len: u32) -> RuntimeResult<()> {
    if !ctx.fits_in_lin_mem(ptr, len) {
        return Err(Efault);
    }
    let mut buf: Vec<u8> = Vec::new();
    buf.reserve_exact(len as usize);
    let res = trace_getrandom(ctx, &mut buf, len as usize, 0);
    RuntimeError::from_syscall_ret(res)?;
    let copy_ok = ctx
        .copy_buf_to_sandbox(ptr, &buf, res as u32)
        .ok_or(Efault)?;
    Ok(())
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[external_call(Err)]
#[external_method(shift)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
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
#[external_call(Ok)]
#[external_method(ok_or)]
#[external_method(len)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
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
        body_invariant!(trace_safe(ctx, trace));
        if ctx.arg_buffer[idx] == b'\0' {
            ctx.write_u32((argv as usize) + cursor, start);
            cursor += 4;
            start = (idx as u32) + 1;
        }
        idx += 1;
    }
    Ok(())
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[external_method(ok_or)]
#[external_method(len)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
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
        body_invariant!(trace_safe(ctx, trace));
        if ctx.env_buffer[idx] == b'\0' {
            ctx.write_u32((env as usize) + cursor, start);
            cursor += 4;
            start = (idx as u32) + 1;
        }
        idx += 1;
    }
    Ok(())
}

// modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(no_effect!(old(trace), trace))]
pub fn wasi_args_sizes_get(ctx: &VmCtx) -> RuntimeResult<(u32, u32)> {
    Ok((ctx.argc as u32, ctx.arg_buffer.len() as u32))
}

// modifies: none
#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
#[ensures(no_effect!(old(trace), trace))]
pub fn wasi_environ_sizes_get(ctx: &VmCtx) -> RuntimeResult<(u32, u32)> {
    Ok((ctx.envc as u32, ctx.env_buffer.len() as u32))
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[external_call(Err)]
#[external_call(new)]
#[external_method(ok_or)]
#[external_method(reserve_exact)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
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
        body_invariant!(trace_safe(ctx, trace));
        let start = (ri_data + i * 8) as usize;
        let ptr = ctx.read_u32(start);
        let len = ctx.read_u32(start + 4);
        if !ctx.fits_in_lin_mem(ptr, len) {
            return Err(Efault);
        }
        let mut buf: Vec<u8> = Vec::new();
        buf.reserve_exact(len as usize);
        let flags = 0;
        // TODO: handle flags
        let result = trace_recv(ctx, fd, &mut buf, len as usize, flags);
        RuntimeError::from_syscall_ret(result)?;
        let result = result as u32;
        let copy_ok = ctx
            .copy_buf_to_sandbox(ptr, &buf, result as u32)
            .ok_or(Efault)?;
        num += result;
        i += 1;
    }
    // TODO: handle ro_flags
    Ok((num, 0))
}

#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[external_call(Err)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
pub fn wasi_sock_send(
    ctx: &VmCtx,
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
        body_invariant!(trace_safe(ctx, trace));
        let start = (si_data + i * 8) as usize;
        let ptr = ctx.read_u32(start);
        let len = ctx.read_u32(start + 4);
        if !ctx.fits_in_lin_mem(ptr, len) {
            return Err(Efault);
        }
        let host_buffer = ctx.copy_buf_from_sandbox(ptr, len);
        let flags = 0;
        // TODO: handle flags
        let result = trace_send(ctx, fd, &host_buffer, len as usize, flags);
        RuntimeError::from_syscall_ret(result)?;
        num += result as u32;
        i += 1;
    }
    Ok(num)
}

// ensures: valid(v_fd) => trace = old(shutdown :: trace)
#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[external_call(Err)]
#[external_call(new)]
#[external_method(into)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
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
    let res = trace_shutdown(ctx, fd, how.into());
    RuntimeError::from_syscall_ret(res)?;
    Ok(())
}

// TODO: Do we need to check alignment on the pointers?
// TODO: clean this up, pretty gross
#[with_ghost_var(trace: &mut Trace)]
#[external_call(Ok)]
#[external_call(Err)]
#[external_call(new)]
#[external_method(into)]
#[external_method(subscription_clock_abstime)]
#[requires(trace_safe(ctx, trace))]
#[ensures(trace_safe(ctx, trace))]
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
        body_invariant!(trace_safe(ctx, trace));
        // TODO: refactor to use constants
        let sub_offset = i * 48;
        let event_offset = i * 32;

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
                let res = trace_nanosleep(ctx, &req, &mut rem);
                RuntimeError::from_syscall_ret(res)?;

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

                let res = trace_poll(ctx, &mut pollfd, timeout);
                RuntimeError::from_syscall_ret(res)?;

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

                let res = trace_poll(ctx, &mut pollfd, timeout);
                RuntimeError::from_syscall_ret(res)?;

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
