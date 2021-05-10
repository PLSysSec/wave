#include <unistd.h>
#include <sys/syscall.h>
#include "wrappers_utils.h"
#include "smack.h"
#include "model.h"

//All arguments are the Wasm arguments
int safe_open(vmctx* ctx, const sandboxptr pathname, int flags){

    hostptr host_pathname = path_from_sandbox(ctx, pathname);
    if (host_pathname == NULL)
        return -1;
   
    // assert( (host_pathname >= (char*)membase) && (host_pathname + PATH_MAX <= (char*)(membase + memlen)) );
    return model_open(host_pathname, flags);
}

int safe_close(vmctx* ctx, int fd){
    return model_close(fd);
}

ssize_t safe_read(vmctx* ctx, int fd, sandboxptr buf, size_t count){
    hostptr host_buf = sized_buf_from_sandbox(ctx, buf, count);
    if (host_buf == NULL)
        return -1;
    
    // assert( (host_buf >= (hostptr)membase) && (host_buf + count <= (hostptr)(membase + memlen)) );
    return model_read(fd, host_buf, count);
}

ssize_t safe_write(vmctx* ctx, int fd, const sandboxptr buf, size_t count){
    hostptr host_buf = sized_buf_from_sandbox(ctx, buf, count);
    if (host_buf == NULL)
        return -1;

    // assert( (host_buf >= (hostptr)membase) && (host_buf + count <= (hostptr)(membase + memlen)) );
    return model_write(fd, host_buf, count);
}


void make_one_syscall(vmctx* ctx){
    int choice = __VERIFIER_nondet_int();
    switch(choice){
        case 0:
        {
        int pathname = __VERIFIER_nondet_unsigned_int();
        int fd = __VERIFIER_nondet_int();
        safe_open(ctx, fd, pathname);
        break;
        }

        case 1:
        {
        int fd = __VERIFIER_nondet_int();
        safe_close(ctx, fd);
        break; 
        }
        
        case 2: 
        {
        int fd = __VERIFIER_nondet_int();
        int buf = __VERIFIER_nondet_unsigned_int();
        int size = __VERIFIER_nondet_unsigned_int();
        safe_read(ctx, fd, buf, size);
        break;
        }
        case 3:
        {
        int fd = __VERIFIER_nondet_int();
        int buf = __VERIFIER_nondet_unsigned_int();
        int size = __VERIFIER_nondet_unsigned_int();
        safe_write(ctx, fd, buf, size);
        break;
        }
        default:
        break; 
    }

}

int main(){
    if (!init_model()){
        cleanup_model();
        return -1;
    }
    assume(inode_exists != 0 && inodes != 0 && fd_open != 0 && fdtable != 0);
    vmctx ctx =  {0x40000000, 0x10000000};

    make_one_syscall(&ctx);
    // int pathname = __VERIFIER_nondet_unsigned_int();
    // int fd = __VERIFIER_nondet_int();
    // safe_open(fd, pathname);
    // int fd = __VERIFIER_nondet_int();
    // int buf = __VERIFIER_nondet_unsigned_int();
    // int size = __VERIFIER_nondet_unsigned_int();
    // safe_write(fd, buf, size);
    // int fd = __VERIFIER_nondet_int();
    // safe_close(fd);
    // for(int i = 0; i < 32; i++){
        // make_one_syscall();
    // }
    cleanup_model();
}
