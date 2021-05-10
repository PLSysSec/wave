#include <unistd.h>
#include <sys/syscall.h>
#include "wrappers_utils.h"
#include "smack.h"
#include "model.h"

//All arguments are the Wasm arguments
int safe_open(const sandboxptr pathname, int flags){

    hostptr host_pathname = path_from_sandbox(pathname);
    if (host_pathname == NULL)
        return -1;
   
    assert( (host_pathname >= (char*)membase) && (host_pathname + PATH_MAX <= (char*)(membase + memlen)) );
    return model_open(host_pathname, flags);
}

int safe_close(int fd){
    return model_close(fd);
}

ssize_t safe_read(int fd, sandboxptr buf, size_t count){
    hostptr host_buf = sized_buf_from_sandbox(buf, count);
    if (host_buf == NULL)
        return -1;
    
    assert( (host_buf >= (hostptr)membase) && (host_buf + count <= (hostptr)(membase + memlen)) );
    return model_read(fd, host_buf, count);
}

ssize_t safe_write(int fd, const sandboxptr buf, size_t count){
    hostptr host_buf = sized_buf_from_sandbox(buf, count);
    if (host_buf == NULL)
        return -1;

    assert( (host_buf >= (hostptr)membase) && (host_buf + count <= (hostptr)(membase + memlen)) );
    return model_write(fd, host_buf, count);
}

