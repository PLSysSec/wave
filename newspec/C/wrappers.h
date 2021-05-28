int wasi_open(vmctx *ctx, const sandboxptr pathname, int flags);
int wasi_close(vmctx *ctx, int v_fd);
ssize_t wasi_read(vmctx *ctx, int v_fd, sandboxptr v_buf, size_t v_cnt);
ssize_t wasi_write(vmctx *ctx, int v_fd, const sandboxptr v_buf, size_t v_cnt);

