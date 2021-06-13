#include <stdlib.h>
#include "runtime.h"
#include "wrappers.h"
#include <smack.h>
#include <smack-contracts.h> 

//Non-deterministically select one system call and invoke it with symbolic args
void make_one_syscall(vmctx* ctx){
    int choice = __VERIFIER_nondet_int();
    switch(choice){
        case 0:
        {
        int pathname = __VERIFIER_nondet_unsigned_int();
        int fd = __VERIFIER_nondet_int();
        wasi_open(ctx, fd, pathname);
        break;
        }

        case 1:
        {
        int fd = __VERIFIER_nondet_int();
        wasi_close(ctx, fd);
        break; 
        }
        
        case 2: 
        {
        int fd = __VERIFIER_nondet_int();
        int buf = __VERIFIER_nondet_unsigned_int();
        int size = __VERIFIER_nondet_unsigned_int();
        wasi_read(ctx, fd, buf, size);
        break;
        }
        case 3:
        {
        int fd = __VERIFIER_nondet_int();
        int buf = __VERIFIER_nondet_unsigned_int();
        int size = __VERIFIER_nondet_unsigned_int();
        wasi_write(ctx, fd, buf, size);
        break;
        }
        default:
        break; 
    }

}


// void assert_simple(vmctx* ctx, sandbox_fd sbx_fd, host_fd h_fd){
//     // sandbox_fd sbx_fd = __VERIFIER_nondet_int();
//     // assume(sbx_fd >= 0 && sbx_fd < MAX_SANDBOX_FDS);
//     // assume(sbx_fd == 0 || sbx_fd == 1);
//     // assert(ctx->fd_sbx_to_host[sbx_fd] >= 0);

//     if (in_fd_map(ctx, sbx_fd)){
//     //     // host_fd dummy_h_fd = translate_fd(ctx, sbx_fd);
//         assert(ctx->fd_sbx_to_host[sbx_fd] >= 0 && ctx->fd_sbx_to_host[sbx_fd] < MAX_HOST_FDS);
//     //     // assert(dummy_h_fd >= 0 && dummy_h_fd < MAX_HOST_FDS);
//     }
//     return;
// }

// void assume_simple(vmctx* ctx, sandbox_fd sbx_fd, host_fd h_fd){
//     // sandbox_fd sbx_fd = __VERIFIER_nondet_int();
//     // assume(sbx_fd >= 0 && sbx_fd < MAX_SANDBOX_FDS);
//     // assume(sbx_fd == 0 || sbx_fd == 1);
//     // assume(ctx->fd_sbx_to_host[sbx_fd] >= 0);
//     if (in_fd_map(ctx, sbx_fd)){
//         assume(ctx->fd_sbx_to_host[sbx_fd] >= 0 && ctx->fd_sbx_to_host[sbx_fd] < MAX_HOST_FDS);
//     }
//     return;
// }


// Harness for verifier
//1. Create a fresh sandbox context
//2. Invoke system calls
int main(){
    // 1. Check that our initial state is safe;
    // vmctx ctx = fresh_ctx();
    // assert_safe(&ctx);
    // free((void*)ctx.membase);
   
    // 2. check that an arbitrary context that satisfies our safety invariant
    // is safe afterwards.
    vmctx sym_ctx = symbolic_ctx();
    
    sandbox_fd sbx_fd = __VERIFIER_nondet_int();
    assume(sbx_fd >= 0 && sbx_fd < MAX_SANDBOX_FDS);
    host_fd h_fd = __VERIFIER_nondet_int();
    assume(h_fd >= 0 && h_fd < MAX_HOST_FDS);

    assume_safe(&sym_ctx, sbx_fd, h_fd);
    assert_safe(&sym_ctx, sbx_fd, h_fd);
    // assume_safe(&sym_ctx);
    // // make_one_syscall(&sym_ctx);
    // assert_safe(&sym_ctx);
    // make_one_syscall(&ctx);
    // assert_safe(&ctx);
    // make_one_syscall(&ctx);
    // assert(safe(&ctx));
    free((void*)sym_ctx.membase);
}

