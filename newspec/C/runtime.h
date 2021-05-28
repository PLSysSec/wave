#include <linux/limits.h>
#include <stdbool.h>
#include <string.h>
#include <smack.h>
#include <smack-contracts.h>

#define MAX_SANDBOX_FDS 8
#define MAX_HOST_FDS 1024

#define VALID_CTX(ctx) (ctx->membase < ctx->membase + ctx->memlen)

#define SFI_SAFE(ctx) (true) //This is handled by the builtin memory safety checker

#define FD_SAFE(ctx) (true) // Unimplemented - I think I want to move to rust for better types to implement this 
#define PATH_SAFE(ctx) (true) // Unimplemented - I think I want to move to rust for better types to implement this 
#define RESOURCE_SAFE(ctx) FD_SAFE(ctx) && PATH_SAFE(ctx) 

#define SAFE(ctx) VALID_CTX(ctx) && SFI_SAFE(ctx) && RESOURCE_SAFE(ctx) 

typedef char* hostptr;
typedef unsigned int sandboxptr;

typedef int host_fd;
typedef int sandbox_fd;

typedef struct vmctx{
    const char* membase;
    const size_t memlen;
    host_fd fd_sbx_to_host[MAX_SANDBOX_FDS];
    sandbox_fd fd_host_to_sbx[MAX_HOST_FDS];
    int counter;
} vmctx;

// pre: { }
// post: { validCtx() }
vmctx fresh_ctx(){
    host_fd fd_sbx_to_host[MAX_SANDBOX_FDS] = {-1};
    sandbox_fd fd_host_to_sbx[MAX_HOST_FDS] = {-1};
    // size_t memlen = 1024 * 1024;
    size_t memlen = __VERIFIER_nondet_unsigned_long_long();
    assume(memlen >= 1024 * 1024 && memlen <= 4*1024*1024*1024);
    char* membase = malloc(memlen);
    if (membase == NULL || (membase <= (char*)memlen) || (membase + memlen <= membase) ){
        free(membase);
        exit(-1);
    }
     
    // assume(membase > (char*)memlen);
    vmctx ctx =  {membase, memlen, *fd_sbx_to_host, *fd_host_to_sbx, 0};
    return ctx;
}

// pre: { valid_ctx(ctx) }
// post: { buf >= ctx->membase }
bool inMemRegion(vmctx* ctx, void* ptr) { 
    requires(SAFE(ctx));
    ensures(SAFE(ctx));
    return (ptr >= (void*)ctx->membase) && (ptr <= (void*)(ctx->membase + ctx->memlen)); 
}


// pre: { valid_ctx(ctx), inMemRegion(buf), cnt < ctx->memlen }
// post: { buf + cnt < ctx->membase + ctx->memlen }
bool fitsInMemRegion(vmctx* ctx, void* buf, size_t cnt) { 
    requires(SAFE(ctx));
    ensures(SAFE(ctx));
    return (char*)(buf + cnt) < (ctx->membase + ctx->memlen);
}

// ptr_to_sandbox
// pre: { !inMemRegion(buf) }
// post: { inMemRegion(buf) }
sandboxptr reverse_swizzle(vmctx* ctx, hostptr buf)
{
    requires(SAFE(ctx));
    ensures(SAFE(ctx));
    return (sandboxptr)(buf - ctx->membase);
}

//ptr_from_sandbox
// pre:  { inMemRegion(ctx, ptr)  }
// post: { !inMemRegion(ctx, ptr) }
hostptr swizzle(vmctx* ctx, sandboxptr ptr)
{
    requires(SAFE(ctx));
    ensures(SAFE(ctx));
    hostptr hptr = (hostptr)(ptr + ctx->membase);
    return hptr;
}

// pre: { ... }
// post: { ... }
hostptr copy_buf_from_sandbox(vmctx *ctx, const sandboxptr src, size_t n){
    requires(SAFE(ctx));
    ensures(SAFE(ctx));
    hostptr swizzled_src = swizzle(ctx, src);
    if (!inMemRegion(ctx, swizzled_src) || !fitsInMemRegion(ctx, swizzled_src, n)){
        return NULL;
    }
    char* host_buffer = malloc(n);
    if (host_buffer == NULL){
        return NULL;
    }
    memcpy(host_buffer, swizzled_src, n);
    return host_buffer;
}

