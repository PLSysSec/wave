#[cfg(feature = "verify")]
use crate::external_specs::result::*;
use crate::os::*;
use crate::runtime::*;
use crate::types::*;
use prusti_contracts::*;
use std::convert::TryInto;
use RuntimeError::*;

// Note: Prusti can't really handle iterators, so we need to use while loops

#[cfg(feature = "verify")]
predicate! {
    fn safe(ctx: &VmCtx) -> bool {
        true
    }
}

// Modifies: fdmap
pub fn wasi_path_open(ctx: &mut VmCtx, pathname: u32, flags: i32) -> RuntimeResult<u32> {
    if !ctx.fits_in_lin_mem(pathname, PATH_MAX) {
        return Err(Ebadf);
    }

    let host_buffer = ctx.copy_buf_from_sandbox(pathname, PATH_MAX);
    let host_pathname = ctx.resolve_path(host_buffer)?;
    let fd = os_open(host_pathname, flags);
    ctx.fdmap.create(fd.into())
}

//modifies: fdmap
pub fn wasi_fd_close(ctx: &mut VmCtx, v_fd: u32) -> RuntimeResult<u32> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }
    let fd = ctx.fdmap.m[v_fd as usize]?;
    ctx.fdmap.delete(v_fd);
    Ok(os_close(fd) as u32)
}

// modifies: mem
pub fn wasi_fd_read(ctx: &mut VmCtx, v_fd: u32, iovs: u32, iovcnt: u32) -> RuntimeResult<u32> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    let mut num: u32 = 0;
    let mut i = 0;
    while i < iovcnt {
        let start = (iovs + i * 8) as usize;
        let ptr = ctx.read_u32(start);
        let len = ctx.read_u32(start + 4);
        if !ctx.fits_in_lin_mem(ptr, len) {
            return Err(Efault);
        }
        let mut buf: Vec<u8> = Vec::new();
        buf.reserve_exact(len as usize);
        let result = os_read(fd, &mut buf, len as usize);
        // TODO: This pattern is a bit strange
        //       Probably better to have syscalls return an opaque SyscallRet type, which can
        //       be converted to RuntimeResult<usize> which we can use ? operator on.
        //       This also would put the logic to convert from a syscall return into OS specific
        //       code, which would be good for portability.
        RuntimeError::from_syscall_ret(result)?;
        let result = result as u32;
        let copy_ok = ctx
            .copy_buf_to_sandbox(ptr, &buf, result as u32)
            .ok_or(Efault)?;
        num += result;
        i += 1;
    }
    Ok(num)
}

// modifies: none
pub fn wasi_fd_write(ctx: &VmCtx, v_fd: u32, iovs: u32, iovcnt: u32) -> RuntimeResult<u32> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    let mut num: u32 = 0;
    let mut i = 0;
    while i < iovcnt {
        let start = (iovs + i * 8) as usize;
        let ptr = ctx.read_u32(start);
        let len = ctx.read_u32(start + 4);
        if !ctx.fits_in_lin_mem(ptr, len) {
            return Err(Efault);
        }
        let host_buffer = ctx.copy_buf_from_sandbox(ptr, len);
        let result = os_write(fd, &host_buffer, len as usize);
        RuntimeError::from_syscall_ret(result)?;
        num += result as u32;
        i += 1;
    }
    Ok(num)
}

// modifies: none
pub fn wasi_seek(ctx: &VmCtx, v_fd: u32, v_filedelta: i64, v_whence: Whence) -> RuntimeResult<u64> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    let ret = os_seek(fd, v_filedelta, v_whence.into());
    RuntimeError::from_syscall_ret(ret)?;
    Ok(ret as u64)
}

// modifies: none
pub fn wasi_tell(ctx: &VmCtx, v_fd: u32) -> RuntimeResult<u64> {
    wasi_seek(ctx, v_fd, 0, Whence::Cur)
}

// modifies: none
pub fn wasi_advise(
    ctx: &VmCtx,
    v_fd: u32,
    offset: u64,
    len: u64,
    advice: Advice,
) -> RuntimeResult<u32> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    // these casts could cause offset and len to become negative
    // I don't think this will be an issue as os_advise will throw an EINVAL error
    let ret = os_advise(fd, offset as i64, len as i64, advice.into());
    RuntimeError::from_syscall_ret(ret)?;
    Ok(ret as u32)
}

