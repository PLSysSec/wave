// #include <linux/limits.h>
// #include <stdbool.h>
// #include <string.h>
use smack::*;


pub const MAX_SANDBOX_FDS: i32 = 8;
pub const MAX_HOST_FDS: i32 = 1024;

// #define VALID_CTX(ctx) (ctx->membase < ctx->membase + ctx->memlen)

// #define SFI_SAFE(ctx) (true) //This is handled by the builtin memory safety checker

// #define FD_SAFE(ctx) (true) // Unimplemented - I think I want to move to rust for better types to implement this 
// #define PATH_SAFE(ctx) (true) // Unimplemented - I think I want to move to rust for better types to implement this 
// #define RESOURCE_SAFE(ctx) FD_SAFE(ctx) && PATH_SAFE(ctx) 

// #define SAFE(ctx) VALID_CTX(ctx) && SFI_SAFE(ctx) && RESOURCE_SAFE(ctx) 

//typedef char* hostptr;
pub type HostPtr = usize;
pub type SboxPtr = u32;

pub type HostFd = i32;
pub type SboxFd = i32;



pub struct VmCtx {
    pub membase: usize,
    pub memlen: usize,
    pub fd_sbox_to_host: [HostFd; MAX_HOST_FDS as usize], 
    pub fd_host_to_sbox: [SboxFd; MAX_SANDBOX_FDS as usize],
    pub counter: i32,
}

// pre: { }
// post: { validCtx() }
//TODO: instantiate stdin,stdout,stderr
pub fn fresh_ctx() -> VmCtx{
    let memlen = unsafe{__VERIFIER_nondet_u64() as usize};
    //let mem = smack::vec![0; memlen];
    let membase = unsafe{malloc(memlen) as usize};
    let fd_sbox_to_host = [-1; MAX_HOST_FDS as usize];
    let fd_host_to_sbox = [-1; MAX_SANDBOX_FDS as usize];
    let counter = 0;

    let ctx = VmCtx {
        membase: membase,
        memlen: memlen,
        fd_sbox_to_host: fd_sbox_to_host, 
        fd_host_to_sbox: fd_host_to_sbox,
        counter: counter,
    };
    smack::assume!(memlen >= 1024 * 1024 && memlen <= 4*1024*1024*1024);
    return ctx;
}

// pre: { valid_ctx(ctx) }
// post: { buf >= ctx->membase }
pub fn in_mem_region(ctx: &VmCtx, ptr: usize) -> bool { 
    // return true;
    return (ptr >= ctx.membase) && (ptr <= (ctx.membase + ctx.memlen)); 
}


// // pre: { valid_ctx(ctx), inMemRegion(buf), cnt < ctx->memlen }
// // post: { buf + cnt < ctx->membase + ctx->memlen }
pub fn fits_in_mem_region(ctx: &VmCtx, buf: usize, cnt: usize) -> bool { 
    // return true;
    return (buf + cnt) < (ctx.membase + ctx.memlen);
}

// // ptr_to_sandbox
// // pre: { !inMemRegion(buf) }
// // post: { inMemRegion(buf) }
// pub fn reverse_swizzle(ctx: &VmCtx, buf: HostPtr) -> SboxPtr
// {
//     return (sandboxptr)(buf - ctx->membase);
// }

// //ptr_from_sandbox
// // pre:  { inMemRegion(ctx, ptr)  }
// // post: { !inMemRegion(ctx, ptr) }
pub fn swizzle(ctx: &VmCtx, ptr: SboxPtr) -> HostPtr
{
    let hptr: HostPtr = ctx.membase + (ptr as usize);
    return hptr;
}

// // pre: { ... }
// // post: { ... }
// pub fn copy_buf_from_sandbox(ctx: &VmCtx, src: SboxPtr, n: size) -> HostPtr{
//     hostptr swizzled_src = swizzle(ctx, src);
//     if (!inMemRegion(ctx, swizzled_src) || !fitsInMemRegion(ctx, swizzled_src, n)){
//         return NULL;
//     }
//     char* host_buffer = malloc(n);
//     if (host_buffer == NULL){
//         return NULL;
//     }
//     memcpy(host_buffer, swizzled_src, n);
//     return host_buffer;
// }

// // pre: { ... }
// // post: { ... }
// void copy_buf_to_sandbox(ctx: &VmCtx, dst: SboxPtr, src: HostPtr, n: size){
//     memcpy(swizzle(ctx, dst), src, n);
// }


