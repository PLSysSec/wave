use crate::tcb::verifier::trace::{Effect, EffectType, Trace};
use crate::types::{VmCtx, LINEAR_MEM_SIZE};
use prusti_contracts::*;

#[cfg(feature = "verify")]
predicate! {
    pub fn ctx_safe(ctx: &VmCtx) -> bool {
        ctx.memlen == LINEAR_MEM_SIZE &&
        // ctx.mem.len() == LINEAR_MEM_SIZE &&
        ctx.argc < 1024 &&
        ctx.envc < 1024 &&
        ctx.arg_buffer.len() < 1024 * 1024 &&
        ctx.env_buffer.len() < 1024 * 1024
    }
}

#[cfg(feature = "verify")]
predicate! {
    pub fn trace_safe(trace: &Trace, memlen: usize) -> bool {
        forall(|i: usize|
            (i < trace.len() ==> (
                match trace.lookup(i) {
                    // dumb right now, just make sure count less than size of mem...
                    Effect { typ: EffectType::ReadN, f1: addr, f2: count } => (count < memlen),
                    Effect { typ: EffectType::WriteN, f1: addr, f2: count } => (count < memlen),
                    Effect { typ: EffectType::Shutdown, ..  } => true, // currently, all shutdowns are safe
                    Effect { typ: EffectType::FdAccess, ..  } => true,
                    Effect { typ: EffectType::PathAccess, ..  } => true,
                }
            ))
        )
    }
}
