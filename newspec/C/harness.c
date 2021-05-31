#include <stdlib.h>
#include "runtime.h"
#include "wrappers.h"
#include <smack.h>
#include <smack-contracts.h> 

//Non-deterministically select one system call and invoke it with symbolic args
void make_one_syscall(vmctx* ctx){
    // __SMACK_code("requires (@ == -$1);", ctx->fd_sbx_to_host[0]);
    //requires(SAFE(ctx));
    //ensures(SAFE(ctx));
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

// Harness for verifier
//1. Create a fresh sandbox context
//2. Invoke system calls
int main(){
    // assert(false);
    vmctx ctx = fresh_ctx();
    // assert(false);
    // assume(ctx.membase < ctx.membase + ctx.memlen);
    assert_safe(&ctx);
    // assert(safe(&ctx));
    // assert(SAFE(ctx));
    // assume(ctx.fd_sbx_to_host[0] == -1);
    //forall x:ref :: $processStatus[x] == $process_uninitialized);
    //valid sandbox fd have valid host fd
    // __SMACK_code("assume (forall sfd: ref :: @.fd_sbx_to_host[sfd] == -$1);", ctx);
    // __SMACK_code("assume (-$1 == -$1);");
    // sandbox_fd i = ctx.fd_sbx_to_host[0];
    // __SMACK_code("assert (@ == -$1);", ctx.fd_sbx_to_host[0]);
    // __SMACK_code("assert (@ == -$1);", ctx.fd_sbx_to_host[1]);
    // __SMACK_code("assert (@ == -$1);", ctx.fd_sbx_to_host[2]);
    // __SMACK_code("assert (@ == -$1);", ctx.fd_sbx_to_host[3]);
    // __SMACK_code("assert (@ == -$1);", ctx.fd_sbx_to_host[4]);
    // __SMACK_code("assert (@ == -$1);", ctx.fd_sbx_to_host[5]);
    // __SMACK_code("assert (@ == -$1);", ctx.fd_sbx_to_host[6]);
    // __SMACK_code("assert (@ == -$1);", ctx.fd_sbx_to_host[7]);

    // __SMACK_code("assert (@.fd_sbx_to_host[0] == -$1);", ctx);
    // __SMACK_CODE("assume (forall x: ref :: $sle.ref.bool(dst,x) && $slt.ref.bool(x,$add.ref(dst,len)) ==> M.ret[x] == M.src[$add.ref($sub.ref(src,dst),x)]);");
    // __SMACK_code("assume (forall x: ref :: $sle.ref.bool(0,x) && $slt.ref.bool(x,8) ==> true);");

    // assert(forall)

    //__SMACK_code("forall fd:ref :: (fd == -1) || (fd < 8 && fd > 0  inRevFdMap(@ , fd) => inFdMap(ctx, translateFd(@ , fd));", ctx, ctx);
    // forall fd. inRevFdMap(ctx fd) => inFdMap(ctx, translateFd(ctx, fd))
    make_one_syscall(&ctx);
    assert_safe(&ctx);
    // make_one_syscall(&ctx);
    // assert_safe(&ctx);
    // make_one_syscall(&ctx);
    // assert(safe(&ctx));
    
    // assert(false);
    //make_one_syscall(&ctx);

    // for(int i = 0; i < 32; i++){
    //     invariant(i <= 32);
    //     make_one_syscall(&ctx);
    // }
    free((void*)ctx.membase);
}

