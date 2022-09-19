#[cfg(feature = "verify")]
use crate::tcb::verifier::*;
use crate::types::*;
use crate::{effect, effects, unwrap_result};
use prusti_contracts::*;
use wave_macros::{external_methods, with_ghost_var};

use RuntimeError::*;

// #[macro_export]
// macro_rules! effects_from_iovs {
//     ($trace:expr, $buf:expr) => {
//         $trace.len() == old($trace.len() + $buf.len()) &&
//         forall(|i: usize| (i >= $trace.len()) ||
//         {
//             if i < old($trace.len())
//                 { $trace.lookup(i) == old($trace.lookup(i)) }
//             else
//             {
//                 let this = $buf.lookup(i - old($trace.len()));
//                 let ev = $trace.lookup(i);
//                 iov_eq(ev, &this)
//             }
//         }
//     )
//     }
// }

// #[cfg(feature = "verify")]
// predicate! {
//     pub fn effects_from_iovs(old_trace: &Trace, trace: &Trace, buf: &NativeIoVecs) -> bool {
//         trace.len() == old_trace.len() + buf.len() &&
//         forall(|i: usize| (i < trace.len()) ==>
//         {
//             if i < old(trace.len())
//                 { trace.lookup(i) == old_trace.lookup(i) }
//             else
//             {
//                 let this = buf.lookup(i - old_trace.len());
//                 let ev = trace.lookup(i);
//                 iov_eq(ev, &this)
//             }
//         }
//         )
//     }
// }

#[cfg(feature = "verify")]
predicate! {
    pub fn iov_eq_read(ev: Effect, iov: &NativeIoVec) -> bool {
        match ev {
            effect!(ReadMem,addr,count) =>
                addr == iov.iov_base &&
                count == iov.iov_len,
            _ => false,
        }
    }
}

#[cfg(feature = "verify")]
predicate! {
    pub fn iov_eq_write(ev: Effect, iov: &NativeIoVec) -> bool {
        match ev {
            effect!(WriteMem,addr,count) =>
                addr == iov.iov_base &&
                count == iov.iov_len,
            _ => false,
        }
    }
}

#[with_ghost_var(trace: &mut Trace)]
#[requires(ctx_safe(ctx))]
#[requires(trace_safe(trace, ctx))]
#[ensures(ctx_safe(ctx))]
#[ensures(trace_safe(trace, ctx))]
#[ensures(
    match &result {
        Ok(wasm_iovs) => wasm_iovs.len() >= 0 &&  (forall(|idx: usize|  (idx < wasm_iovs.len() && idx >= 0) ==> {
            let iov = wasm_iovs.lookup(idx);
            let buf = iov.iov_base;
            let cnt = iov.iov_len;
            // ctx.fits_in_lin_mem(buf, cnt, trace)
            (buf >= 0) && (cnt >= 0) &&
            (buf as usize) + (cnt as usize) < LINEAR_MEM_SIZE &&
            (buf <= buf + cnt)
        })),
        _ => true,
    }
)]
#[external_methods(push)]
pub fn parse_iovs(ctx: &VmCtx, iovs: u32, iovcnt: u32) -> RuntimeResult<WasmIoVecs> {
    let mut i = 0;
    let mut wasm_iovs = WasmIoVecs::new();
    while i < iovcnt {
        body_invariant!(ctx_safe(ctx));
        body_invariant!(trace_safe(trace, ctx));
        body_invariant!(wasm_iovs.len() >= 0);
        body_invariant!(
            forall(|idx: usize|  (idx < wasm_iovs.len() && idx >= 0) ==> {
                let iov = wasm_iovs.lookup(idx);
                let buf = iov.iov_base;
                let cnt = iov.iov_len;
                // ctx.fits_in_lin_mem(buf, cnt, trace)
                (buf >= 0) && (cnt >= 0) &&
                (buf as usize) + (cnt as usize) < LINEAR_MEM_SIZE &&
                (buf <= buf + cnt)
            })

        );

        let start = (iovs + i * 8) as usize;
        //TODO: Once we fix ? operatior - fix this
        //let (ptr, len) = ctx.read_u32_pair(start)?;
        let v = ctx.read_u32_pair(start);
        unwrap_result!(v);
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
    assert!(wasm_iovs.len() >= 0);
    Ok(wasm_iovs)
}
