use crate::external_specs::option::*;
use crate::types::*;
use prusti_contracts::*;
use RuntimeError::*;
use std::ptr::copy_nonoverlapping;

// predicate! {
//     fn fd_safe(ctx: &FdMap) -> bool {
//         forall(|s_fd: SboxFd|
//             (s_fd < MAX_SBOX_FDS ==> ctx.lookup(s_fd) >= 0))
//     }
// }

//Do we need this?
// predicate! {
//     fn valid(ctx: &VmCtx) -> bool {
//         (ctx.membase < ctx.membase + ctx.memlen)
//     }
// }

predicate! {
    fn safe(ctx: &VmCtx) -> bool {
        true
    }
}

// pre: { }
// post: { validCtx() }
//TODO: instantiate stdin,stdout,stderr
#[ensures(safe(&result))]
pub fn fresh_ctx() -> VmCtx {
    let memlen = LINEAR_MEM_SIZE;
    let mem = vec![0; memlen];
    let fdmap = FdMap::new();
    let ctx = VmCtx {
        mem,
        memlen,
        fdmap,
        errno: Success,
    };
    return ctx;
}

impl VmCtx {
    // pre: { valid_ctx(ctx) }
    // post: { buf >= ctx->membase }
    #[pure]
    pub fn in_lin_mem(&self, ptr: SboxPtr) -> bool {
        return ptr <= self.memlen;
    }

    // // // pre: { valid_ctx(ctx), inMemRegion(buf), cnt < ctx->memlen }
    // // // post: { buf + cnt < ctx->membase + ctx->memlen }
    #[pure]
    pub fn fits_in_lin_mem(&self, buf: SboxPtr, cnt: usize) -> bool {
        return self.in_lin_mem(buf) && self.in_lin_mem(buf + cnt);
    }

    #[trusted]
    #[requires(safe(self))]
    #[ensures(safe(self))]
    pub fn copy_buf_from_sandbox(&self, src: SboxPtr, n: usize) -> Option<Vec<u8>> {
        if !self.fits_in_lin_mem(src, n){
            return None;
        }
        let mut host_buffer: Vec<u8> = Vec::new();
        host_buffer.reserve_exact(n);
        unsafe{copy_nonoverlapping(self.mem.as_ptr().offset(src as isize), host_buffer.as_mut_ptr(), n)};
        return Some(host_buffer);
    }

    // // pre: { ... }
    // // post: { ... }
    // void copy_buf_to_sandbox(ctx: &VmCtx, dst: SboxPtr, src: HostPtr, n: size){
    //     memcpy(swizzle(ctx, dst), src, n);
    // }

    // // pre: {}
    // // post:  { PathSandboxed(out_path) }
    pub fn resolve_path(&self, in_path: Vec<u8>) -> Vec<u8>{
        //TODO: finish
        //memcpy(out_path, in_path, PATH_MAX);
        return in_path;
    }
}
