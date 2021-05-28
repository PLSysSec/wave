#include <unistd.h>
#include <sys/types.h>
#include <sys/stat.h>
#include <stdlib.h>
#include "runtime.h"
#include "os.h"
#include <smack.h>
#include <smack-contracts.h> 

// predicate SFISafe(ctx) =
// not exists. a. a < ctx.membase | a >= ctx.membase + ctx.memlength. access(a)

// predicate FdSafe(ctx) =
// not exists. fd. inRevFdMap(ctx, fd) & os_read_fd(fd)

// validctx(ctx):
// ctx.membase < ctx.membase + ctx.membaseLen
// forall fd. inRevFdMap(ctx fd) => inFdMap(ctx, translateFd(ctx, fd))
// forall vfd. inFdMap(ctx vfd) => inRevFdMap(ctx, translateFd(ctx, vfd))

// WASIRead(ctx): ... write at most v_cnt bytes etc.

// validCtx(ctx), SFISafe(ctx), FdSafe(ctx) = ...


//pre: {..., }
//post: {..., inFDMap(ctx, fd), inRevFDMap(ctx, translate_fd(fd) )}
int wasi_open(vmctx *ctx, const sandboxptr pathname, int flags){
    requires(SAFE(ctx));
    ensures(SAFE(ctx));
    hostptr host_buffer = copy_buf_from_sandbox(ctx, pathname, PATH_MAX);
    if (host_buffer == NULL){
        return -1;
    }
    hostptr host_pathname = malloc(PATH_MAX);
    resolve_path(ctx, host_buffer, host_pathname);
    int fd = os_open(host_pathname, flags);
    
    int sbx_fd = create_seal(ctx, fd, ctx->counter++);
    free(host_buffer);
    free(host_pathname);
    return sbx_fd;
}

//pre: {...}
//post: {..., !inFDMap(ctx, fd), !inRevFDMap(ctx, translate_fd(fd) )}
int wasi_close(vmctx *ctx, int v_fd){
    requires(SAFE(ctx));
    ensures(SAFE(ctx));
    // ensures(ctx->membase < ctx->membase + ctx->memlen);
    if ((v_fd < 0) || (v_fd >= MAX_SANDBOX_FDS) || !in_fd_map(ctx, v_fd)){
        // errno = EBADF;
        return -1;
    }
    int fd = translate_fd(ctx, v_fd);
    delete_seal(ctx, v_fd);
    return close(fd);
}

// pre: { validCtx(ctx)}
// post: { validCtx(ctx), SFISafe(ctx), FdSafe(ctx), WASIRead(ctx) }
ssize_t wasi_read(vmctx *ctx, int v_fd, sandboxptr v_buf, size_t v_cnt) {
  requires(SAFE(ctx));
  ensures(SAFE(ctx));
  void *buf = swizzle(ctx, v_buf);

  if (!inMemRegion(ctx, buf) || (v_cnt >= ctx->memlen) || !fitsInMemRegion(ctx, buf, v_cnt)){
      return -1;
  }
  if (v_fd < 0 || v_fd >= MAX_SANDBOX_FDS || !in_fd_map(ctx, v_fd)){
        return -1;
  }
  int fd = translate_fd(ctx, v_fd);
  return os_read(fd, buf, v_cnt);
}


ssize_t wasi_write(vmctx *ctx, int v_fd, const sandboxptr v_buf, size_t v_cnt) {
  requires(SAFE(ctx));
  ensures(SAFE(ctx));
  void *buf = swizzle(ctx, v_buf);

  if (!inMemRegion(ctx, buf) || (v_cnt >= ctx->memlen) || !fitsInMemRegion(ctx, buf, v_cnt)){
      return -1;
  }
  if (v_fd < 0 || v_fd >= MAX_SANDBOX_FDS || !in_fd_map(ctx, v_fd)){
        return -1;
  }
  int fd = translate_fd(ctx, v_fd);
  return os_write(fd, buf, v_cnt);
}