// pre: { ... }
// post: { ... }
void copy_buf_to_sandbox(vmctx *ctx, sandboxptr dst, const hostptr src, size_t n){
    requires(SAFE(ctx));
    ensures(SAFE(ctx));
    memcpy(swizzle(ctx, dst), src, n);
}


// pre: {}
// post:  { PathSandboxed(out_path) }
void resolve_path(vmctx* ctx, const char* in_path, char* out_path){ 
    requires(SAFE(ctx));
    ensures(SAFE(ctx));
    //TODO: finish
    memcpy(out_path, in_path, PATH_MAX);
}


// pre: { v_fd < MAX_SANDBOX_FDS }
// post { }
bool in_fd_map(vmctx* ctx, sandbox_fd v_fd){
    requires(SAFE(ctx));
    requires( v_fd >= 0 && v_fd < MAX_SANDBOX_FDS );
    ensures(SAFE(ctx));
    return (ctx->fd_sbx_to_host[v_fd] != -1);
}

// pre: { fd < MAX_HOST_FDS }
// post { }
bool in_rev_fd_map(vmctx* ctx, host_fd fd){
    requires(SAFE(ctx));
    ensures(SAFE(ctx));
    return (ctx->fd_host_to_sbx[fd] != -1);
}

// pre: { !inFdMap(ctx, v_fd), !inRevFdMap(ctx, fd) }
// post {  inFdMap(ctx, v_fd), translateFd(ctx, v_fd) == fd }
sandbox_fd create_seal(vmctx* ctx, host_fd h_fd, sandbox_fd v_fd){
    requires(SAFE(ctx));
    ensures(SAFE(ctx));
    // requires( h_fd >= 0 && h_fd < MAX_HOST_FDS);
    // requires( v_fd >= 0 && v_fd < MAX_SANDBOX_FDS);
    if  (h_fd < 0 || h_fd >= MAX_HOST_FDS){
        return -1;
    }
    if  (v_fd < 0 || v_fd >= MAX_HOST_FDS){
        return -1;
    }

    // ensures(VALID_CTX(ctx));
    // sandbox_fd fresh_fd = ctx->counter++;
    ctx->fd_sbx_to_host[v_fd] = h_fd;
    ctx->fd_host_to_sbx[h_fd] = v_fd;
    return v_fd;
}

// pre: { inFdMap(ctx, v_fd), inRevFdMap(ctx, translate_fd(fd)) }
// post { !inFdMap(ctx, v_fd), !inRevFdMap(ctx, translateFd(v_fd)) }
void delete_seal(vmctx* ctx, sandbox_fd v_fd){
    requires(SAFE(ctx));
    requires( v_fd >= 0 && v_fd < MAX_SANDBOX_FDS);
    ensures(SAFE(ctx));
    // if  (v_fd < 0 || v_fd >= MAX_HOST_FDS){
    //     return;
    // }
    host_fd h_fd = ctx->fd_sbx_to_host[v_fd];
    if (h_fd >= 0 && h_fd < MAX_HOST_FDS){
        ctx->fd_sbx_to_host[v_fd] = -1;
        ctx->fd_host_to_sbx[h_fd] = -1;
    }
}

// sandbox_fd reverse_translate(vmctx* ctx, host_fd h_fd)
// {
//     return ctx->fd_host_to_sbx[h_fd];
// }

// pre: { v_fd in ctx->fdMap }
// post: { isOpenFd(result) }
host_fd translate_fd(vmctx* ctx, sandbox_fd sbx_fd)
{
    requires( sbx_fd >= 0 && sbx_fd < MAX_SANDBOX_FDS);
    requires(SAFE(ctx));
    ensures(SAFE(ctx));
    host_fd fd = ctx->fd_sbx_to_host[sbx_fd];
    return fd;
}
