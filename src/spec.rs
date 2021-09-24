use crate::types::VmCtx;
use crate::trace::{Trace, Effect};

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
