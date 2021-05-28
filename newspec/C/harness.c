#include <stdlib.h>
#include "runtime.h"
#include "wrappers.h"
#include <smack.h>
#include <smack-contracts.h> 

//Non-deterministically select one system call and invoke it with symbolic args
void make_one_syscall(vmctx* ctx){
    requires(SAFE(ctx));
    ensures(SAFE(ctx));
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
    vmctx ctx = fresh_ctx();
    assume(ctx.membase < ctx.membase + ctx.memlen);

    make_one_syscall(&ctx);
    //make_one_syscall(&ctx);

    // for(int i = 0; i < 32; i++){
    //     invariant(i <= 32);
    //     make_one_syscall(&ctx);
    // }
    free((void*)ctx.membase);
}

