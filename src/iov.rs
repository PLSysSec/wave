use crate::{rvec::RVec, types::*, unwrap_result};
use RuntimeError::*;

pub fn parse_iovs(ctx: &VmCtx, iovs: u32, iovcnt: u32) -> RuntimeResult<RVec<WasmIoVec>> {
    let mut i = 0;
    let mut wasm_iovs = RVec::new();
    while i < iovcnt {
        let start = (iovs + i * 8) as usize;
        let v = ctx.read_u32_pair(start);
        unwrap_result!(v);
        // let ptr = v.fst;
        // let len = v.snd;
        let (ptr, len) = v;

        if !ctx.fits_in_lin_mem(ptr, len) {
            return Err(Efault);
        }

        wasm_iovs.push(WasmIoVec {
            iov_base: ptr,
            iov_len: len,
        });
        i += 1;
    }
    assert(wasm_iovs.len() >= 0);

    Ok(wasm_iovs)
}
