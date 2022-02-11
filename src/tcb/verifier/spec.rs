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
                    Effect { typ: EffectType::ReadN, f1: addr, f2: count, .. } => (addr < memlen) && (count < memlen) && (addr <= (addr + count)),
                    Effect { typ: EffectType::WriteN, f1: addr, f2: count, .. } => (addr < memlen) && (count < memlen) && (addr <= (addr + count)),
                    Effect { typ: EffectType::Shutdown, ..  } => true, // currently, all shutdowns are safe
                    Effect { typ: EffectType::FdAccess, ..  } => true,
                    Effect { typ: EffectType::PathAccess, ..  } => true,
                    Effect { typ: EffectType::NetAccess, f1: _proto, f2:addr, f3:port } => true, //  addr_in_netlist(netlist, addr as u32, port as u32),
                    Effect { typ: EffectType::SockCreation, f1: domain, f2:ty, ..  } => domain == (libc::AF_INET as usize) && (ty == (libc::SOCK_STREAM as usize) || ty == (libc::SOCK_DGRAM as usize)),
                }
            ))
        )
    }
}
