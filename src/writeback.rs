#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::types::*;
use extra_args::with_ghost_var;
use prusti_contracts::*;

// TODO: should these turn err or -err ?

#[with_ghost_var(trace: &mut Trace)]
pub fn wasm2c_marshal<T>(res: RuntimeResult<T>) -> u32 {
    match res {
        Ok(r) => 0,
        Err(err) => err.into(),
    }
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx_ref.memlen) && ctx_safe(ctx_ref))]
#[ensures(trace_safe(trace, ctx_ref.memlen) && ctx_safe(ctx_ref))]
pub fn wasm2c_marshal_and_writeback_u32(
    ctx_ref: &mut VmCtx,
    addr: usize,
    res: RuntimeResult<u32>,
) -> u32 {
    if !ctx_ref.fits_in_lin_mem_usize(addr, 4) {
        return RuntimeError::Eoverflow.into();
    }
    //log::debug!("wasm2c_marshal_and_writeback_u32: {:?}", result);
    match res {
        Ok(r) => {
            ctx_ref.write_u32(addr, r); // writeback result
            0
        }
        Err(err) => err.into(),
    }
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx_ref.memlen) && ctx_safe(ctx_ref))]
#[ensures(trace_safe(trace, ctx_ref.memlen) && ctx_safe(ctx_ref))]
pub fn wasm2c_marshal_and_writeback_prestat(
    ctx_ref: &mut VmCtx,
    addr: usize,
    res: RuntimeResult<u32>,
) -> u32 {
    if !ctx_ref.fits_in_lin_mem_usize(addr, 12) {
        return RuntimeError::Eoverflow.into();
    }
    //log::debug!("wasm2c_marshal_and_writeback_prestat: {:?}", result);
    match res {
        Ok(r) => {
            ctx_ref.write_u32(addr, 0);
            ctx_ref.write_u64(addr + 4, r as u64); // writeback result
            0
        }
        Err(err) => err.into(),
    }
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx_ref.memlen) && ctx_safe(ctx_ref))]
#[ensures(trace_safe(trace, ctx_ref.memlen) && ctx_safe(ctx_ref))]
pub fn wasm2c_marshal_and_writeback_u64(
    ctx_ref: &mut VmCtx,
    addr: usize,
    res: RuntimeResult<u64>,
) -> u32 {
    if !ctx_ref.fits_in_lin_mem_usize(addr, 8) {
        return RuntimeError::Eoverflow.into();
    }
    //log::debug!("wasm2c_marshal_and_writeback_u64: {:?}", result);
    match res {
        Ok(r) => {
            ctx_ref.write_u64(addr, r); // writeback result
            0
        }
        Err(err) => err.into(),
    }
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx_ref.memlen) && ctx_safe(ctx_ref))]
#[ensures(trace_safe(trace, ctx_ref.memlen) && ctx_safe(ctx_ref))]
pub fn wasm2c_marshal_and_writeback_timestamp(
    ctx_ref: &mut VmCtx,
    addr: usize,
    res: RuntimeResult<Timestamp>,
) -> u32 {
    if !ctx_ref.fits_in_lin_mem_usize(addr, 8) {
        return RuntimeError::Eoverflow.into();
    }
    //log::debug!("wasm2c_marshal_and_writeback_timestamp: {:?}", result);
    match res {
        Ok(r) => {
            ctx_ref.write_u64(addr, r.nsec()); // writeback result
            0
        }
        Err(err) => err.into(),
    }
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx_ref.memlen) && ctx_safe(ctx_ref))]
#[ensures(trace_safe(trace, ctx_ref.memlen) && ctx_safe(ctx_ref))]
pub fn wasm2c_marshal_and_writeback_fdstat(
    ctx_ref: &mut VmCtx,
    addr: usize,
    res: RuntimeResult<FdStat>,
) -> u32 {
    if !ctx_ref.fits_in_lin_mem_usize(addr, 24) {
        return RuntimeError::Eoverflow.into();
    }
    //log::debug!("wasm2c_marshal_and_writeback_fdstat: {:?}", result);
    match res {
        Ok(r) => {
            ctx_ref.write_u16(addr, r.fs_filetype.to_wasi() as u16);
            ctx_ref.write_u16(addr + 2, r.fs_flags.to_posix() as u16);
            ctx_ref.write_u64(addr + 8, r.fs_rights_base);
            ctx_ref.write_u64(addr + 16, r.fs_rights_inheriting);
            0
        }
        Err(err) => err.into(),
    }
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx_ref.memlen) && ctx_safe(ctx_ref))]
#[ensures(trace_safe(trace, ctx_ref.memlen) && ctx_safe(ctx_ref))]
pub fn wasm2c_marshal_and_writeback_filestat(
    ctx_ref: &mut VmCtx,
    addr: usize,
    res: RuntimeResult<FileStat>,
) -> u32 {
    if !ctx_ref.fits_in_lin_mem_usize(addr, 64) {
        return RuntimeError::Eoverflow.into();
    }
    //log::debug!("wasm2c_marshal_and_writeback_filestat: {:?}", result);
    match res {
        Ok(r) => {
            ctx_ref.write_u64(addr, r.dev);
            ctx_ref.write_u64(addr + 8, r.ino);
            ctx_ref.write_u64(addr + 16, r.filetype.to_wasi() as u64);
            ctx_ref.write_u64(addr + 24, r.nlink);
            ctx_ref.write_u64(addr + 32, r.size);
            ctx_ref.write_u64(addr + 40, r.atim.nsec());
            ctx_ref.write_u64(addr + 48, r.mtim.nsec());
            ctx_ref.write_u64(addr + 56, r.ctim.nsec());
            0
        }
        Err(err) => err.into(),
    }
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(trace_safe(trace, ctx_ref.memlen) && ctx_safe(ctx_ref))]
#[ensures(trace_safe(trace, ctx_ref.memlen) && ctx_safe(ctx_ref))]
pub fn wasm2c_marshal_and_writeback_u32_pair(
    ctx_ref: &mut VmCtx,
    addr0: usize,
    addr1: usize,
    res: RuntimeResult<(u32, u32)>,
) -> u32 {
    if !ctx_ref.fits_in_lin_mem_usize(addr0, 4) {
        return RuntimeError::Eoverflow.into();
    }
    if !ctx_ref.fits_in_lin_mem_usize(addr1, 4) {
        return RuntimeError::Eoverflow.into();
    }
    //log::debug!("wasm2c_marshal_and_writeback_u32_pair: {:?}", result);
    match res {
        Ok((v0, v1)) => {
            ctx_ref.write_u32(addr0, v0); // writeback envc
            ctx_ref.write_u32(addr1, v1); // writeback environ_buf
            0
        }
        Err(err) => err.into(),
    }
}
