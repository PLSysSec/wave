#include <unistd.h>
#include <sys/syscall.h>
#include <sys/types.h>
#include <sys/stat.h>
#include "wrappers_utils.h"
#include "smack.h"
//All arguments are the Wasm arguments
int safe_open(const sandboxptr pathname, int flags){
    hostptr host_pathname = path_from_sandbox(pathname);
    if (host_pathname == NULL)
        return -1;
   
    assert( (host_pathname >= (char*)membase) && (host_pathname + PATH_MAX <= (char*)(membase + memlen)) );
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
    
    assert( (host_buf >= (hostptr)membase) && (host_buf + count <= (hostptr)(membase + memlen)) );
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

    assert( (host_buf >= (hostptr)membase) && (host_buf + count <= (hostptr)(membase + memlen)) );
    return syscall(SYS_write, 
        fd, 
        host_buf, 
        count, 
        NULL);
}


// Policy(fstat) = {  }
int safe_fstat(int fd, sandboxptr statbuf){
    size_t __statbuf_size = sizeof(statbuf);
    hostptr host_statbuf = sized_buf_from_sandbox(statbuf, __statbuf_size);
    if (host_statbuf == NULL)
        return -1;

    assert( (host_statbuf >= (hostptr)membase) && (host_statbuf + __statbuf_size <= (hostptr)(membase + memlen)) );
    return syscall(SYS_fstat, 
        fd, 
        host_statbuf, 
        NULL);
}

// Policy(lseek) = {  }
off_t safe_lseek(int fd, off_t offset, int whence){
    return syscall(SYS_lseek, 
        fd, 
        offset, 
        whence, 
        NULL);
}

// Policy(dup2) = {  }
int safe_dup2(int oldfd, int newfd){

    return syscall(SYS_dup2, 
        oldfd, 
        newfd,  
        NULL);
}

// Policy(unlink) = { pathname = PathType }
int safe_unlink(const sandboxptr pathname){
    hostptr host_pathname = path_from_sandbox(pathname);
    if (host_pathname == NULL)
        return -1;

    assert( (host_pathname >= (char*)membase) && (host_pathname + PATH_MAX <= (char*)(membase + memlen)) );
    return syscall(SYS_unlink, 
        host_pathname,
        NULL);
}

// Policy(symlink) = { target = PathType, linkpath = PathType }
int safe_symlink(const sandboxptr target, const sandboxptr linkpath){
    hostptr host_target = path_from_sandbox(target);
    if (host_target == NULL)
        return -1;

    hostptr host_linkpath = path_from_sandbox(linkpath);
    if (host_linkpath == NULL)
        return -1;

    assert( (host_target >= (char*)membase) && (host_target + PATH_MAX <= (char*)(membase + memlen)) );
    assert( (host_linkpath >= (char*)membase) && (host_linkpath + PATH_MAX <= (char*)(membase + memlen)) );
    
    return syscall(SYS_symlink, 
        host_target,
        host_linkpath,
        NULL);
}

// Policy(readlink) = { pathname = Pathtype, buf = SizedBuf(bufsiz) }
ssize_t safe_readlink(const sandboxptr pathname, sandboxptr buf, size_t bufsiz){
    hostptr host_pathname = path_from_sandbox(pathname);
    if (host_pathname == NULL)
        return -1;

    hostptr host_buf = sized_buf_from_sandbox(buf, bufsiz);
    if (host_buf == NULL)
        return -1;

    assert( (host_pathname >= (char*)membase) && (host_pathname + PATH_MAX <= (char*)(membase + memlen)) );
    assert( (host_buf >= (hostptr)membase) && (host_buf + bufsiz <= (hostptr)(membase + memlen)) );

    return syscall(SYS_readlink, 
        host_pathname,
        host_buf,
        bufsiz,
        NULL);
}

// Policy(getcwd) = { buf = SizedBuf(size) }
sandboxptr safe_getcwd(sandboxptr buf, size_t size){
    hostptr host_buf = sized_buf_from_sandbox(buf, size);
    if (host_buf == NULL)
        return -1;
    
    assert( (host_buf >= (hostptr)membase) && (host_buf + size <= (hostptr)(membase + memlen)) );
    sandboxptr __ret = ptr_to_sandbox(
        syscall(SYS_getcwd, 
            host_buf,
            size,
            NULL)
        );
    
    return __ret;
}

// Policy(chdir) = { path = PathType }
int safe_chdir(const sandboxptr path){
     hostptr host_path = path_from_sandbox(path);
    if (host_path == NULL)
        return -1;
   
    assert( (host_path >= (char*)membase) && (host_path + PATH_MAX <= (char*)(membase + memlen)) );
    return syscall(SYS_chdir, 
        host_path,
        NULL);
}

// Policy(mkdir) = { pathname = PathType }
int safe_mkdir(const sandboxptr pathname, mode_t mode){
     hostptr host_pathname = path_from_sandbox(pathname);
    if (host_pathname == NULL)
        return -1;
   
    assert( (host_pathname >= (char*)membase) && (host_pathname + PATH_MAX <= (char*)(membase + memlen)) );

    return syscall(SYS_mkdir, 
        host_pathname,
        mode,
        NULL);
}

// Policy(rmdir) = { pathname = PathType }
int safe_rmdir(const sandboxptr pathname){
    assert(false);
    hostptr host_pathname = path_from_sandbox(pathname);
    if (host_pathname == NULL)
        return -1;
    assert( (host_pathname >= (char*)membase) && (host_pathname + PATH_MAX <= (char*)(membase + memlen)) );
    return syscall(SYS_rmdir, 
        host_pathname,
        NULL);
}


