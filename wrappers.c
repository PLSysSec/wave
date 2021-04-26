#include <unistd.h>
#include <sys/syscall.h>
#include "wrappers_utils.h"
#include "smack.h"

//All arguments are the Wasm arguments
int safe_open(const sandboxptr pathname, int flags){

    hostptr host_pathname = path_from_sandbox(pathname);
    if (host_pathname == NULL)
        return -1;
   
    assert( (host_pathname >= (void*)membase) && (host_pathname <= (void*)(membase + memlen)) );
    return syscall(SYS_open, 
        host_pathname, 
        flags, 
        NULL);
}

int safe_close(int fd){
    return syscall(SYS_close, 
        fd, 
        NULL);
}

ssize_t safe_read(int fd, sandboxptr buf, size_t count){
    hostptr host_buf = sized_buf_from_sandbox(buf, count);
    if (host_buf == NULL)
        return -1;
    
    assert( (host_buf >= (void*)membase) && (host_buf <= (void*)(membase + memlen)) );
    return syscall(SYS_read, 
        fd, 
        host_buf, 
        count, 
        NULL);
}

ssize_t safe_write(int fd, const sandboxptr buf, size_t count){
    hostptr host_buf = sized_buf_from_sandbox(buf, count);
    if (host_buf == NULL)
        return -1;

    assert( (host_buf >= (void*)membase) && (host_buf <= (void*)(membase + memlen)) );
    return syscall(SYS_write, 
        fd, 
        host_buf, 
        count, 
        NULL);
}

/*
int main(){

}
*/
