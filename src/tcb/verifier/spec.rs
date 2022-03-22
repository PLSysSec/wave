use crate::tcb::verifier::trace::{Effect, EffectType, Trace};
use crate::types::{addr_in_netlist, VmCtx, HOMEDIR_FD, LINEAR_MEM_SIZE};
use crate::tcb::path::path_safe;
use crate::tcb::misc::netlist_unmodified;
use prusti_contracts::*;
use crate::setup_teardown::{mem_setup_correctly, raw_ptr};

#[cfg(feature = "verify")]
predicate! {
    pub fn ctx_safe(ctx: &VmCtx) -> bool {
        ctx.memlen == LINEAR_MEM_SIZE &&
        // ctx.mem.len() == LINEAR_MEM_SIZE &&
        //ctx.homedir_host_host == ctx.fdmap[HOMEDIR_FD] &&
        // ctx.fdmap.lookup(HOMEDIR_FD).is_ok() &&
        ctx.argc < 1024 &&
        ctx.envc < 1024 &&
        ctx.arg_buffer.len() < 1024 * 1024 &&
        ctx.env_buffer.len() < 1024 * 1024 &&
        netlist_unmodified(&ctx.netlist) && 
        mem_setup_correctly(raw_ptr(&(ctx.mem.as_slice())))
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
                    Effect { typ: EffectType::PathAccessAt, f1: dir_fd, f2:_, f3:_, p: Some(path), should_follow: Some(b) } => dir_fd == ctx.homedir_host_fd.to_raw() && path.len() == 4096 && path_safe(&path, b),
                    Effect { typ: EffectType::NetAccess, f1: _proto, f2:addr, f3:port, .. } => addr_in_netlist(&ctx.netlist, addr as u32, port as u32),
                    Effect { typ: EffectType::SockCreation, f1: domain, f2:ty, ..  } => domain == (libc::AF_INET as usize) && (ty == (libc::SOCK_STREAM as usize) || ty == (libc::SOCK_DGRAM as usize)),
                    _ => false,
                }
            ))
        )
    }
}
