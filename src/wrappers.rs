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

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_path_open(ctx: &mut VmCtx, pathname: u32, flags: i32) -> RuntimeResult<u32> {
    if !ctx.fits_in_lin_mem(pathname, PATH_MAX) {
        return Err(Ebadf);
    }

    let host_buffer = ctx.copy_buf_from_sandbox(pathname, PATH_MAX);
    let host_pathname = ctx.resolve_path(host_buffer)?;
    let fd = os_open(host_pathname, flags);
    ctx.fdmap.create(fd.into())
}

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_fd_close(ctx: &mut VmCtx, v_fd: u32) -> RuntimeResult<u32> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }
    let fd = ctx.fdmap.m[v_fd as usize]?;
    ctx.fdmap.delete(v_fd);
    Ok(os_close(fd) as u32)
}

//TODO: fix return type
#[requires(safe(ctx))]
#[ensures(safe(ctx))]
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

// TODO: fix return type
#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_fd_write(ctx: &mut VmCtx, v_fd: u32, iovs: u32, iovcnt: u32) -> RuntimeResult<u32> {
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
pub fn wasi_seek(
    ctx: &mut VmCtx,
    v_fd: u32,
    v_filedelta: i64,
    v_whence: Whence,
) -> RuntimeResult<u64> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    let ret = os_seek(fd, v_filedelta, v_whence.into());
    RuntimeError::from_syscall_ret(ret)?;
    Ok(ret as u64)
}

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_tell(ctx: &mut VmCtx, v_fd: u32) -> RuntimeResult<u64> {
    wasi_seek(ctx, v_fd, 0, Whence::Cur)
}

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_advise(
    ctx: &mut VmCtx,
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

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_allocate(ctx: &mut VmCtx, v_fd: u32, offset: u64, len: u64) -> RuntimeResult<u32> {
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

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_sync(ctx: &mut VmCtx, v_fd: u32) -> RuntimeResult<u32> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    let ret = os_sync(fd);
    RuntimeError::from_syscall_ret(ret)?;
    Ok(ret as u32)
}

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_datasync(ctx: &mut VmCtx, v_fd: u32) -> RuntimeResult<u32> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    let ret = os_datasync(fd);
    RuntimeError::from_syscall_ret(ret)?;
    Ok(ret as u32)
}

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_fdstat_get(ctx: &mut VmCtx, v_fd: u32) -> RuntimeResult<FdStat> {
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
// pub fn wasi_fdstat_set(ctx: &mut VmCtx, flags: FdFlags) -> u32

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_filestat_get(ctx: &mut VmCtx, v_fd: u32) -> RuntimeResult<FileStat> {
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

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_filestat_set_size(ctx: &mut VmCtx, v_fd: u32, size: u64) -> RuntimeResult<()> {
    if v_fd >= MAX_SBOX_FDS {
        return Err(Ebadf);
    }

    let fd = ctx.fdmap.m[v_fd as usize]?;
    let ret = os_ftruncate(fd, size as i64);
    RuntimeError::from_syscall_ret(ret)?;
    Ok(())
}

// TODO: how the heck does this work:
// https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md#-fd_filestat_set_timesfd-fd-atim-timestamp-mtim-timestamp-fst_flags-fstflags---result-errno

// TODO: refactor read and pread into common impl
#[requires(safe(ctx))]
#[ensures(safe(ctx))]
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
//TODO: fix return type
#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_fd_pwrite(ctx: &mut VmCtx, v_fd: u32, iovs: u32, iovcnt: u32) -> RuntimeResult<u32> {
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
pub fn wasi_clock_res_get(ctx: &mut VmCtx, id: ClockId) -> RuntimeResult<Timestamp> {
    let mut spec = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };

    let ret = os_clock_get_res(id.into(), &mut spec);
    RuntimeError::from_syscall_ret(ret)?;
    Ok(spec.into())
}

#[requires(safe(ctx))]
#[ensures(safe(ctx))]
pub fn wasi_clock_time_get(
    ctx: &mut VmCtx,
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::RuntimeResult;
    use std::time::Instant;

    // some basic sanity tests
    #[test]
    fn test_time_get() -> RuntimeResult<()> {
        let mut ctx = fresh_ctx(String::from("."));
        let ret = wasi_clock_time_get(&mut ctx, ClockId::Realtime, Timestamp::new(0))?;
        let reference = Instant::now();

        assert_ne!(ret, Timestamp::new(0));
        assert_eq!(ctx.errno, RuntimeError::Success);
        Ok(())
    }

    #[test]
    fn test_res_get() -> RuntimeResult<()> {
        let mut ctx = fresh_ctx(String::from("."));
        let ret = wasi_clock_res_get(&mut ctx, ClockId::Realtime)?;
        let reference = Instant::now();

        assert_ne!(ret, Timestamp::new(0));
        assert_eq!(ctx.errno, RuntimeError::Success);
        Ok(())
    }
}
