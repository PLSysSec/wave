use crate::external_specs::option::*;
use crate::types::*;
use prusti_contracts::*;
use RuntimeError::*;

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

    // //ptr_from_sandbox
    // // pre:  { inMemRegion(ctx, ptr)  }
    // // post: { !inMemRegion(ctx, ptr) }
    // #[pure]
    // #[requires(self.in_mem_region(ptr))]
    // #[ensures(!self.in_mem_region(result))]
    // pub fn swizzle(&self, ptr: SboxPtr) -> HostPtr
    // {
    //     let hptr: HostPtr = self.membase + (ptr as usize);
    //     return hptr;
    // }

    // // // pre: { ... }
    // // // post: { ... }
    // pub fn copy_buf_from_sandbox(ctx: &VmCtx, src: SboxPtr, n: usize) -> Option<HostPtr>{
    //     let swizzled_src = swizzle(ctx, src);
    //     if !in_mem_region(ctx, swizzled_src) || !fits_in_mem_region(ctx, swizzled_src, n){
    //         return None;
    //     }

    //     let mut host_buffer: [u8; PATH_MAX] = [0; PATH_MAX];
    //     unsafe{copy_nonoverlapping(swizzled_src as *mut u8, host_buffer.as_mut_ptr(), PATH_MAX)};
    //     return Some(host_buffer.as_ptr() as usize);

    //     // char* host_buffer = malloc(n);
    //     // if (host_buffer == NULL){
    //     //     return NULL;
    //     // }
    //     // memcpy(host_buffer, swizzled_src, n);
    //     // return host_buffer;
    // }

    // // pre: { ... }
    // // post: { ... }
    // void copy_buf_to_sandbox(ctx: &VmCtx, dst: SboxPtr, src: HostPtr, n: size){
    //     memcpy(swizzle(ctx, dst), src, n);
    // }

    // // pre: {}
    // // post:  { PathSandboxed(out_path) }
    // pub fn resolve_path(ctx: &VmCtx, in_path: HostPtr) -> HostPtr{
    //     //TODO: finish
    //     //memcpy(out_path, in_path, PATH_MAX);
    //     return in_path;
    // }
}
