#include <linux/limits.h>
#include <stdbool.h>

typedef char* hostptr;
typedef unsigned int sandboxptr;


unsigned long long membase = 0x40000000;
unsigned long long memlen =  0x10000000;

typedef struct vmctx{
    const unsigned long long membase;
    const unsigned long long memlen;
} vmctx;


sandboxptr ptr_to_sandbox(vmctx* ctx, hostptr buf)
{
    return buf - (hostptr)ctx->membase;
}


hostptr ptr_from_sandbox(vmctx* ctx, sandboxptr buf)
{
    return buf + (hostptr)ctx->membase;
}


// sandboxptrs 
bool inBounds(vmctx* ctx, sandboxptr ptr){
    return (ptr >= ctx->membase) && (ptr <= (ctx->membase + ctx->memlen));  
}

sandboxptr sized_buf_to_sandbox(vmctx* ctx, hostptr buf, size_t size)
{
    return ptr_to_sandbox(ctx, buf);
}

// returns pointer if success, or null if memory violation
hostptr sized_buf_from_sandbox(vmctx* ctx, sandboxptr buf, size_t size)
{
    if ((size < ctx->memlen) && inBounds(ctx, ctx->membase + buf) && inBounds(ctx, ctx->membase + buf + size)){
        return ptr_from_sandbox(ctx, buf);
    }
    else{
        return NULL;
    }
}


sandboxptr path_to_sandbox(vmctx* ctx, hostptr buf)
{
    return sized_buf_to_sandbox(ctx, buf, PATH_MAX);
}


hostptr path_from_sandbox(vmctx* ctx, sandboxptr buf)
{
    return sized_buf_from_sandbox(ctx, buf, PATH_MAX);
}


// inline int sealed_to_sandbox()
// {
//     return 2;
// }


// inline int sealed_from_sandbox()
// {
//     return 2;
// }
