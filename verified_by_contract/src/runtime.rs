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
    #[ensures((result == true) ==> ptr < self.memlen)]
    pub fn in_lin_mem(&self, ptr: SboxPtr) -> bool {
        return ptr < self.memlen;
    }

    // // // pre: { valid_ctx(ctx), inMemRegion(buf), cnt < ctx->memlen }
    // // // post: { buf + cnt < ctx->membase + ctx->memlen }
    #[pure]
    #[ensures(result == true ==> buf < self.memlen && buf + cnt < self.memlen)]
    pub fn fits_in_lin_mem(&self, buf: SboxPtr, cnt: usize) -> bool {
        return self.in_lin_mem(buf) && self.in_lin_mem(buf + cnt);
    }

    // #[trusted]
    #[requires(safe(self))]
    #[ensures(safe(self))]
    pub fn copy_buf_from_sandbox(&self, src: SboxPtr, n: usize) -> Option<Vec<u8>> {
        if !self.fits_in_lin_mem(src, n){
            return None;
        }
        let mut host_buffer: Vec<u8> = Vec::new();
        host_buffer.reserve_exact(n);
        self.memcpy_from_sandbox(&mut host_buffer, src, n);
        // unsafe{copy_nonoverlapping(self.mem.as_ptr().offset(src as isize), host_buffer.as_mut_ptr(), n)};
        return Some(host_buffer);
    }

    #[requires(safe(self))]
    #[ensures(safe(self))]
    pub fn copy_buf_to_sandbox(&mut self, dst: SboxPtr, src: &Vec<u8>, n: usize) -> Option<()> {
        if !self.fits_in_lin_mem(dst, n){
            return None;
        }
        self.memcpy_to_sandbox(dst, &src, n);
        Some(())
    }

    // TODO: make sure vecs have reserved enough space
    // TODO: make sure lengths of vecs are correct after copy
    #[trusted]
    #[requires(src < self.memlen)]
    #[requires(src + n < self.memlen)]
    pub fn memcpy_from_sandbox(&self, dst: &mut Vec<u8>, src: SboxPtr, n: usize){
        unsafe{copy_nonoverlapping(self.mem.as_ptr().offset(src as isize), dst.as_mut_ptr(), n)};
    }

    #[trusted]
    #[requires(dst < self.memlen)]
    #[requires(dst + n < self.memlen)]
    pub fn memcpy_to_sandbox(&mut self, dst: SboxPtr, src: &Vec<u8>, n: usize){
        unsafe{copy_nonoverlapping(src.as_ptr(), self.mem.as_mut_ptr().offset(dst as isize), n)};
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
