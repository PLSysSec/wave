use crate::tcb::verifier::trace::{Effect, EffectType, Trace};
use crate::types::{addr_in_netlist, VmCtx, HOMEDIR_FD, LINEAR_MEM_SIZE};
use prusti_contracts::*;

#[cfg(feature = "verify")]
predicate! {
    pub fn ctx_safe(ctx: &VmCtx) -> bool {
        ctx.memlen == LINEAR_MEM_SIZE &&
        // ctx.mem.len() == LINEAR_MEM_SIZE &&
        // TODO(Evan): ctx.homedir is no longer an fd. What should this check instead?
        // ctx.homedir == HOMEDIR_FD &&
        ctx.argc < 1024 &&
        ctx.envc < 1024 &&
        ctx.arg_buffer.len() < 1024 * 1024 &&
        ctx.env_buffer.len() < 1024 * 1024
    }
}

// TODO: make the memory-safety spec more convincing

#[cfg(feature = "verify")]
predicate! {
    pub fn trace_safe(trace: &Trace, ctx: &VmCtx) -> bool {
        forall(|i: usize|
            (i < trace.len() ==> (
                match trace.lookup(i) {
                    // dumb right now, just make sure count less than size of mem...
                    Effect { typ: EffectType::ReadN, f1: addr, f2: count, .. } => (addr < ctx.memlen) && (count < ctx.memlen) && (addr <= (addr + count)),
                    Effect { typ: EffectType::WriteN, f1: addr, f2: count, .. } => (addr < ctx.memlen) && (count < ctx.memlen) && (addr <= (addr + count)),
                    Effect { typ: EffectType::Shutdown, ..  } => true, // currently, all shutdowns are safe
                    Effect { typ: EffectType::FdAccess, ..  } => true,
                    Effect { typ: EffectType::PathAccessAt, f1: dir_fd, ..  } => true /*dir_fd == ctx.HOMEDIR_FD*/,
                    Effect { typ: EffectType::NetAccess, f1: _proto, f2:addr, f3:port } => addr_in_netlist(&ctx.netlist, addr as u32, port as u32),
                    Effect { typ: EffectType::SockCreation, f1: domain, f2:ty, ..  } => domain == (libc::AF_INET as usize) && (ty == (libc::SOCK_STREAM as usize) || ty == (libc::SOCK_DGRAM as usize)),
                }
            ))
        )
    }
}