// modifies: none
pub fn wasi_allocate(ctx: &VmCtx, v_fd: u32, offset: u64, len: u64) -> RuntimeResult<u32> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    // these casts could cause offset and len to become negative
    // I don't think this will be an issue as os_advise will throw an EINVAL error
    let ret = os_allocate(fd, offset as i64, len as i64);
    RuntimeError::from_syscall_ret(ret)?;
    Ok(ret as u32)
}

// modifies: none
pub fn wasi_sync(ctx: &VmCtx, v_fd: u32) -> RuntimeResult<u32> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    let ret = os_sync(fd);
    RuntimeError::from_syscall_ret(ret)?;
    Ok(ret as u32)
}

// modifies: None
pub fn wasi_datasync(ctx: &VmCtx, v_fd: u32) -> RuntimeResult<u32> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    let ret = os_datasync(fd);
    RuntimeError::from_syscall_ret(ret)?;
    Ok(ret as u32)
}

//modifies: none
pub fn wasi_fdstat_get(ctx: &VmCtx, v_fd: u32) -> RuntimeResult<FdStat> {
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
    let filetype = os_fstat(fd, &mut stat);
    RuntimeError::from_syscall_ret(filetype)?;

    let mode_flags = os_fgetfl(fd);
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

// TODO: need wasm layout for FdFlags to read from ptr
pub fn wasi_fdstat_set(ctx: &mut VmCtx, v_fd: u32, flags: FdFlags) -> RuntimeResult<()> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    let posix_flags = flags.to_posix();

    let ret = os_fsetfl(fd, posix_flags);
    RuntimeError::from_syscall_ret(ret)?;
    Ok(())
}

// modifies: None
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
    let filetype = os_fstat(fd, &mut stat);
    RuntimeError::from_syscall_ret(filetype)?;
    Ok(stat.into())
}

// modifies: none
pub fn wasi_filestat_set_size(ctx: &VmCtx, v_fd: u32, size: u64) -> RuntimeResult<()> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    let ret = os_ftruncate(fd, size as i64);
    RuntimeError::from_syscall_ret(ret)?;
    Ok(())
}

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_fd_filestat_set_times(
    ctx: &mut VmCtx,
    v_fd: u32,
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

    let fd = ctx.fdmap.m[v_fd as usize]?;
    // TODO: should inval clock be handled in higher level, or have Unkown ClockId variant
    //       and handle here?
    // TODO: how to handle `precision` arg? Looks like some runtimes ignore it...
    let mut specs: [libc::timespec; 2] = [
        libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
    ];

    specs[0].tv_sec = 0;
    specs[0].tv_nsec = if fst_flags.atim() {
        atim.nsec() as i64
    } else if fst_flags.atim_now() {
        libc::UTIME_NOW
    } else {
        libc::UTIME_OMIT
    };

    specs[1].tv_sec = 0;
    specs[1].tv_nsec = if fst_flags.mtim() {
        mtim.nsec() as i64
    } else if fst_flags.mtim_now() {
        libc::UTIME_NOW
    } else {
        libc::UTIME_OMIT
    };

    let res = os_futimens(fd, &specs);
    RuntimeError::from_syscall_ret(res)?;

    Ok(())
}

// TODO: refactor read and pread into common impl
// modifies: mem
pub fn wasi_fd_pread(ctx: &mut VmCtx, v_fd: u32, iovs: u32, iovcnt: u32) -> RuntimeResult<u32> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    let mut num: u32 = 0;
    let mut i = 0;
    while i < iovcnt {
        let start = (iovs + i * 8) as usize;
        let ptr = ctx.read_u32(start);
        let len = ctx.read_u32(start + 4);
        if !ctx.fits_in_lin_mem(ptr, len) {
            return Err(Efault);
        }
        let mut buf: Vec<u8> = Vec::new();
        buf.reserve_exact(len as usize);
        let result = os_read(fd, &mut buf, len as usize);
        RuntimeError::from_syscall_ret(result)?;
        let result = result as u32;
        let copy_ok = ctx
            .copy_buf_to_sandbox(ptr, &buf, result as u32)
            .ok_or(Efault)?;
        num += result;
        i += 1;
    }
    Ok(num)
}

