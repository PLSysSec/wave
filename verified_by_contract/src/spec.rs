use prusti_contracts::*;
use crate::types::*;

#[macro_use]
predicate! {
    fn fd_safe(ctx: &VmCtx) -> bool {
        forall(|s_fd: usize|
            (0 <= s_fd && s_fd < MAX_HOST_FDS && in_fd_map(ctx, s_fd) ==> translate_fd(ctx, s_fd) >= 0))
    }
}

#[macro_use]
predicate! {
    fn valid(ctx: &VmCtx) -> bool {
        (ctx.membase < ctx.membase + ctx.memlen)
    }
}

#[macro_use]
predicate! {
    fn safe(ctx: &VmCtx) -> bool {
        valid(ctx) && fd_safe()
    }
}
