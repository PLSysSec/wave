use prusti_contracts::*;
use std::ptr::copy_nonoverlapping;
use crate::types::*;
// use crate::spec::*;
use crate::spec::safe;

// pre: { }
// post: { validCtx() }
//TODO: instantiate stdin,stdout,stderr
#[ensures(safe(ctx))]
pub fn fresh_ctx() -> VmCtx{
    let memlen = unsafe{__VERIFIER_nondet_u64() as usize};
    //let mem = smack::vec![0; memlen];
    let membase = unsafe{malloc(memlen) as usize};
    let fd_sbox_to_host = [-1; MAX_HOST_FDS as usize];
    let counter = 0;

    let ctx = VmCtx {
        membase: membase,
        memlen: memlen,
        fd_sbox_to_host: fd_sbox_to_host, 
        counter: counter,
    };
    // smack::assume!(memlen >= 1024 * 1024 && memlen <= 4*1024*1024*1024);
    return ctx;
}

impl VmCtx {
    // pre: { valid_ctx(ctx) }
    // post: { buf >= ctx->membase }
    #[pure]
    pub fn in_mem_region(&self, ptr: usize) -> bool { 
        // return true;
        return (ptr >= self..membase) && (ptr <= (self.membase + self.memlen)); 
    }


    // // pre: { valid_ctx(ctx), inMemRegion(buf), cnt < ctx->memlen }
    // // post: { buf + cnt < ctx->membase + ctx->memlen }
    #[pure]
    pub fn fits_in_mem_region(&self, buf: usize, cnt: usize) -> bool { 
        // return true;
        return (buf + cnt) < (self.membase + self.memlen);
    }

    // //ptr_from_sandbox
    // // pre:  { inMemRegion(ctx, ptr)  }
    // // post: { !inMemRegion(ctx, ptr) }
    #[pure]
    #[requires(in_mem_region(ptr))]
    #[ensures(!in_mem_region(result))]
    pub fn swizzle(ctx: &self, ptr: SboxPtr) -> HostPtr
    {
        let hptr: HostPtr = self.membase + (ptr as usize);
        return hptr;
    }

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


    // pre: { v_fd < MAX_SANDBOX_FDS }
    // post { }
    #[pure]
    #[requires()]
    pub fn in_fd_map(ctx: &VmCtx, v_fd: SboxFd) -> bool {
        return ctx.fd_sbox_to_host[v_fd as usize] != -1;
    }


    #[requires(h_fd >= 0 && h_fd < MAX_HOST_FDS)]
    #[requires(v_fd >= 0 && v_fd < MAX_SBOX_FDS)]
    #[requires(!in_fd_map(ctx, v_fd))]
    // #[requires(!in_fd_map(ctx, translate_fd(ctx, v_fd))) ]
    #[ensures(in_fd_map(ctx, v_fd))]
    // #[ensures(in_fd_map(ctx, translate_fd(ctx, v_fd)))]
    pub fn create_seal(ctx: &mut VmCtx, h_fd: HostFd, v_fd: SboxFd){
        ctx.fd_sbox_to_host[v_fd as usize] = h_fd;
    }

    #[requires(v_fd >= 0 && v_fd < MAX_SBOX_FDS)]
    #[ensures(in_fd_map(ctx, v_fd))]
    // #[ensures(in_fd_map(ctx, translate_fd(ctx, v_fd)))]
    #[ensures(!in_fd_map(ctx, v_fd))]
    // #[ensures(!in_fd_map(ctx, translate_fd(ctx, v_fd))) ]
    pub fn delete_seal(ctx: &mut VmCtx, v_fd: SboxFd){
        let h_fd = ctx.fd_sbox_to_host[v_fd as usize];
        if (h_fd >= 0) && (h_fd < MAX_HOST_FDS){
            ctx.fd_sbox_to_host[v_fd as usize] = -1;
            ctx.fd_host_to_sbox[h_fd as usize] = -1;
        }
    }

    #[pure]
    #[requires(in_fd_map(sbox_fd))]
    pub fn translate_fd(ctx: &VmCtx, sbox_fd: SboxFd) -> HostFd
    {
        let fd = ctx.fd_sbox_to_host[sbox_fd as usize];
        return fd;
    }

}