// TODO: refactor write and pwrite into common impl
// modifies: none
pub fn wasi_fd_pwrite(ctx: &VmCtx, v_fd: u32, iovs: u32, iovcnt: u32) -> RuntimeResult<u32> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    let mut num: u32 = 0;
    let mut i = 0;
    while i < iovcnt {
        let start = (iovs + i * 8) as usize;
        let ptr = ctx.read_u32(start);
        let len = ctx.read_u32(start + 4);
        if !ctx.fits_in_lin_mem(ptr, len) {
            return Err(Efault);
        }
        let host_buffer = ctx.copy_buf_from_sandbox(ptr, len);
        let result = os_write(fd, &host_buffer, len as usize);
        RuntimeError::from_syscall_ret(result)?;
        num += result as u32;
        i += 1;
    }
    Ok(num)
}

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_path_create_directory(ctx: &mut VmCtx, v_fd: u32, pathname: u32) -> RuntimeResult<()> {
    if !ctx.fits_in_lin_mem(pathname, PATH_MAX) {
        return Err(Ebadf);
    }

    let host_buffer = ctx.copy_buf_from_sandbox(pathname, PATH_MAX);
    let host_pathname = ctx.ensure_relative_path(host_buffer)?;
    let fd = ctx.fdmap.m[v_fd as usize]?;
    // TODO: wasi doesn't seem so specify what permissions should be?
    //       I will use rw------- cause it seems sane.
    let mode = libc::S_IRUSR + libc::S_IWUSR; // using add cause | isn't supported
    let res = os_mkdirat(fd, host_pathname, mode);
    RuntimeError::from_syscall_ret(res)?;
    Ok(())
}

// TODO: handle lookup flags
// TODO: this needs to make sure that the pathname is relative. If pathname is abosolute it won't
//       respect the fd.
#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_path_filestat_get(
    ctx: &mut VmCtx,
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
    let host_pathname = ctx.ensure_relative_path(host_buffer)?;
    let fd = ctx.fdmap.m[v_fd as usize]?;
    // Unsafe necessary as libc::stat is opaque. It is safe but we can replace it by implementing
    // the struct ourselves if we want to avoid as much unsafe as possible.
    // Safety: Safe as libc::stat is valid with an all-zero byte-pattern (i.e. it is not a
    //         reference)
    let mut stat: libc::stat = unsafe { std::mem::zeroed() };
    let res = os_fstatat(fd, host_pathname, &mut stat, 0);
    RuntimeError::from_syscall_ret(res)?;
    Ok(stat.into())
}

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_path_filestat_set_times(
    ctx: &mut VmCtx,
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

    let host_buffer = ctx.copy_buf_from_sandbox(pathname, PATH_MAX);
    let host_pathname = ctx.ensure_relative_path(host_buffer)?;
    let fd = ctx.fdmap.m[v_fd as usize]?;
    // TODO: should inval clock be handled in higher level, or have Unkown ClockId variant
    //       and handle here?
    let mut specs: [libc::timespec; 2] = [
        libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
        libc::timespec {
            tv_sec: 0,
            tv_nsec: 0,
        },
    ];

    specs[0].tv_sec = 0;
    specs[0].tv_nsec = if fst_flags.atim() {
        atim.nsec() as i64
    } else if fst_flags.atim_now() {
        libc::UTIME_NOW
    } else {
        libc::UTIME_OMIT
    };

    specs[1].tv_sec = 0;
    specs[1].tv_nsec = if fst_flags.mtim() {
        mtim.nsec() as i64
    } else if fst_flags.mtim_now() {
        libc::UTIME_NOW
    } else {
        libc::UTIME_OMIT
    };

    // TODO: path flags
    let res = os_utimensat(fd, host_pathname, &specs, 0);
    RuntimeError::from_syscall_ret(res)?;

    Ok(())
}

