use crate::tcb::verifier::trace::{Effect, Trace};
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
                    Effect::ReadN(count) => (count < memlen),
                    Effect::WriteN(count) => (count < memlen),
                    Effect::Shutdown => true, // currently, all shutdowns are safe
                    Effect::FdAccess => true,
                    Effect::PathAccess => true,
                }
            ))
        )
    }
}
