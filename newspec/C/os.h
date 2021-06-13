#include <smack.h>
#include <smack-contracts.h>
#include <sys/syscall.h>
#include "runtime.h"

#define ACCESS_MEM(ptr, size) memset(ptr, 0, size)  
#define ACCESS_FD(fd) (assert(true)) 
#define ACCESS_PATH(path) (assert(true)) 

//TODO: { Path Sandboxing }
int os_open(const char *pathname, int flags){
    ACCESS_PATH(pathname);
    return syscall(SYS_open, 
        pathname, 
        flags, 
        NULL);
}


int os_close(int fd){
    ACCESS_FD(fd);
    return syscall(SYS_close, 
        fd, 
        NULL);
}

ssize_t os_read(int fd, void *buf, size_t cnt) { 
    ACCESS_MEM(buf, cnt); 
    ACCESS_FD(fd); 
    return syscall(SYS_read, 
        fd, 
        buf, 
        cnt, 
        NULL);
}

ssize_t os_write(int fd, void *buf, size_t cnt) { 
    ACCESS_MEM(buf, cnt);
    ACCESS_FD(fd);
    return syscall(SYS_write, 
        fd, 
        buf, 
        cnt, 
        NULL);
}
