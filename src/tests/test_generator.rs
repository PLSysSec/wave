use crate::os;
use crate::runtime::fresh_ctx;
use crate::tcb::os_specs::linux::os_read;
use crate::tcb::os_specs::linux::*;
use crate::tcb::verification::ctx_safe;
use crate::types::{SboxPtr, VmCtx, LINEAR_MEM_SIZE};
use quickcheck::{QuickCheck, TestResult};
use quickcheck_macros;

#[quickcheck_macros::quickcheck]
fn check_fresh_ctx(homedir: String) -> TestResult {
    init();
    let result = fresh_ctx(homedir);
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_normalize_path(vec_path: PathBuf) -> TestResult {
    init();
    let path = &vec_path;
    let result = normalize_path(path);
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_wasi_prestat_dirname(
    mut vec_ctx: VmCtx,
    v_fd: u32,
    path: u32,
    path_len: u32,
) -> TestResult {
    init();
    let ctx = &mut vec_ctx;
    if !(true && ctx_safe(ctx)) {
        return TestResult::discard();
    }
    let result = wasi_prestat_dirname(ctx, v_fd, path, path_len);
    if !(true && ctx_safe(ctx)) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_wasi_fd_readdir(
    mut vec_ctx: VmCtx,
    v_fd: SboxFd,
    buf: SboxFd,
    buf_len: usize,
    cookie: u64,
) -> TestResult {
    init();
    let ctx = &mut vec_ctx;
    if !(true && ctx_safe(ctx)) {
        return TestResult::discard();
    }
    let result = wasi_fd_readdir(ctx, v_fd, buf, buf_len, cookie);
    if !(true && ctx_safe(ctx)) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_open(pathname: Vec<u8>, flags: i32) -> TestResult {
    init();
    let result = os_open(pathname, flags);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_close(fd: usize) -> TestResult {
    init();
    // HACK: added by el
    if fd == 1 {
        return TestResult::dicard();
    }
    let result = os_close(fd);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_read(fd: usize, mut vec_buf: Vec<u8>, cnt: usize) -> TestResult {
    init();
    let buf = &mut vec_buf[..];
    if !(buf.len() >= cnt) {
        return TestResult::discard();
    }
    let result = os_read(fd, buf, cnt);
    if !(!(result >= 0) || (buf.len() >= result as usize)) {
        return TestResult::failed();
    }
    if !(!(result >= 0) || (result as usize <= cnt)) {
        return TestResult::failed();
    }
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_pread(fd: usize, mut vec_buf: Vec<u8>, cnt: usize, offset: usize) -> TestResult {
    init();
    let buf = &mut vec_buf[..];
    if !(buf.len() >= cnt) {
        return TestResult::discard();
    }
    let result = os_pread(fd, buf, cnt, offset);
    if !(!(result >= 0) || (buf.len() >= result as usize)) {
        return TestResult::failed();
    }
    if !(!(result >= 0) || (result as usize <= cnt)) {
        return TestResult::failed();
    }
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_write(fd: usize, vec_buf: Vec<u8>, cnt: usize) -> TestResult {
    init();
    let buf = &vec_buf[..];
    if !(buf.len() >= cnt) {
        return TestResult::discard();
    }
    let result = os_write(fd, buf, cnt);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_pwrite(fd: usize, vec_buf: Vec<u8>, cnt: usize, offset: usize) -> TestResult {
    init();
    let buf = &vec_buf[..];
    if !(buf.len() >= cnt) {
        return TestResult::discard();
    }
    let result = os_pwrite(fd, buf, cnt, offset);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_seek(fd: usize, offset: i64, whence: i32) -> TestResult {
    init();
    let result = os_seek(fd, offset, whence);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_advise(fd: usize, offset: i64, len: i64, advice: i32) -> TestResult {
    init();
    let result = os_advise(fd, offset, len, advice);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_allocate(fd: usize, offset: i64, len: i64) -> TestResult {
    init();
    let result = os_allocate(fd, offset, len);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_sync(fd: usize) -> TestResult {
    init();
    let result = os_sync(fd);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_datasync(fd: usize) -> TestResult {
    init();
    let result = os_datasync(fd);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_fstat(fd: usize, mut vec_stat: libc::stat) -> TestResult {
    init();
    let stat = &mut vec_stat;
    let result = os_fstat(fd, stat);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_fstatat(fd: usize, path: Vec<u8>, mut vec_stat: libc::stat, flags: i32) -> TestResult {
    init();
    let stat = &mut vec_stat;
    let result = os_fstatat(fd, path, stat, flags);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_fgetfl(fd: usize) -> TestResult {
    init();
    let result = os_fgetfl(fd);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_fsetfl(fd: usize, flags: libc::c_int) -> TestResult {
    init();
    let result = os_fsetfl(fd, flags);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_ftruncate(fd: usize, length: libc::off_t) -> TestResult {
    init();
    let result = os_ftruncate(fd, length);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_linkat(
    old_fd: usize,
    old_path: Vec<u8>,
    new_fd: usize,
    new_path: Vec<u8>,
    flags: i32,
) -> TestResult {
    init();
    let result = os_linkat(old_fd, old_path, new_fd, new_path, flags);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_mkdirat(dir_fd: usize, pathname: Vec<u8>, mode: libc::mode_t) -> TestResult {
    init();
    let result = os_mkdirat(dir_fd, pathname, mode);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_readlinkat(
    dir_fd: usize,
    pathname: Vec<u8>,
    mut vec_buf: Vec<u8>,
    cnt: usize,
) -> TestResult {
    init();
    let buf = &mut vec_buf[..];
    if !(buf.len() >= cnt) {
        return TestResult::discard();
    }
    let result = os_readlinkat(dir_fd, pathname, buf, cnt);
    if !(!(result >= 0) || (buf.len() == result as usize)) {
        return TestResult::failed();
    }
    if !(!(result >= 0) || (result as usize <= cnt)) {
        return TestResult::failed();
    }
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_unlinkat(dir_fd: usize, pathname: Vec<u8>, flags: libc::c_int) -> TestResult {
    init();
    let result = os_unlinkat(dir_fd, pathname, flags);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_renameat(
    old_dir_fd: usize,
    old_pathname: Vec<u8>,
    new_dir_fd: usize,
    new_pathname: Vec<u8>,
) -> TestResult {
    init();
    let result = os_renameat(old_dir_fd, old_pathname, new_dir_fd, new_pathname);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_symlinkat(old_pathname: Vec<u8>, dir_fd: usize, new_pathname: Vec<u8>) -> TestResult {
    init();
    let result = os_symlinkat(old_pathname, dir_fd, new_pathname);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_futimens(fd: usize, vec_specs: Vec<libc::timespec>) -> TestResult {
    init();
    let specs = &vec_specs;
    if !(specs.capacity() >= 2) {
        return TestResult::discard();
    }
    let result = os_futimens(fd, specs);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_utimensat(
    fd: usize,
    pathname: Vec<u8>,
    vec_specs: Vec<libc::timespec>,
    flags: libc::c_int,
) -> TestResult {
    init();
    let specs = &vec_specs;
    if !(specs.capacity() >= 2) {
        return TestResult::discard();
    }
    let result = os_utimensat(fd, pathname, specs, flags);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_clock_get_time(clock_id: libc::clockid_t, mut vec_spec: libc::timespec) -> TestResult {
    init();
    let spec = &mut vec_spec;
    let result = os_clock_get_time(clock_id, spec);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_clock_get_res(clock_id: libc::clockid_t, mut vec_spec: libc::timespec) -> TestResult {
    init();
    let spec = &mut vec_spec;
    let result = os_clock_get_res(clock_id, spec);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_getrandom(mut vec_buf: Vec<u8>, cnt: usize, flags: u32) -> TestResult {
    init();
    let buf = &mut vec_buf[..];
    if !(buf.len() >= cnt) {
        return TestResult::discard();
    }
    let result = os_getrandom(buf, cnt, flags);
    if !(!(result >= 0) || (buf.len() >= result as usize)) {
        return TestResult::failed();
    }
    if !(!(result >= 0) || (result as usize <= cnt)) {
        return TestResult::failed();
    }
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_recv(fd: usize, mut vec_buf: Vec<u8>, cnt: usize, flags: u32) -> TestResult {
    init();
    let buf = &mut vec_buf[..];
    if !(buf.len() >= cnt) {
        return TestResult::discard();
    }
    let result = os_recv(fd, buf, cnt, flags);
    if !(!(result >= 0) || (buf.len() >= result as usize)) {
        return TestResult::failed();
    }
    if !(!(result >= 0) || (result as usize <= cnt)) {
        return TestResult::failed();
    }
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_send(fd: usize, vec_buf: Vec<u8>, cnt: usize, flags: u32) -> TestResult {
    init();
    let buf = &vec_buf[..];
    if !(buf.len() >= cnt) {
        return TestResult::discard();
    }
    let result = os_send(fd, buf, cnt, flags);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_shutdown(fd: usize, how: libc::c_int) -> TestResult {
    init();
    let result = os_shutdown(fd, how);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_nanosleep(vec_req: libc::timespec, mut vec_rem: libc::timespec) -> TestResult {
    init();
    let req = &vec_req;
    let rem = &mut vec_rem;
    let result = os_nanosleep(req, rem);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_poll(mut vec_pollfd: libc::pollfd, timeout: libc::c_int) -> TestResult {
    init();
    let pollfd = &mut vec_pollfd;
    let result = os_poll(pollfd, timeout);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_getdents64(fd: usize, mut vec_dirp: Vec<u8>, count: usize) -> TestResult {
    init();
    let dirp = &mut vec_dirp;
    let result = os_getdents64(fd, dirp, count);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_socket(domain: i32, ty: i32, protocol: i32) -> TestResult {
    init();
    let result = os_socket(domain, ty, protocol);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_os_connect(sockfd: usize, vec_addr: libc::sockaddr_in, addrlen: u32) -> TestResult {
    init();
    let addr = &vec_addr;
    let result = os_connect(sockfd, addr, addrlen);
    if !(true) {
        return TestResult::failed();
    }
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_vec_checked_lookup(vec_vec: Vec<RuntimeResult<HostFd>>, index: SboxFd) -> TestResult {
    init();
    let vec = &vec_vec;
    if !(index < MAX_SBOX_FDS) {
        return TestResult::discard();
    }
    let result = vec_checked_lookup(vec, index);
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_nth_bit_set(bv: u16, n: i32) -> TestResult {
    init();
    let result = nth_bit_set(bv, n);
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_nth_bit_set_u32(bv: u32, n: u32) -> TestResult {
    init();
    let result = nth_bit_set_u32(bv, n);
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_with_nth_bit_set(bv: u16, n: i32) -> TestResult {
    init();
    let result = with_nth_bit_set(bv, n);
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_bitwise_and(bv1: i32, bv2: i32) -> TestResult {
    init();
    let result = bitwise_and(bv1, bv2);
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_bitwise_and_u32(bv1: u32, bv2: u32) -> TestResult {
    init();
    let result = bitwise_and_u32(bv1, bv2);
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_bitwise_or(bv1: i32, bv2: i32) -> TestResult {
    init();
    let result = bitwise_or(bv1, bv2);
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_fresh_stat() -> TestResult {
    init();
    let result = fresh_stat();
    TestResult::passed()
}
#[quickcheck_macros::quickcheck]
fn check_slice_mem_mut(ptr: SboxPtr, len: u32) -> TestResult {
    init();
    let mut ctx = fresh_ctx(".".to_string());
    ctx.check_slice_mem_mut_impl(ptr, len)
}
#[quickcheck_macros::quickcheck]
fn check_copy_arg_buffer_to_sandbox(dst: SboxPtr, n: u32) -> TestResult {
    init();
    let mut ctx = fresh_ctx(".".to_string());
    ctx.check_copy_arg_buffer_to_sandbox_impl(dst, n)
}
#[quickcheck_macros::quickcheck]
fn check_copy_environ_buffer_to_sandbox(dst: SboxPtr, n: u32) -> TestResult {
    init();
    let mut ctx = fresh_ctx(".".to_string());
    ctx.check_copy_environ_buffer_to_sandbox_impl(dst, n)
}
#[quickcheck_macros::quickcheck]
fn check_memcpy_from_sandbox(mut vec_dst: Vec<u8>, src: SboxPtr, n: u32) -> TestResult {
    init();
    let dst = &mut vec_dst;
    let mut ctx = fresh_ctx(".".to_string());
    ctx.check_memcpy_from_sandbox_impl(dst, src, n)
}
#[quickcheck_macros::quickcheck]
fn check_memcpy_to_sandbox(dst: SboxPtr, vec_src: Vec<u8>, n: u32) -> TestResult {
    init();
    let src = &vec_src;
    let mut ctx = fresh_ctx(".".to_string());
    ctx.check_memcpy_to_sandbox_impl(dst, src, n)
}
#[quickcheck_macros::quickcheck]
fn check_resolve_path(in_path: Vec<u8>) -> TestResult {
    init();
    let mut ctx = fresh_ctx(".".to_string());
    ctx.check_resolve_path_impl(in_path)
}
impl VmCtx {
    fn check_slice_mem_mut_impl(&mut self, ptr: SboxPtr, len: u32) -> TestResult {
        if !(self.fits_in_lin_mem(ptr, len)) {
            return TestResult::discard();
        }
        if !(true) {
            return TestResult::discard();
        }
        let result = self.slice_mem_mut(ptr, len);
        if !(true) {
            return TestResult::failed();
        }
        if !(result.len() == (len as usize)) {
            return TestResult::failed();
        }
        if !(true) {
            return TestResult::failed();
        }
        TestResult::passed()
    }
    fn check_copy_arg_buffer_to_sandbox_impl(&mut self, dst: SboxPtr, n: u32) -> TestResult {
        if !(self.arg_buffer.len() == (n as usize)) {
            return TestResult::discard();
        }
        if !(true && ctx_safe(self)) {
            return TestResult::discard();
        }
        let result = self.copy_arg_buffer_to_sandbox(dst, n);
        if !(true && ctx_safe(self)) {
            return TestResult::failed();
        }
        TestResult::passed()
    }
    fn check_copy_environ_buffer_to_sandbox_impl(&mut self, dst: SboxPtr, n: u32) -> TestResult {
        if !(self.env_buffer.len() == (n as usize)) {
            return TestResult::discard();
        }
        if !(true && ctx_safe(self)) {
            return TestResult::discard();
        }
        let result = self.copy_environ_buffer_to_sandbox(dst, n);
        if !(true && ctx_safe(self)) {
            return TestResult::failed();
        }
        TestResult::passed()
    }
    fn check_memcpy_from_sandbox_impl(
        &self,
        dst: &mut Vec<u8>,
        src: SboxPtr,
        n: u32,
    ) -> TestResult {
        if !(dst.capacity() >= (n as usize)) {
            return TestResult::discard();
        }
        if !(self.fits_in_lin_mem(src, n)) {
            return TestResult::discard();
        }
        if !(true && ctx_safe(self)) {
            return TestResult::discard();
        }
        let self_dot_memlen_old = self.memlen;
        let result = self.memcpy_from_sandbox(dst, src, n);
        if !(true && ctx_safe(self)) {
            return TestResult::failed();
        }
        if !(dst.len() == (n as usize)) {
            return TestResult::failed();
        }
        if !(self.memlen == self_dot_memlen_old) {
            return TestResult::failed();
        }
        TestResult::passed()
    }
    fn check_memcpy_to_sandbox_impl(&mut self, dst: SboxPtr, src: &Vec<u8>, n: u32) -> TestResult {
        if !(src.len() >= (n as usize)) {
            return TestResult::discard();
        }
        if !(self.fits_in_lin_mem(dst, n)) {
            return TestResult::discard();
        }
        if !(true && ctx_safe(self)) {
            return TestResult::discard();
        }
        let self_dot_memlen_old = self.memlen;
        let result = self.memcpy_to_sandbox(dst, src, n);
        if !(true && ctx_safe(self)) {
            return TestResult::failed();
        }
        if !(self_dot_memlen_old == self.memlen) {
            return TestResult::failed();
        }
        TestResult::passed()
    }
    fn check_resolve_path_impl(&self, in_path: Vec<u8>) -> TestResult {
        let result = self.resolve_path(in_path);
        TestResult::passed()
    }
}