// // pre: {}
// // post:  { PathSandboxed(out_path) }
// pub fn resolve_path(ctx: &VmCtx, in_path: String, out_path: String){ 
//     //TODO: finish
//     memcpy(out_path, in_path, PATH_MAX);
// }


// pre: { v_fd < MAX_SANDBOX_FDS }
// post { }
pub fn in_fd_map(ctx: &VmCtx, v_fd: SboxFd) -> bool {
    // requires( v_fd >= 0 && v_fd < MAX_SANDBOX_FDS );
    return ctx.fd_sbox_to_host[v_fd as usize] != -1;
}

// // pre: { fd < MAX_HOST_FDS }
// // post { }
pub fn in_rev_fd_map(ctx: &VmCtx, h_fd: HostFd) -> bool {
    return ctx.fd_host_to_sbox[h_fd as usize] != -1;
}

// // pre: { !inFdMap(ctx, v_fd), !inRevFdMap(ctx, fd) }
// // post {  inFdMap(ctx, v_fd), translateFd(ctx, v_fd) == fd }
pub fn create_seal(ctx: &mut VmCtx, h_fd: HostFd, v_fd: SboxFd) -> SboxFd{
    // requires( h_fd >= 0 && h_fd < MAX_HOST_FDS);
    // requires( v_fd >= 0 && v_fd < MAX_SANDBOX_FDS);
    if h_fd < 0 || h_fd >= MAX_HOST_FDS{
        return -1;
    }
    if v_fd < 0 || v_fd >= MAX_HOST_FDS{
        return -1;
    }

    // ensures(VALID_CTX(ctx));
    // sandbox_fd fresh_fd = ctx->counter++;
    ctx.fd_sbox_to_host[v_fd as usize] = h_fd;
    ctx.fd_host_to_sbox[h_fd as usize] = v_fd;
    return v_fd;
}

// // pre: { inFdMap(ctx, v_fd), inRevFdMap(ctx, translate_fd(fd)) }
// // post { !inFdMap(ctx, v_fd), !inRevFdMap(ctx, translateFd(v_fd)) }
pub fn delete_seal(ctx: &mut VmCtx, v_fd: SboxFd){
    // requires( v_fd >= 0 && v_fd < MAX_SANDBOX_FDS);
    // if  (v_fd < 0 || v_fd >= MAX_HOST_FDS){
    //     return;
    // }
    let h_fd = ctx.fd_sbox_to_host[v_fd as usize];
    if (h_fd >= 0) && (h_fd < MAX_HOST_FDS){
        ctx.fd_sbox_to_host[v_fd as usize] = -1;
        ctx.fd_host_to_sbox[h_fd as usize] = -1;
    }
}

pub fn reverse_translate(ctx: &VmCtx, h_fd: HostFd) -> SboxFd
{
     return ctx.fd_host_to_sbox[h_fd as usize];
 }

// // pre: { v_fd in ctx->fdMap }
// // post: { isOpenFd(result) }
pub fn translate_fd(ctx: &VmCtx, sbox_fd: SboxFd) -> HostFd
{
    // requires( sbox_fd >= 0 && sbox_fd < MAX_SANDBOX_FDS);
    let fd = ctx.fd_sbox_to_host[sbox_fd as usize];
    return fd;
}



pub fn assert_safe(ctx: &VmCtx){
    let sbox_fd = unsafe{__VERIFIER_nondet_i32()};
    smack::assume!(sbox_fd >= 0 && sbox_fd < MAX_SANDBOX_FDS);
    let h_fd = unsafe{__VERIFIER_nondet_i32()};
    smack::assume!(h_fd >= 0 && h_fd < MAX_HOST_FDS);

    //check range of host_fds
    //check for bijection
    if in_fd_map(ctx, sbox_fd){
        let dummy_h_fd = translate_fd(ctx, sbox_fd);
        smack::assert!(dummy_h_fd >= 0 && dummy_h_fd < MAX_HOST_FDS);
    }

    if in_rev_fd_map(ctx, h_fd){
        let dummy_sbox_fd = reverse_translate(ctx, h_fd);
        smack::assert!(dummy_sbox_fd >= 0 && dummy_sbox_fd < MAX_SANDBOX_FDS);
    }

    // assert(VALID_CTX(ctx));
    return;
}