// TODO: handle LookupFlags
// TODO: same caveat as wasi_path_filestat_get in terms of relative and absolute path.
#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_path_link(
    ctx: &mut VmCtx,
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
    let old_host_pathname = ctx.ensure_relative_path(old_host_buffer)?;
    let new_host_buffer = ctx.copy_buf_from_sandbox(new_pathname, PATH_MAX);
    let new_host_pathname = ctx.ensure_relative_path(new_host_buffer)?;
    let old_fd = ctx.fdmap.m[v_old_fd as usize]?;
    let new_fd = ctx.fdmap.m[v_new_fd as usize]?;

    let res = os_linkat(old_fd, old_host_pathname, new_fd, new_host_pathname, 0);
    RuntimeError::from_syscall_ret(res)?;
    Ok(())
}

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
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
    let host_pathname = ctx.ensure_relative_path(host_buffer)?;
    let fd = ctx.fdmap.m[v_fd as usize]?;

    if !ctx.fits_in_lin_mem(ptr, len) {
        return Err(Efault);
    }
    let mut buf: Vec<u8> = Vec::new();
    buf.reserve_exact(len as usize);

    let res = os_readlinkat(fd, host_pathname, &mut buf, len as usize);
    RuntimeError::from_syscall_ret(res)?;
    let res = res as u32;
    let copy_ok = ctx.copy_buf_to_sandbox(ptr, &buf, res).ok_or(Efault)?;
    Ok(res)
}

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_path_remove_directory(ctx: &mut VmCtx, v_fd: u32, pathname: u32) -> RuntimeResult<()> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }
    if !ctx.fits_in_lin_mem(pathname, PATH_MAX) {
        return Err(Ebadf);
    }

    let host_buffer = ctx.copy_buf_from_sandbox(pathname, PATH_MAX);
    let host_pathname = ctx.ensure_relative_path(host_buffer)?;
    let fd = ctx.fdmap.m[v_fd as usize]?;

    let res = os_unlinkat(fd, host_pathname, libc::AT_REMOVEDIR);
    let err = RuntimeError::from_syscall_ret(res);
    // posix spec allows unlinkat to return EEXIST for a non-empty directory
    // however, the wasi spec requires that ENOTEMPTY is returned
    // see: https://man7.org/linux/man-pages/man2/rmdir.2.html
    if let Err(errno) = err {
        if errno == Eexist {
            return Err(RuntimeError::Enotempty);
        }
    }
    err
}

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_path_rename(
    ctx: &mut VmCtx,
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
    let old_host_pathname = ctx.ensure_relative_path(old_host_buffer)?;
    let new_host_buffer = ctx.copy_buf_from_sandbox(new_pathname, PATH_MAX);
    let new_host_pathname = ctx.ensure_relative_path(new_host_buffer)?;
    let old_fd = ctx.fdmap.m[v_old_fd as usize]?;
    let new_fd = ctx.fdmap.m[v_new_fd as usize]?;

    let res = os_renameat(old_fd, old_host_pathname, new_fd, new_host_pathname);
    RuntimeError::from_syscall_ret(res)?;
    Ok(())
}

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_path_symlink(
    ctx: &mut VmCtx,
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
    let new_host_pathname = ctx.ensure_relative_path(new_host_buffer)?;
    let fd = ctx.fdmap.m[v_fd as usize]?;

    let res = os_symlinkat(old_host_pathname, fd, new_host_pathname);
    RuntimeError::from_syscall_ret(res)?;
    Ok(())
}

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_path_unlink_file(ctx: &mut VmCtx, v_fd: u32, pathname: u32) -> RuntimeResult<()> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }
    if !ctx.fits_in_lin_mem(pathname, PATH_MAX) {
        return Err(Ebadf);
    }

    let host_buffer = ctx.copy_buf_from_sandbox(pathname, PATH_MAX);
    let host_pathname = ctx.ensure_relative_path(host_buffer)?;
    let fd = ctx.fdmap.m[v_fd as usize]?;

    let res = os_unlinkat(fd, host_pathname, 0);
    RuntimeError::from_syscall_ret(res)?;
    Ok(())
}

// modifies: none
pub fn wasi_clock_res_get(ctx: &VmCtx, id: ClockId) -> RuntimeResult<Timestamp> {
    let mut spec = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };

    let ret = os_clock_get_res(id.into(), &mut spec);
    RuntimeError::from_syscall_ret(ret)?;
    Ok(spec.into())
}

// modifies: none
pub fn wasi_clock_time_get(
    ctx: &VmCtx,
    id: ClockId,
    precision: Timestamp,
) -> RuntimeResult<Timestamp> {
    // TODO: should inval clock be handled in higher level, or have Unkown ClockId variant
    //       and handle here?
    // TODO: how to handle `precision` arg? Looks like some runtimes ignore it...
    let mut spec = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };

    let ret = os_clock_get_time(id.into(), &mut spec);
    RuntimeError::from_syscall_ret(ret)?;
    Ok(spec.into())
}

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_random_get(ctx: &mut VmCtx, ptr: u32, len: u32) -> RuntimeResult<()> {
    if !ctx.fits_in_lin_mem(ptr, len) {
        return Err(Efault);
    }
    let mut buf: Vec<u8> = Vec::new();
    buf.reserve_exact(len as usize);
    let res = os_getrandom(&mut buf, len as usize, 0);
    RuntimeError::from_syscall_ret(res)?;
    let copy_ok = ctx
        .copy_buf_to_sandbox(ptr, &buf, res as u32)
        .ok_or(Efault)?;
    Ok(())
}
