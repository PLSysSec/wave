// #[cfg(feature = "verify")]
// use crate::tcb::verifier::*;
use crate::types::*;
// use prusti_contracts::*;
// use wave_macros::with_ghost_var;

// #[with_ghost_var(trace: &mut Trace)]
pub fn wasm2c_marshal<T>(res: RuntimeResult<T>) -> u32 {
    match res {
        Ok(r) => 0,
        Err(err) => err.into(),
    }
}

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
pub fn wasm2c_marshal_and_writeback_u32(
    ctx: &mut VmCtx,
    addr: usize,
    res: RuntimeResult<u32>,
) -> u32 {
    if !ctx.fits_in_lin_mem_usize(addr, 4) {
        return RuntimeError::Eoverflow.into();
    }
    //log::debug!("wasm2c_marshal_and_writeback_u32: {:?}", result);
    match res {
        Ok(r) => {
            ctx.write_u32(addr, r); // writeback result
            0
        }
        Err(err) => err.into(),
    }
}

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
pub fn wasm2c_marshal_and_writeback_prestat(
    ctx: &mut VmCtx,
    addr: usize,
    res: RuntimeResult<u32>,
) -> u32 {
    if !ctx.fits_in_lin_mem_usize(addr, 12) {
        return RuntimeError::Eoverflow.into();
    }
    //log::debug!("wasm2c_marshal_and_writeback_prestat: {:?}", result);
    match res {
        Ok(r) => {
            ctx.write_u32(addr, 0);
            ctx.write_u64(addr + 4, r as u64); // writeback result
            0
        }
        Err(err) => err.into(),
    }
}

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
pub fn wasm2c_marshal_and_writeback_u64(
    ctx: &mut VmCtx,
    addr: usize,
    res: RuntimeResult<u64>,
) -> u32 {
    if !ctx.fits_in_lin_mem_usize(addr, 8) {
        return RuntimeError::Eoverflow.into();
    }
    //log::debug!("wasm2c_marshal_and_writeback_u64: {:?}", result);
    match res {
        Ok(r) => {
            ctx.write_u64(addr, r); // writeback result
            0
        }
        Err(err) => err.into(),
    }
}

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
pub fn wasm2c_marshal_and_writeback_timestamp(
    ctx: &mut VmCtx,
    addr: usize,
    res: RuntimeResult<Timestamp>,
) -> u32 {
    if !ctx.fits_in_lin_mem_usize(addr, 8) {
        return RuntimeError::Eoverflow.into();
    }
    //log::debug!("wasm2c_marshal_and_writeback_timestamp: {:?}", result);
    match res {
        Ok(r) => {
            ctx.write_u64(addr, r.nsec()); // writeback result
            0
        }
        Err(err) => err.into(),
    }
}

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
pub fn wasm2c_marshal_and_writeback_fdstat(
    ctx: &mut VmCtx,
    addr: usize,
    res: RuntimeResult<FdStat>,
) -> u32 {
    if !ctx.fits_in_lin_mem_usize(addr, 24) {
        return RuntimeError::Eoverflow.into();
    }
    //log::debug!("wasm2c_marshal_and_writeback_fdstat: {:?}", result);
    match res {
        Ok(r) => {
            ctx.write_u16(addr, r.fs_filetype.to_wasi() as u16);
            ctx.write_u16(addr + 2, r.fs_flags.to_posix() as u16);
            ctx.write_u64(addr + 8, r.fs_rights_base);
            ctx.write_u64(addr + 16, r.fs_rights_inheriting);
            0
        }
        Err(err) => err.into(),
    }
}

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
pub fn wasm2c_marshal_and_writeback_filestat(
    ctx: &mut VmCtx,
    addr: usize,
    res: RuntimeResult<FileStat>,
) -> u32 {
    if !ctx.fits_in_lin_mem_usize(addr, 64) {
        return RuntimeError::Eoverflow.into();
    }
    //log::debug!("wasm2c_marshal_and_writeback_filestat: {:?}", result);
    match res {
        Ok(r) => {
            ctx.write_u64(addr, r.dev);
            ctx.write_u64(addr + 8, r.ino);
            ctx.write_u64(addr + 16, r.filetype.to_wasi() as u64);
            ctx.write_u64(addr + 24, r.nlink);
            ctx.write_u64(addr + 32, r.size);
            ctx.write_u64(addr + 40, r.atim.nsec());
            ctx.write_u64(addr + 48, r.mtim.nsec());
            ctx.write_u64(addr + 56, r.ctim.nsec());
            0
        }
        Err(err) => err.into(),
    }
}

// #[with_ghost_var(trace: &mut Trace)]
// #[requires(ctx_safe(ctx))]
// #[requires(trace_safe(trace, ctx))]
// #[ensures(ctx_safe(ctx))]
// #[ensures(trace_safe(trace, ctx))]
pub fn wasm2c_marshal_and_writeback_u32_pair(
    ctx: &mut VmCtx,
    addr0: usize,
    addr1: usize,
    res: RuntimeResult<(u32, u32)>,
) -> u32 {
    if !ctx.fits_in_lin_mem_usize(addr0, 4) {
        return RuntimeError::Eoverflow.into();
    }
    if !ctx.fits_in_lin_mem_usize(addr1, 4) {
        return RuntimeError::Eoverflow.into();
    }
    //log::debug!("wasm2c_marshal_and_writeback_u32_pair: {:?}", result);
    match res {
        Ok((v0, v1)) => {
            ctx.write_u32(addr0, v0); // writeback envc
            ctx.write_u32(addr1, v1); // writeback environ_buf
            0
        }
        Err(err) => err.into(),
    }
}
