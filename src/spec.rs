use crate::trace::{Effect, Trace};
use crate::types::VmCtx;
use prusti_contracts::*;

#[cfg(feature = "verify")]
predicate! {
    pub fn trace_safe(ctx: &VmCtx, trace: &Trace) -> bool {
        forall(|i: usize|
            (i < trace.len() ==> (
                match trace.lookup(i) {
                    // dumb right now, just make sure count less than size of mem...
                    Effect::ReadN { count } => (count < ctx.memlen),
                    Effect::WriteN { count } => (count < ctx.memlen),
                    Effect::Shutdown => true, // currently, all shutdowns are safe
                }
            ))
        )
    }
}
